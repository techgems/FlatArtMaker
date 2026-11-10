use std::fmt;

/// The single error type every crate maps its own errors into.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    /// A constructor rejected a value, so the type can't hold it.
    #[error("invalid {what}: {reason}")]
    InvalidValue { what: &'static str, reason: String },
    /// A pipeline stage has no adapter wired in yet.
    #[error("{0} is not implemented yet")]
    NotImplemented(Stage),
    /// The pipeline was asked for a result before it had this input.
    #[error("missing input: {0}")]
    MissingInput(&'static str),
    /// An adapter failed while running a stage.
    #[error("{stage} failed: {message}")]
    StageFailed { stage: Stage, message: String },
    #[error("unknown exporter: {0}")]
    UnknownExporter(String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// The pipeline stages, numbered as in SPEC.md "Pipeline".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
    Load,
    DetectColours,
    MapColours,
    CleanUp,
    Paint,
    Printability,
    Trace,
    Mesh,
    Export,
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Load => "1. load",
            Self::DetectColours => "2. detect colours",
            Self::MapColours => "3. map colours",
            Self::CleanUp => "4. clean up",
            Self::Paint => "5. region painting",
            Self::Printability => "6. printability",
            Self::Trace => "7. trace borders",
            Self::Mesh => "8. build mesh",
            Self::Export => "9. export",
        };
        f.write_str(name)
    }
}
