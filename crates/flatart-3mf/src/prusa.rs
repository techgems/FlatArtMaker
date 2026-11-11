//! The `prusa` dialect: one object holding one mesh with every part's
//! triangles, and `Metadata/Slic3r_PE_model.config` giving each part's
//! triangle range, name and extruder.

use crate::model::{
    self, CORE_NS, MATERIAL_NS, OBJECT_NAME, colour_group, name_and_extruder, placement, triangle,
};
use crate::xml::Xml;
use flatart_domain::{MeshModel, PackageFile, Result};
use flatart_ports::{MODEL_PATH, SlicerDialect};

pub const CONFIG_PATH: &str = "Metadata/Slic3r_PE_model.config";

/// The middle of a 250 × 210 mm Prusa bed.
const BED_CENTRE: (f64, f64) = (125.0, 105.0);

const OBJECT_ID: &str = "1";
const COLOURS_ID: &str = "2";

#[derive(Debug, Clone, Copy, Default)]
pub struct PrusaDialect;

fn model_file(model: &MeshModel) -> Result<Vec<u8>> {
    let mut xml = Xml::new()?;
    xml.open(
        "model",
        &[
            ("unit", "millimeter"),
            ("xml:lang", "en-US"),
            ("xmlns", CORE_NS),
            ("xmlns:slic3rpe", "http://schemas.slic3r.org/3mf/2017/06"),
            ("xmlns:m", MATERIAL_NS),
        ],
    )?;
    // Without this, PrusaSlicer places the parts wrongly.
    xml.text("metadata", &[("name", "slic3rpe:Version3mf")], "1")?;
    xml.open("resources", &[])?;
    colour_group(&mut xml, model, COLOURS_ID)?;
    xml.open("object", &[("id", OBJECT_ID), ("type", "model")])?;
    xml.open("mesh", &[])?;
    model::vertices(&mut xml, model.parts.iter().map(|p| &p.mesh))?;
    xml.open("triangles", &[])?;
    let mut offset = 0u32;
    for (i, part) in model.parts.iter().enumerate() {
        let entry = i.to_string();
        for &t in &part.mesh.triangles {
            triangle(&mut xml, t, offset, Some((COLOURS_ID, &entry)))?;
        }
        offset += part.mesh.vertices.len() as u32;
    }
    xml.close("triangles")?;
    xml.close("mesh")?;
    xml.close("object")?;
    xml.close("resources")?;
    xml.open("build", &[])?;
    xml.empty(
        "item",
        &[
            ("objectid", OBJECT_ID),
            ("transform", &placement(model, BED_CENTRE)),
            ("printable", "1"),
        ],
    )?;
    xml.close("build")?;
    xml.close("model")?;
    Ok(xml.finish())
}

fn config_file(model: &MeshModel) -> Result<Vec<u8>> {
    let mut xml = Xml::new()?;
    xml.open("config", &[])?;
    xml.open("object", &[("id", OBJECT_ID), ("instances_count", "1")])?;
    xml.empty(
        "metadata",
        &[("type", "object"), ("key", "name"), ("value", OBJECT_NAME)],
    )?;
    let mut first = 0usize;
    for part in &model.parts {
        let count = part.mesh.triangles.len();
        if count == 0 {
            continue;
        }
        let (name, extruder) = name_and_extruder(model, part)?;
        let (from, to) = (first.to_string(), (first + count - 1).to_string());
        xml.open("volume", &[("firstid", &from), ("lastid", &to)])?;
        xml.empty(
            "metadata",
            &[("type", "volume"), ("key", "name"), ("value", name)],
        )?;
        xml.empty(
            "metadata",
            &[
                ("type", "volume"),
                ("key", "extruder"),
                ("value", &extruder),
            ],
        )?;
        xml.close("volume")?;
        first += count;
    }
    xml.close("object")?;
    xml.close("config")?;
    Ok(xml.finish())
}

impl SlicerDialect for PrusaDialect {
    fn id(&self) -> &str {
        "prusa"
    }

    fn parts(&self, model: &MeshModel) -> Result<Vec<PackageFile>> {
        Ok(vec![
            PackageFile {
                path: MODEL_PATH.into(),
                bytes: model_file(model)?,
            },
            PackageFile {
                path: CONFIG_PATH.into(),
                bytes: config_file(model)?,
            },
        ])
    }
}
