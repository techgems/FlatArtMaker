//! What to build: the base and each colour, as slabs of areas.
//!
//! Like the 3MF: the base runs from 0 to B under the whole art (holes the
//! art encloses filled when "Fill holes in base" is on), and the base
//! colour's areas carry on up to B + C in the base's component. Every other
//! colour runs from B (or 0 with no base) up by C.
//!
//! The base colour's areas are bodies of their own on top of the slab, not
//! fused into it. Fused, the slab's top face touches itself wherever two of
//! those areas meet only at a corner, which a face boundary can't do.

use crate::loops::{loops, signed_area};
use crate::place::Placed;
use flatart_domain::{BASE_NAME, Error, Label, Result, VectorModel};

/// One slab: the areas with material between `lo` and `hi`, given per
/// border side as [left, right].
pub struct Slab {
    pub lo: f64,
    pub hi: f64,
    pub member: Vec<[bool; 2]>,
}

/// One component of the STEP file.
pub struct Part {
    /// The component's name, which also names its bodies.
    pub name: String,
    /// Its slabs; each area of each slab is one body.
    pub slabs: Vec<Slab>,
}

/// The parts, base first, then the colours in palette order. Colours with
/// no area are left out.
pub fn parts(model: &VectorModel, placed: &Placed) -> Result<Vec<Part>> {
    let map = &model.borders;
    let palette = &model.palette;
    if let Some(face) = map.faces.iter().find(|f| palette.get(f.colour).is_none()) {
        return Err(Error::InvalidInput(format!(
            "a face has output colour {}, which isn't in the palette",
            face.colour.0
        )));
    }
    let base = model.extrusion.base.map(|b| b.get().0);
    let layer = model.extrusion.colour.get().0;
    let base_colour = base.and(palette.base());
    let side = |pick: &dyn Fn(usize, Label) -> bool| -> Vec<[bool; 2]> {
        map.borders
            .iter()
            .enumerate()
            .map(|(b, border)| [pick(b, border.left), pick(b, border.right)])
            .collect()
    };
    let mut parts = Vec::new();

    if let (Some(b), false) = (base, map.faces.is_empty()) {
        let colour = base_colour
            .ok_or_else(|| Error::InvalidInput("the palette has no base colour".into()))?;
        // The art's holes: loops round the art that run clockwise.
        let art = side(&|_, label| !label.is_outside());
        let mut enclosed = vec![false; map.borders.len()];
        if model.extrusion.fill_base_holes {
            for lp in loops(placed, &art)? {
                if signed_area(placed, &lp) < 0.0 {
                    for step in lp {
                        enclosed[step.border] = true;
                    }
                }
            }
        }
        let mut slabs = vec![Slab {
            lo: 0.0,
            hi: b,
            member: side(&|b, label| !label.is_outside() || enclosed[b]),
        }];
        if map.faces.iter().any(|f| f.colour == colour) {
            slabs.push(Slab {
                lo: b,
                hi: b + layer,
                member: side(&|_, label| label == Label::from(colour)),
            });
        }
        parts.push(Part {
            name: BASE_NAME.to_owned(),
            slabs,
        });
    }

    let floor = base.unwrap_or(0.0);
    for colour in palette.colours() {
        if base_colour == Some(colour.id) || !map.faces.iter().any(|f| f.colour == colour.id) {
            continue;
        }
        parts.push(Part {
            name: colour.name.as_str().to_owned(),
            slabs: vec![Slab {
                lo: floor,
                hi: floor + layer,
                member: side(&|_, label| label == Label::from(colour.id)),
            }],
        });
    }
    Ok(parts)
}
