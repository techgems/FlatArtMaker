//! The adapter.

use crate::write::{Component, write};
use crate::{brep, parts, place};
use flatart_domain::{ExportOptions, Result, VectorModel};
use flatart_ports::Exporter;

/// Writes the model as a STEP (AP214) file: a root part holding one
/// component per colour (plus `Base`, which also holds the base colour's
/// areas), each holding one solid per area.
#[derive(Debug, Clone, Copy, Default)]
pub struct StepExporter;

impl Exporter<VectorModel> for StepExporter {
    fn id(&self) -> &str {
        "step"
    }

    fn file_extension(&self) -> &str {
        "step"
    }

    fn mime_type(&self) -> &str {
        "model/step"
    }

    fn export(&self, model: &VectorModel, options: &ExportOptions) -> Result<Vec<u8>> {
        let placed = place::place(&model.borders, model.scale.get().0);
        let mut components = Vec::new();
        for part in parts::parts(model, &placed)? {
            let mut bodies = Vec::new();
            for slab in &part.slabs {
                bodies.extend(brep::solids(&placed, slab)?);
            }
            // Largest first; ties top to bottom, then left to right.
            bodies.sort_by(|a, b| {
                b.area
                    .total_cmp(&a.area)
                    .then(b.top.total_cmp(&a.top))
                    .then(a.left.total_cmp(&b.left))
            });
            components.push(Component {
                name: part.name,
                bodies,
            });
        }
        Ok(write(&options.name, &components)?.into_bytes())
    }
}
