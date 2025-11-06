pub mod nokhwa;

use ::nokhwa::NokhwaError;
use async_trait::async_trait;
use livekit::options::{TrackPublishOptions, VideoCodec};
use livekit::track::LocalVideoTrack;
use livekit::webrtc::prelude::VideoFrame;
use livekit::webrtc::video_frame::VideoBuffer;
use livekit::webrtc::video_source::native::NativeVideoSource;
use livekit::webrtc::video_source::RtcVideoSource;
use livekit::webrtc::video_source::VideoResolution;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::task;
use tokio::task::JoinHandle;

/// Functionality for providing different video sources.
use thiserror::Error;
#[derive(Debug, Error)]
pub enum VideoError {
    #[error("nokhwa error: {0}")]
    Nokhwa(#[from] NokhwaError),
    #[error("no data to return")]
    EmptySource,
    #[error("failed to convert to YUV: {0}")]
    YUVError(#[from] yuv::YuvError),
}

/// Trait for any object that can produce video frames.
#[async_trait]
pub trait VideoSource: Send + Sync {
    type Buf: AsRef<dyn VideoBuffer>;
    /// Initialize the source (open device, allocate buffers, etc.)
    async fn start(&mut self) -> Result<(), VideoError>;

    /// Capture or retrieve the next frame.
    async fn next_frame(&mut self) -> Result<VideoFrame<Self::Buf>, VideoError>;

    /// Stop or release the source.
    async fn stop(&mut self) -> Result<(), VideoError>;

    /// Optional: human-readable name of the source (for UI or logging).
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Returns the nominal width and height for this video source.
    fn resolution(&self) -> (u32, u32);

    /// Optionally return a preferred LiveKit video encoding (e.g. VP8, H264).
    fn preferred_codec(&self) -> Option<VideoCodec> {
        None
    }

    /// Create a LiveKit LocalVideoSource for publishing frames.
    ///
    /// This is a convenience so that each source can customize format hints.
    fn create_local_source(&self) -> NativeVideoSource {
        let (width, height) = self.resolution();
        let resolution = VideoResolution { width, height };
        NativeVideoSource::new(resolution)
    }

    /// Build the LiveKit LocalVideoTrack for this source.
    ///
    /// Default implementation uses the source’s name and resolution.
    fn create_local_track(&self) -> (LocalVideoTrack, NativeVideoSource) {
        let source = self.create_local_source();
        let track = LocalVideoTrack::create_video_track(
            self.name(),
            RtcVideoSource::Native(source.clone()),
        );
        (track, source)
    }

    /// Optionally customize publish options (like simulcast or codecs).
    fn publish_options(&self) -> TrackPublishOptions {
        let mut opts = TrackPublishOptions::default();
        if let Some(codec) = self.preferred_codec() {
            opts.video_codec = codec;
        }
        opts
    }
}

pub(crate) async fn create_local_track<V: VideoSource + 'static>(
    mut camera: V,
    handle: Handle,
) -> Result<(LocalVideoTrack, JoinHandle<()>), VideoError> {
    let (track, source) = camera.create_local_track();
    let join = handle.spawn(async move {
        println!("before start");
        camera.start().await;
        println!("after start");
        loop {
            let frame = camera.next_frame().await.unwrap();
            /*if task::yield_now().await.is_err() {
                camera.stop().await;
                println!("Task cancelled!");
                break;
            }*/

            source.capture_frame(&frame);
        }
    });
    Ok((track, join))
}
