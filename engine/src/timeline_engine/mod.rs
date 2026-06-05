//! Timeline operations — pure functions over ProjectState timeline + media.

mod ops;
mod resolve;
mod edit;

pub use ops::{
    add_clip, compute_duration, move_clip, remove_clip, split_clip, trim_clip, AddClipParams,
};
pub use resolve::{resolve_frame, resolve_playback_path, ActiveLayer, FrameComposition, TrackKind};
pub use edit::{clip_at_playhead, find_clip_at, ClipHit};