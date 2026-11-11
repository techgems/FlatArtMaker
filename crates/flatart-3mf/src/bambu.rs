//! The `bambu` dialect (Bambu Studio and Orca-based slicers): one mesh object
//! per part, a parent object whose components are those parts, and
//! `Metadata/model_settings.config` giving each part's name and extruder.
//!
//! No `Application` tag: without a project config, it crashes the OrcaSlicer
//! command line.

use crate::model::{
    self, CORE_NS, MATERIAL_NS, OBJECT_NAME, colour_group, name_and_extruder, placement, triangle,
};
use crate::xml::Xml;
use flatart_domain::{MeshModel, PackageFile, Result};
use flatart_ports::{MODEL_PATH, SlicerDialect};

pub const CONFIG_PATH: &str = "Metadata/model_settings.config";

/// The middle of a 256 × 256 mm Bambu bed.
const BED_CENTRE: (f64, f64) = (128.0, 128.0);

const IDENTITY: &str = "1 0 0 0 1 0 0 0 1 0 0 0";

#[derive(Debug, Clone, Copy, Default)]
pub struct BambuDialect;

/// Part objects are 1…n, the parent n + 1, the colour group n + 2.
fn ids(model: &MeshModel) -> (String, String) {
    let n = model.parts.len();
    ((n + 1).to_string(), (n + 2).to_string())
}

fn model_file(model: &MeshModel) -> Result<Vec<u8>> {
    let (parent, colours) = ids(model);
    let mut xml = Xml::new()?;
    xml.open(
        "model",
        &[
            ("unit", "millimeter"),
            ("xml:lang", "en-US"),
            ("xmlns", CORE_NS),
            (
                "xmlns:BambuStudio",
                "http://schemas.bambulab.com/package/2021",
            ),
            (
                "xmlns:p",
                "http://schemas.microsoft.com/3dmanufacturing/production/2015/06",
            ),
            ("xmlns:m", MATERIAL_NS),
        ],
    )?;
    xml.text("metadata", &[("name", "BambuStudio:3mfVersion")], "1")?;
    xml.open("resources", &[])?;
    colour_group(&mut xml, model, &colours)?;
    for (i, part) in model.parts.iter().enumerate() {
        let (id, entry) = ((i + 1).to_string(), i.to_string());
        xml.open(
            "object",
            &[
                ("id", &id),
                ("type", "model"),
                ("pid", &colours),
                ("pindex", &entry),
            ],
        )?;
        xml.open("mesh", &[])?;
        model::vertices(&mut xml, std::iter::once(&part.mesh))?;
        xml.open("triangles", &[])?;
        for &t in &part.mesh.triangles {
            triangle(&mut xml, t, 0, None)?;
        }
        xml.close("triangles")?;
        xml.close("mesh")?;
        xml.close("object")?;
    }
    xml.open("object", &[("id", &parent), ("type", "model")])?;
    xml.open("components", &[])?;
    for i in 0..model.parts.len() {
        let id = (i + 1).to_string();
        xml.empty("component", &[("objectid", &id), ("transform", IDENTITY)])?;
    }
    xml.close("components")?;
    xml.close("object")?;
    xml.close("resources")?;
    xml.open("build", &[])?;
    xml.empty(
        "item",
        &[
            ("objectid", &parent),
            ("transform", &placement(model, BED_CENTRE)),
            ("printable", "1"),
        ],
    )?;
    xml.close("build")?;
    xml.close("model")?;
    Ok(xml.finish())
}

fn config_file(model: &MeshModel) -> Result<Vec<u8>> {
    let (parent, _) = ids(model);
    let mut xml = Xml::new()?;
    xml.open("config", &[])?;
    xml.open("object", &[("id", &parent)])?;
    xml.empty("metadata", &[("key", "name"), ("value", OBJECT_NAME)])?;
    xml.empty("metadata", &[("key", "extruder"), ("value", "1")])?;
    for (i, part) in model.parts.iter().enumerate() {
        let (name, extruder) = name_and_extruder(model, part)?;
        let id = (i + 1).to_string();
        xml.open("part", &[("id", &id), ("subtype", "normal_part")])?;
        xml.empty("metadata", &[("key", "name"), ("value", name)])?;
        xml.empty("metadata", &[("key", "extruder"), ("value", &extruder)])?;
        xml.close("part")?;
    }
    xml.close("object")?;
    xml.close("config")?;
    Ok(xml.finish())
}

impl SlicerDialect for BambuDialect {
    fn id(&self) -> &str {
        "bambu"
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
