//! Data types shared by every FlatArtMaker crate.
//!
//! Interface signatures in `flatart-ports` use only these types, so no
//! third-party type ever crosses a port boundary.

mod engine_info;
mod error;
mod units;

pub use engine_info::EngineInfo;
pub use error::Error;
pub use units::{Mm, Px};
