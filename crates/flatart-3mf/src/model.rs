//! The pieces of `3D/3dmodel.model` both dialects share.

use crate::xml::{Xml, number};
use flatart_domain::{Error, Mesh, MeshModel, MeshPart, Result};

pub const CORE_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/core/2015/02";
pub const MATERIAL_NS: &str = "http://schemas.microsoft.com/3dmanufacturing/material/2015/02";

/// The object's name in the slicer's object list.
pub const OBJECT_NAME: &str = "FlatArt";

/// A part's name and extruder (its colour's slot), per the dialect contract.
pub fn name_and_extruder<'m>(model: &'m MeshModel, part: &MeshPart) -> Result<(&'m str, String)> {
    let name = model
        .part_name(part)
        .ok_or_else(|| Error::InvalidInput(format!("part colour {} has no name", part.colour.0)))?;
    let slot = model
        .slot(part)
        .ok_or_else(|| Error::InvalidInput(format!("part colour {} has no slot", part.colour.0)))?;
    Ok((name, slot.get().to_string()))
}

/// `#RRGGBBFF` for the part's display colour.
fn colour(model: &MeshModel, part: &MeshPart) -> String {
    let rgb = model
        .palette
        .get(part.colour)
        .map_or((128, 128, 128), |c| (c.display.r, c.display.g, c.display.b));
    format!("#{:02X}{:02X}{:02X}FF", rgb.0, rgb.1, rgb.2)
}

/// `<m:colorgroup>` with one colour per part, in part order.
pub fn colour_group(xml: &mut Xml, model: &MeshModel, id: &str) -> Result<()> {
    xml.open("m:colorgroup", &[("id", id)])?;
    for part in &model.parts {
        xml.empty("m:color", &[("color", &colour(model, part))])?;
    }
    xml.close("m:colorgroup")
}

/// `<vertices>` for `meshes`, one after another.
pub fn vertices<'m>(xml: &mut Xml, meshes: impl Iterator<Item = &'m Mesh>) -> Result<()> {
    xml.open("vertices", &[])?;
    for mesh in meshes {
        for v in &mesh.vertices {
            xml.empty(
                "vertex",
                &[
                    ("x", &number(v.x.0)),
                    ("y", &number(v.y.0)),
                    ("z", &number(v.z.0)),
                ],
            )?;
        }
    }
    xml.close("vertices")
}

/// One `<triangle>`, its indices moved by `offset`, with a colour when
/// `colour` names the group and the entry.
pub fn triangle(
    xml: &mut Xml,
    t: [u32; 3],
    offset: u32,
    colour: Option<(&str, &str)>,
) -> Result<()> {
    let [a, b, c] = t.map(|i| (i + offset).to_string());
    let mut attributes = vec![("v1", a.as_str()), ("v2", b.as_str()), ("v3", c.as_str())];
    if let Some((group, entry)) = colour {
        attributes.extend([("pid", group), ("p1", entry)]);
    }
    xml.empty("triangle", &attributes)
}

/// The build item's transform: the footprint centred on `centre` (mm).
pub fn placement(model: &MeshModel, centre: (f64, f64)) -> String {
    let (mut lo, mut hi) = ((f64::MAX, f64::MAX), (f64::MIN, f64::MIN));
    for v in model.parts.iter().flat_map(|p| &p.mesh.vertices) {
        lo = (lo.0.min(v.x.0), lo.1.min(v.y.0));
        hi = (hi.0.max(v.x.0), hi.1.max(v.y.0));
    }
    let (dx, dy) = if lo.0 <= hi.0 {
        (
            centre.0 - (lo.0 + hi.0) / 2.0,
            centre.1 - (lo.1 + hi.1) / 2.0,
        )
    } else {
        centre
    };
    format!("1 0 0 0 1 0 0 0 1 {} {} 0", number(dx), number(dy))
}
