//! The pipeline: runs each stage through its port and caches the result.
//!
//! It receives adapters as `Box<dyn Trait>` from a wiring crate, through
//! [`Pipeline::builder`], and never names a concrete adapter.

mod builder;
mod cache;
mod pipeline;
mod thumbnail;

pub use builder::PipelineBuilder;
pub use pipeline::Pipeline;

use flatart_domain::EngineInfo;

/// Describes the engine build.
pub fn engine_info() -> EngineInfo {
    EngineInfo {
        version: env!("CARGO_PKG_VERSION"),
    }
}

#[cfg(test)]
mod tests;
