/// A length in millimetres.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Mm(pub f64);

/// A length or coordinate in source-image pixels.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct Px(pub f64);
