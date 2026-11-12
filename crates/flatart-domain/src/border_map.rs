//! The shared-border map (SPEC "Core data model"), in image space.

use crate::{CubicBezier, Label, OutputColourId, Px, PxPoint};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[serde(transparent)]
pub struct JunctionId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[serde(transparent)]
pub struct BorderId(pub u32);

/// Where 3 or more areas meet, or 2 areas meet the image edge.
#[derive(Debug, Clone, Copy, PartialEq, Hash, Serialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct Junction {
    pub at: PxPoint,
}

/// How a border ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum BorderEnds {
    /// Runs from one junction to another (possibly the same one).
    Open { start: JunctionId, end: JunctionId },
    /// A closed loop with no junction, starting at an arbitrary point.
    Closed,
}

/// The curve between two areas. Both areas use this same curve.
#[derive(Debug, Clone, PartialEq, Hash, Serialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct Border {
    pub ends: BorderEnds,
    /// Joined end to end. Never empty.
    pub curve: Vec<CubicBezier>,
    /// The area on the left when walking from start to end, as seen on screen
    /// (y down). May be `Label::OUTSIDE`.
    pub left: Label,
    /// The area on the right. Always different from `left`.
    pub right: Label,
}

impl Border {
    pub fn start(&self) -> Option<PxPoint> {
        self.curve.first().map(|c| c.from)
    }

    pub fn end(&self) -> Option<PxPoint> {
        self.curve.last().map(|c| c.to)
    }
}

/// One border in a loop, possibly walked backwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct LoopStep {
    pub border: BorderId,
    pub reversed: bool,
}

/// A closed chain of borders: each step ends where the next one starts.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct Loop {
    pub steps: Vec<LoopStep>,
}

/// One connected area of one colour: an outer loop plus any holes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct Face {
    pub colour: OutputColourId,
    pub outer: Loop,
    pub holes: Vec<Loop>,
}

/// Junctions, borders and faces, all in source-image pixels.
#[derive(Debug, Clone, PartialEq, Hash, Default, Serialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct BorderMap {
    /// The source image's size.
    pub width: Px,
    pub height: Px,
    pub junctions: Vec<Junction>,
    pub borders: Vec<Border>,
    pub faces: Vec<Face>,
}

impl BorderMap {
    pub fn border(&self, id: BorderId) -> Option<&Border> {
        self.borders.get(id.0 as usize)
    }

    pub fn junction(&self, id: JunctionId) -> Option<&Junction> {
        self.junctions.get(id.0 as usize)
    }
}
