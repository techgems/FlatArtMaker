//! Ties the passes together into a `ColourMapper`.

use crate::auto::auto_map;
use crate::base::edge_majority;
use crate::names::auto_name;
use crate::replay::{Context, Replayed, replay};
use flatart_domain::settings::MappingSettings;
use flatart_domain::{
    ColourMapping, ColourName, Detection, EditLog, Label, LabelMap, MappedColours, OutputColour,
    OutputPalette, Result,
};
use flatart_ports::ColourMapper;

/// Merges the most similar detected colours (pixel-weighted) until the
/// user's colour count is reached, then replays the edit log.
///
/// A `Map` matches the nearest current detected colour within
/// [`REMATCH_LIMIT`](crate::REMATCH_LIMIT).
#[derive(Debug, Clone, Copy, Default)]
pub struct MergeMapper;

impl ColourMapper for MergeMapper {
    fn map(
        &self,
        detection: &Detection,
        edits: &EditLog,
        settings: &MappingSettings,
    ) -> Result<MappedColours> {
        let auto = auto_map(&detection.palette, settings)?;
        let cx = Context {
            auto: &auto,
            detected: &detection.palette,
            width: detection.labels.width(),
            height: detection.labels.height(),
        };
        let replayed = replay(&cx, edits);
        let labels = output_labels(&detection.labels, &replayed)?;
        let palette = palette(&replayed, &labels)?;
        Ok(MappedColours {
            palette,
            mapping: ColourMapping {
                targets: replayed.targets,
            },
            labels,
            paints: replayed.paints,
            dropped_edits: replayed.dropped,
            similarity: auto.similarity,
        })
    }
}

/// Each detected label replaced by its output colour's ID.
fn output_labels(detected: &LabelMap, replayed: &Replayed) -> Result<LabelMap> {
    let cells = detected
        .labels()
        .iter()
        .map(|&d| match replayed.targets.get(usize::from(d.0)) {
            Some(&id) if !d.is_outside() => Label::from(id),
            _ => Label::OUTSIDE,
        })
        .collect();
    LabelMap::new(
        detected.width(),
        detected.height(),
        detected.upscale(),
        cells,
    )
}

/// Fills in automatic names (in ID order, so reordering doesn't swap them)
/// and the default base.
fn palette(replayed: &Replayed, labels: &LabelMap) -> Result<OutputPalette> {
    let outputs = &replayed.outputs;
    let mut names: Vec<Option<ColourName>> = outputs.iter().map(|o| o.name.clone()).collect();
    let mut taken: Vec<ColourName> = names.iter().flatten().cloned().collect();
    let mut by_id: Vec<usize> = (0..outputs.len()).collect();
    by_id.sort_by_key(|&i| outputs[i].id);
    for i in by_id {
        if names[i].is_none() {
            let name = auto_name(outputs[i].display, &taken)?;
            taken.push(name.clone());
            names[i] = Some(name);
        }
    }

    let colours = outputs
        .iter()
        .zip(names)
        .filter_map(|(o, name)| {
            Some(OutputColour {
                id: o.id,
                name: name?,
                display: o.display,
            })
        })
        .collect();
    let slots: Vec<_> = replayed.outputs.iter().map(|o| o.id).collect();
    let base = replayed.base.or_else(|| edge_majority(labels, &slots));
    OutputPalette::new(colours, base)
}
