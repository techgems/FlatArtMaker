//! The colour-mapping edit log (SPEC Pipeline steps 2–5). `flatart-mapping`
//! replays it.

use crate::{ColourName, Error, Oklab, OutputColourId, PxPoint, Result, Rgb};
use serde::{Deserialize, Serialize};

/// Paints the connected area under `at` with `colour`.
#[derive(Debug, Clone, Copy, PartialEq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct Paint {
    /// The clicked point, in source-image pixels.
    pub at: PxPoint,
    pub colour: OutputColourId,
}

/// One colour-mapping operation.
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum EditOp {
    /// Points a detected colour at an output colour.
    Map {
        detected: Oklab,
        output: OutputColourId,
    },
    PaintRegion(Paint),
    /// Adds an output colour, named automatically. `id` must be unused and at
    /// least `OutputColourId::FIRST_ADDED`: see [`EditLog::next_output_id`].
    AddOutput {
        id: OutputColourId,
        display: Rgb,
    },
    ChangeOutputColour {
        id: OutputColourId,
        display: Rgb,
    },
    /// Moves everything using `id` to `reassign_to`.
    DeleteOutput {
        id: OutputColourId,
        reassign_to: OutputColourId,
    },
    Rename {
        id: OutputColourId,
        name: ColourName,
    },
    /// `None` goes back to the automatic choice: the colour most common
    /// along the art's outer edge.
    SetBase {
        id: Option<OutputColourId>,
    },
    /// The new slot order.
    Reorder {
        order: Vec<OutputColourId>,
    },
    /// A colour from the eyedropper, used as a fixed centre in detection.
    PinDetected {
        colour: Oklab,
    },
    ResetMapping {
        clear_paint: bool,
    },
}

/// The ordered operations plus the undo position.
///
/// Operations before `position` are active; the rest can be redone.
#[derive(Debug, Clone, PartialEq, Hash, Default, Serialize, Deserialize)]
#[serde(try_from = "EditLogRepr", into = "EditLogRepr")]
pub struct EditLog {
    ops: Vec<EditOp>,
    position: usize,
}

impl EditLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fails when `position` is past the end.
    pub fn from_parts(ops: Vec<EditOp>, position: usize) -> Result<Self> {
        if position > ops.len() {
            return Err(Error::InvalidValue {
                what: "edit log",
                reason: format!("position {position} is past {} operations", ops.len()),
            });
        }
        Ok(Self { ops, position })
    }

    /// Adds an operation. Anything that could have been redone is discarded.
    pub fn push(&mut self, op: EditOp) {
        self.ops.truncate(self.position);
        self.ops.push(op);
        self.position = self.ops.len();
    }

    /// Returns `false` when there's nothing to undo.
    pub fn undo(&mut self) -> bool {
        let can = self.position > 0;
        if can {
            self.position -= 1;
        }
        can
    }

    /// Returns `false` when there's nothing to redo.
    pub fn redo(&mut self) -> bool {
        let can = self.position < self.ops.len();
        if can {
            self.position += 1;
        }
        can
    }

    /// The operations in effect, in order.
    pub fn active(&self) -> &[EditOp] {
        &self.ops[..self.position]
    }

    pub fn all(&self) -> &[EditOp] {
        &self.ops
    }

    pub fn position(&self) -> usize {
        self.position
    }

    /// An unused ID for a new `AddOutput`: one past the highest added so far,
    /// and never below `OutputColourId::FIRST_ADDED`.
    pub fn next_output_id(&self) -> OutputColourId {
        let highest = self
            .ops
            .iter()
            .filter_map(|op| match op {
                EditOp::AddOutput { id, .. } => Some(id.0.saturating_add(1)),
                _ => None,
            })
            .max()
            .unwrap_or(0);
        OutputColourId(highest.max(OutputColourId::FIRST_ADDED.0))
    }

    /// The active eyedropper pins, which detection needs before mapping runs.
    pub fn pins(&self) -> Vec<Oklab> {
        self.active()
            .iter()
            .filter_map(|op| match op {
                EditOp::PinDetected { colour } => Some(*colour),
                _ => None,
            })
            .collect()
    }
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[cfg_attr(feature = "tsify", tsify(rename = "EditLog"))]
struct EditLogRepr {
    ops: Vec<EditOp>,
    position: usize,
}

impl TryFrom<EditLogRepr> for EditLog {
    type Error = Error;
    fn try_from(repr: EditLogRepr) -> Result<Self> {
        Self::from_parts(repr.ops, repr.position)
    }
}

impl From<EditLog> for EditLogRepr {
    fn from(log: EditLog) -> Self {
        Self {
            ops: log.ops,
            position: log.position,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pin(l: f64) -> EditOp {
        EditOp::PinDetected {
            colour: Oklab::new(l, 0.0, 0.0).unwrap(),
        }
    }

    #[test]
    fn undo_and_redo_move_the_position() {
        let mut log = EditLog::new();
        assert!(!log.undo());
        log.push(pin(0.1));
        log.push(pin(0.2));
        assert!(log.undo());
        assert_eq!(log.active().len(), 1);
        assert!(log.redo());
        assert!(!log.redo());
        assert_eq!(log.active().len(), 2);
    }

    #[test]
    fn push_after_undo_drops_the_redo_tail() {
        let mut log = EditLog::new();
        log.push(pin(0.1));
        log.push(pin(0.2));
        log.undo();
        log.push(pin(0.3));
        assert_eq!(log.all().len(), 2);
        assert_eq!(log.all()[0], pin(0.1));
        assert_eq!(log.pins()[1].l(), 0.3);
    }

    #[test]
    fn pins_only_counts_active_operations() {
        let mut log = EditLog::new();
        log.push(pin(0.1));
        log.undo();
        assert!(log.pins().is_empty());
    }

    #[test]
    fn next_output_id_skips_automatic_ids_and_added_ones() {
        let mut log = EditLog::new();
        assert_eq!(log.next_output_id(), OutputColourId::FIRST_ADDED);
        let add = |id| EditOp::AddOutput {
            id: OutputColourId(id),
            display: Rgb { r: 0, g: 0, b: 0 },
        };
        log.push(add(15));
        log.push(add(13));
        assert_eq!(log.next_output_id(), OutputColourId(16));
    }

    #[test]
    fn position_past_the_end_is_rejected() {
        assert!(EditLog::from_parts(vec![pin(0.1)], 2).is_err());
        let json = r#"{"ops":[],"position":1}"#;
        assert!(serde_json::from_str::<EditLog>(json).is_err());
    }

    #[test]
    fn round_trips_through_json() {
        let mut log = EditLog::new();
        log.push(pin(0.1));
        log.push(EditOp::Rename {
            id: OutputColourId(1),
            name: ColourName::new("Black").unwrap(),
        });
        let json = serde_json::to_string(&log).unwrap();
        assert_eq!(serde_json::from_str::<EditLog>(&json).unwrap(), log);
    }
}
