//! Which parts there are, and how high each region goes in each.

use crate::flatten::flatten;
use crate::regions::{self, RingPoint};
use crate::solid::Part;
use crate::triangulate::triangulate;
use flatart_domain::settings::MeshSettings;
use flatart_domain::{Error, MeshModel, MeshPart, OutputColourId, PartKind, Result, VectorModel};
use flatart_ports::{MeshBuilder, Triangulator};

/// Extrudes each region of the border map to its height.
#[derive(Debug, Clone, Copy, Default)]
pub struct Extruder;

impl MeshBuilder for Extruder {
    fn build(
        &self,
        model: &VectorModel,
        settings: &MeshSettings,
        triangulator: &dyn Triangulator,
    ) -> Result<MeshModel> {
        let map = &model.borders;
        let palette = &model.palette;
        if let Some(face) = map.faces.iter().find(|f| palette.get(f.colour).is_none()) {
            return Err(Error::InvalidInput(format!(
                "a face has output colour {}, which isn't in the palette",
                face.colour.0
            )));
        }
        let outline = flatten(map, model.scale.get().0, settings.curve_tolerance.get().0);
        let regions = regions::build(map, &outline)?;
        let colour_of = |r: usize| (r < regions.faces).then(|| map.faces[r].colour);

        let base = model.extrusion.base.map(|b| b.get().0);
        let layer = model.extrusion.colour.get().0;
        let base_colour = base.and(palette.base());
        let fill = model.extrusion.fill_base_holes;

        // Which regions get filled in any part.
        let used = |r: usize| r < regions.faces || (base.is_some() && fill);
        let triangles: Vec<Vec<[RingPoint; 3]>> = regions
            .regions
            .iter()
            .enumerate()
            .map(|(r, region)| {
                if used(r) {
                    triangulate(&outline, region, triangulator)
                } else {
                    Ok(Vec::new())
                }
            })
            .collect::<Result<_>>()?;

        let part = |heights: &[Option<(f64, f64)>]| {
            Part {
                outline: &outline,
                regions: &regions,
                triangles: &triangles,
                heights,
            }
            .build()
        };
        let mut parts = Vec::new();

        if let (Some(b), false) = (base, regions.regions.is_empty()) {
            let colour = base_colour
                .ok_or_else(|| Error::InvalidInput("the palette has no base colour".into()))?;
            let heights: Vec<_> = (0..regions.regions.len())
                .map(|r| match colour_of(r) {
                    Some(c) if c == colour => Some((0.0, b + layer)),
                    Some(_) => Some((0.0, b)),
                    None => fill.then_some((0.0, b)),
                })
                .collect();
            parts.push(MeshPart {
                kind: PartKind::Base,
                colour,
                mesh: part(&heights)?,
            });
        }

        let floor = base.unwrap_or(0.0);
        for colour in palette.colours() {
            let id: OutputColourId = colour.id;
            if base_colour == Some(id) || !map.faces.iter().any(|f| f.colour == id) {
                continue;
            }
            let heights: Vec<_> = (0..regions.regions.len())
                .map(|r| (colour_of(r) == Some(id)).then_some((floor, floor + layer)))
                .collect();
            parts.push(MeshPart {
                kind: PartKind::Colour,
                colour: id,
                mesh: part(&heights)?,
            });
        }
        Ok(MeshModel {
            palette: palette.clone(),
            parts,
        })
    }
}
