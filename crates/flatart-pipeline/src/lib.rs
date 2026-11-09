//! The pipeline: runs each stage through its port and caches the result.
//!
//! It receives adapters as `Box<dyn Trait>` from a wiring crate and never
//! names a concrete adapter.

use flatart_domain::EngineInfo;

/// Describes the engine build.
pub fn engine_info() -> EngineInfo {
    EngineInfo {
        version: env!("CARGO_PKG_VERSION"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_crate_version() {
        assert_eq!(engine_info().version, env!("CARGO_PKG_VERSION"));
    }
}
