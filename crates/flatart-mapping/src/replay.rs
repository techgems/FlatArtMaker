//! Replays the edit log's active operations on top of the automatic mapping.

use crate::auto::Auto;
use crate::rematch;
use flatart_domain::{
    ColourName, DetectedPalette, DropReason, DroppedEdit, EditLog, EditOp, MAX_OUTPUT_COLOURS,
    OutputColourId, Paint, Rgb,
};

/// One output colour while replaying. Unnamed colours get an automatic
/// name at the end.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Output {
    pub id: OutputColourId,
    pub display: Rgb,
    pub name: Option<ColourName>,
}

/// The mapping after replay, before names and the default base are filled in.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Replayed {
    /// In slot order.
    pub outputs: Vec<Output>,
    /// The output for each detected colour.
    pub targets: Vec<OutputColourId>,
    pub paints: Vec<Paint>,
    /// Set by `SetBase`; `None` means use the automatic choice.
    pub base: Option<OutputColourId>,
    pub dropped: Vec<DroppedEdit>,
}

/// What the operations are checked against.
pub(crate) struct Context<'a> {
    pub auto: &'a Auto,
    pub detected: &'a DetectedPalette,
    /// The source image size, for paint points.
    pub width: u32,
    pub height: u32,
}

pub(crate) fn replay(cx: &Context<'_>, edits: &EditLog) -> Replayed {
    let mut state = Replayed {
        outputs: cx.auto.outputs.iter().map(auto_output).collect(),
        targets: cx.auto.targets.clone(),
        paints: Vec::new(),
        base: None,
        dropped: Vec::new(),
    };
    for (position, op) in edits.active().iter().enumerate() {
        if let Err(reason) = state.apply(cx, op) {
            state.dropped.push(DroppedEdit { position, reason });
        }
    }
    state
}

fn auto_output(auto: &crate::auto::AutoOutput) -> Output {
    Output {
        id: auto.id,
        display: auto.display,
        name: None,
    }
}

type Applied = std::result::Result<(), DropReason>;

impl Replayed {
    fn has(&self, id: OutputColourId) -> bool {
        self.outputs.iter().any(|o| o.id == id)
    }

    fn require(&self, id: OutputColourId) -> Applied {
        match self.has(id) {
            true => Ok(()),
            false => Err(DropReason::UnknownOutputColour),
        }
    }

    fn output_mut(&mut self, id: OutputColourId) -> std::result::Result<&mut Output, DropReason> {
        self.outputs
            .iter_mut()
            .find(|o| o.id == id)
            .ok_or(DropReason::UnknownOutputColour)
    }

    fn apply(&mut self, cx: &Context<'_>, op: &EditOp) -> Applied {
        match op {
            EditOp::Map { detected, output } => {
                let index = rematch::find(cx.detected, *detected)
                    .ok_or(DropReason::NoMatchingDetectedColour)?;
                self.require(*output)?;
                self.targets[index] = *output;
            }
            EditOp::PaintRegion(paint) => {
                self.require(paint.colour)?;
                let (x, y) = (paint.at.x.0, paint.at.y.0);
                let inside = (0.0..f64::from(cx.width)).contains(&x)
                    && (0.0..f64::from(cx.height)).contains(&y);
                if !inside {
                    return Err(DropReason::OutsideImage);
                }
                self.paints.push(*paint);
            }
            EditOp::AddOutput { id, display } => {
                if self.outputs.len() >= MAX_OUTPUT_COLOURS {
                    return Err(DropReason::TooManyOutputColours);
                }
                if *id < OutputColourId::FIRST_ADDED || self.has(*id) {
                    return Err(DropReason::OutputIdUnavailable);
                }
                self.outputs.push(Output {
                    id: *id,
                    display: *display,
                    name: None,
                });
            }
            EditOp::ChangeOutputColour { id, display } => {
                self.output_mut(*id)?.display = *display;
            }
            EditOp::DeleteOutput { id, reassign_to } => self.delete(*id, *reassign_to)?,
            EditOp::Rename { id, name } => {
                let taken = self.outputs.iter().any(|o| {
                    o.id != *id
                        && o.name
                            .as_ref()
                            .is_some_and(|n| n.as_str().eq_ignore_ascii_case(name.as_str()))
                });
                if taken {
                    self.require(*id)?;
                    return Err(DropReason::NameTaken);
                }
                self.output_mut(*id)?.name = Some(name.clone());
            }
            EditOp::SetBase { id } => {
                if let Some(id) = id {
                    self.require(*id)?;
                }
                self.base = *id;
            }
            EditOp::Reorder { order } => self.reorder(order)?,
            // Detection already used the pin as a fixed centre.
            EditOp::PinDetected { .. } => {}
            EditOp::ResetMapping { clear_paint } => self.reset(cx.auto, *clear_paint)?,
        }
        Ok(())
    }

    fn delete(&mut self, id: OutputColourId, reassign_to: OutputColourId) -> Applied {
        if id == reassign_to {
            return Err(DropReason::ReassignedToItself);
        }
        self.require(id)?;
        self.require(reassign_to)?;
        let moved = |t: &mut OutputColourId| {
            if *t == id {
                *t = reassign_to;
            }
        };
        self.targets.iter_mut().for_each(moved);
        self.paints.iter_mut().for_each(|p| moved(&mut p.colour));
        if let Some(base) = self.base.as_mut() {
            moved(base);
        }
        self.outputs.retain(|o| o.id != id);
        Ok(())
    }

    /// Listed colours first, in the given order; any not listed keep their
    /// relative order after them. Unknown IDs are ignored.
    fn reorder(&mut self, order: &[OutputColourId]) -> Applied {
        if order
            .iter()
            .enumerate()
            .any(|(i, id)| order[..i].contains(id))
        {
            return Err(DropReason::RepeatedInOrder);
        }
        let rank = |o: &Output| {
            order
                .iter()
                .position(|&id| id == o.id)
                .unwrap_or(order.len())
        };
        self.outputs.sort_by_key(rank); // stable
        Ok(())
    }

    /// Back to the automatic mapping. Automatic colours that were deleted
    /// come back at the end; everything else about the colours is kept.
    fn reset(&mut self, auto: &Auto, clear_paint: bool) -> Applied {
        let missing: Vec<Output> = auto
            .outputs
            .iter()
            .filter(|a| !self.has(a.id))
            .map(auto_output)
            .collect();
        if self.outputs.len() + missing.len() > MAX_OUTPUT_COLOURS {
            return Err(DropReason::TooManyOutputColours);
        }
        self.outputs.extend(missing);
        self.targets.clone_from(&auto.targets);
        if clear_paint {
            self.paints.clear();
        }
        Ok(())
    }
}
