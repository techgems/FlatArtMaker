/// Identifies the running engine build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineInfo {
    pub version: &'static str,
}
