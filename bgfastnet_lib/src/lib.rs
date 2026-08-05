//! FastNet protocol decoder for B&G Hydra / H2000 instruments
//!
//! This library decodes FastNet protocol data streams and converts them to
//! Signal K paths in SI units.
//!
//! # Quick Start
//!
//! ```rust
//! use bgfastnet_lib::{FrameBuffer, decode_frame};
//!
//! let mut fb = FrameBuffer::new_default();
//!
//! // Feed raw bytes from the FastNet bus
//! let data = vec![];
//! fb.add_to_buffer(&data);
//! fb.get_complete_frames();
//!
//! // Get decoded frames
//! while let Some(frame) = fb.frame_queue_pop() {
//!     for (path, value) in &frame.values {
//!         println!("{} = {}", path, value);
//!     }
//! }
//! ```

pub mod constants;
pub mod decode;
pub mod frame;
pub mod map;
pub mod utils;

pub use decode::{
    decode_ascii_frame, decode_frame, decode_light_frame, DecodedFrame, DecodedValue,
};
pub use frame::FrameBuffer;
pub use map::{channel_lookup, channel_map, project, unit_for, ChannelInfo};
pub use utils::calculate_checksum;
