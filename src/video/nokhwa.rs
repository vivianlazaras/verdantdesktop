use nokhwa::Camera;

use crate::video::VideoError;
use crate::video::VideoSource;
use async_trait::async_trait;
use livekit::options::TrackPublishOptions;
use livekit::options::VideoCodec;
use livekit::track::LocalVideoTrack;
use livekit::webrtc::prelude::VideoFrame;
use livekit::webrtc::video_frame::I420Buffer;
use livekit::webrtc::video_frame::VideoBuffer;
use livekit::webrtc::video_source::native::NativeVideoSource;
use livekit::webrtc::video_source::RtcVideoSource;
use livekit::webrtc::video_source::VideoResolution;
use nokhwa::utils::{ApiBackend, CameraIndex, FrameFormat};
use std::thread;

use livekit::webrtc::video_frame::VideoRotation;
use tokio::runtime::Handle;
use tokio::sync::mpsc::{self, UnboundedSender, UnboundedReceiver};
use tokio::task;
use tokio::task::JoinHandle;
use nokhwa::pixel_format::RgbAFormat;
use nokhwa::Buffer;
use image::{ImageBuffer, Rgb, Rgba};
use yuv::{rgb_to_yuv420, YuvChromaSubsampling,YuvPlanarImageMut, YuvConversionMode, YuvRange, YuvStandardMatrix};
use std::sync::atomic::{AtomicBool, Ordering};

use std::time::{SystemTime, UNIX_EPOCH};

pub fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64
}

pub struct NokhwaSource {
    camera: UnboundedReceiver<nokhwa::Buffer>,
    width: u32,
    height: u32,
    index: CameraIndex,
    fps: u32,
    format: FrameFormat,
    sender: UnboundedSender<Buffer>,
    open: AtomicBool,
}

impl Default for NokhwaSource {
    fn default() -> Self {
        Self::safe_assumption(CameraIndex::Index(0)).unwrap()
    }
}

impl NokhwaSource {

    pub fn safe_assumption(index: CameraIndex) -> Result<Self, VideoError> {
        Self::new(index, FrameFormat::MJPEG, 30, 640, 480)
    }

    pub fn new(index: CameraIndex, format: FrameFormat, fps: u32, width: u32, height: u32) -> Result<Self, VideoError> {
        let (sender, camera) = mpsc::unbounded_channel();
        Ok(Self {
            camera,
            width,
            height,
            sender,
            index,
            format,
            fps,
            open: AtomicBool::new(false)
        })
    }
    pub fn open(&self) -> Result<std::thread::JoinHandle<()>, VideoError> {
        let index = self.index.clone();
        let format = self.format.clone();
        let sender = self.sender.clone();
        let width = self.width;
        let height = self.height;
        let fps = self.fps;
        self.open.store(true, Ordering::Relaxed);
        let ptr = self.open.as_ptr();
        let open = unsafe { AtomicBool::from_ptr(ptr) };
        let handle = thread::spawn(move || {
            let mut cam = Camera::new_with(
                index,
                width,
                height,
                fps,
                format,
                ApiBackend::Auto,
            )
            .unwrap();
            match cam.open_stream() {
                Ok(cam) => cam,
                Err(e) => {
                    return;
                }
            }
            while let Ok(buffer) = cam.frame() {
                sender.send(buffer);
            }
        });
        Ok(handle)
    }
}

pub fn image_to_i420_buffer(img: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<I420Buffer, VideoError> {
    let width = img.width();
    let height = img.height();

    let rgba_stride = 4 * width; // R,G,B,A per pixel

    let mut planar =
        YuvPlanarImageMut::<u8>::alloc(width as u32, height as u32, YuvChromaSubsampling::Yuv420);
    // Convert RGBA → YUV420 (BT.601, Limited Range)
    rgb_to_yuv420(
        &mut planar,
        img.as_raw(),
        rgba_stride as u32,
        YuvRange::Limited,
        YuvStandardMatrix::Bt601,
        YuvConversionMode::Balanced,
    )?;

    // Now map this PlanarYuv into a LiveKit I420Buffer
    let mut buffer = I420Buffer::with_strides(width, height, width, width / 2, width / 2);
    let (y_plane, u_plane, v_plane) = buffer.data_mut();

    y_plane.copy_from_slice(planar.y_plane.borrow());
    u_plane.copy_from_slice(planar.u_plane.borrow());
    v_plane.copy_from_slice(planar.v_plane.borrow());

    Ok(buffer)
}

#[async_trait::async_trait]
impl VideoSource for NokhwaSource {
    type Buf = I420Buffer;
    async fn start(&mut self) -> Result<(), VideoError> {
        self.open()?;
        Ok(())
    }

    async fn next_frame(&mut self) -> Result<VideoFrame<Self::Buf>, VideoError> {
        let frame = match self.camera.recv().await {
            Some(frame) => frame,
            None => return Err(VideoError::EmptySource),
        };
        let image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = frame.decode_image::<RgbAFormat>()?;
        let buffer = image_to_i420_buffer(&image)?;
        let ts = unix_now();
        Ok(VideoFrame {
            rotation: VideoRotation::VideoRotation0,
            buffer,
            timestamp_us: ts,
        })
    }

    async fn stop(&mut self) -> Result<(), VideoError> {
        self.open.store(false, Ordering::Relaxed);
        Ok(())
    }

    fn name(&self) -> &str {
        "camera"
    }

    fn resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn preferred_codec(&self) -> Option<VideoCodec> {
        Some(VideoCodec::H264)
    }
}
