use crate::video::VideoError;
use crate::video::VideoSource;
use async_trait::async_trait;
use livekit::options::VideoCodec;
use livekit::webrtc::prelude::VideoFrame;
use pipewire as pw;
use pipewire::stream::StreamBox;
//use pw::prelude::*;
use pw::stream::{Stream, StreamFlags};
use pw::spa::param::video::VideoFormat;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::watch;
use yuv::{rgb_to_yuv420, YuvChromaSubsampling, YuvConversionMode, YuvPlanarImageMut, YuvRange, YuvStandardMatrix};
use pipewire::main_loop::MainLoopBox;

pub fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64
}

pub struct PipewireSource {
    width: u32,
    height: u32,
    fps: u32,
    receiver: watch::Receiver<Option<Vec<u8>>>,
    stop_flag: Arc<Mutex<bool>>,
}

impl PipewireSource {
    pub fn new(width: u32, height: u32, fps: u32) -> Result<Self, VideoError> {
        pw::init();

        let (sender, receiver) = watch::channel(None);
        let stop_flag = Arc::new(Mutex::new(false));
        let stop_flag_thread = Arc::clone(&stop_flag);

        std::thread::spawn(move || {
            let mainloop = MainLoopBox::new(None).unwrap();
            let context = pw::context::ContextBox::new(&mainloop, None).unwrap();
            let core = context.connect(None).unwrap();

            let params = [libspa::param::format::VideoFormat::I420];

            let stream = StreamBox::new(
                &core,
                "PipewireSource",
                pw::properties::properties! {
                    "media.class" => "Video/Source",
                },
            )
            .unwrap();

            let sender = Arc::new(Mutex::new(sender));

            let _listener = stream
                .add_local_listener()
                .process({
                    let sender = Arc::clone(&sender);
                    move |stream| {
                        while let Some(mut buffer) = stream.dequeue_buffer() {
                            if let Some(data) = buffer.datas_mut().first_mut() {
                                let bytes = data.as_slice().to_vec();
                                // Update latest frame (watch replaces old value efficiently)
                                if let Ok(tx) = sender.lock() {
                                    let _ = tx.send(Some(bytes));
                                }
                            }
                            stream.queue_buffer(buffer);
                        }
                    }
                })
                .register();

            stream
                .connect(
                    pw::Direction::Input,
                    None,
                    StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS,
                    &params,
                )
                .unwrap();

            while !*stop_flag_thread.lock().unwrap() {
                
            }

            mainloop.quit();
        });

        Ok(Self {
            width,
            height,
            fps,
            receiver,
            stop_flag,
        })
    }
}

fn image_to_i420_buffer(img: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<I420Buffer, VideoError> {
    let width = img.width();
    let height = img.height();
    let rgba_stride = 4 * width;

    let mut planar =
        YuvPlanarImageMut::<u8>::alloc(width, height, YuvChromaSubsampling::Yuv420);
    rgb_to_yuv420(
        &mut planar,
        img.as_raw(),
        rgba_stride,
        YuvRange::Limited,
        YuvStandardMatrix::Bt601,
        YuvConversionMode::Balanced,
    )?;

    let mut buffer = I420Buffer::with_strides(width, height, width, width / 2, width / 2);
    let (y, u, v) = buffer.data_mut();
    y.copy_from_slice(planar.y_plane.borrow());
    u.copy_from_slice(planar.u_plane.borrow());
    v.copy_from_slice(planar.v_plane.borrow());
    Ok(buffer)
}

#[async_trait]
impl VideoSource for PipewireSource {
    type Buf = I420Buffer;

    async fn start(&mut self) -> Result<(), VideoError> {
        Ok(())
    }

    async fn next_frame(&mut self) -> Result<VideoFrame<Self::Buf>, VideoError> {
        let mut rx = self.receiver.clone();

        // Wait until next frame is available
        loop {
            rx.changed().await.map_err(|_| VideoError::EmptySource)?;
            if let Some(frame_data) = rx.borrow().clone() {
                let image = ImageBuffer::from_raw(self.width, self.height, frame_data)
                    .ok_or(VideoError::InvalidFrame)?;
                let buffer = image_to_i420_buffer(&image)?;
                let ts = unix_now();
                return Ok(VideoFrame {
                    rotation: VideoRotation::VideoRotation0,
                    buffer,
                    timestamp_us: ts,
                });
            }
        }
    }

    async fn stop(&mut self) -> Result<(), VideoError> {
        if let Ok(mut stop) = self.stop_flag.lock() {
            *stop = true;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "pipewire-async"
    }

    fn resolution(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn preferred_codec(&self) -> Option<VideoCodec> {
        Some(VideoCodec::H264)
    }
}