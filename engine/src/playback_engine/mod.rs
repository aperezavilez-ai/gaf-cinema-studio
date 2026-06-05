//! Playback engine — state machine, frame pacing, scrub latency tracking.
//! Native decoders (AVFoundation/MediaCodec) attach via FrameComposition in Phase 2+.

mod engine;
mod metrics;

pub use engine::{PlaybackEngine, PlaybackMode, PlaybackStatus};
pub use metrics::PlaybackMetrics;
