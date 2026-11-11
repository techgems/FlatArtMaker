//! The two models the exporters take: vector (SVG, STEP) and mesh (3MF, 3D preview).

use crate::palette::BASE_NAME;
use crate::settings::{Extrusion, PrintScale};
use crate::{BorderMap, MmPoint3, OutputColourId, OutputPalette, RgbaImage, Slot};

/// Everything the vector exporters need: the traced art plus how to size and extrude it.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorModel {
    pub borders: BorderMap,
    pub palette: OutputPalette,
    pub scale: PrintScale,
    pub extrusion: Extrusion,
}

/// A triangle mesh in model space. Triangles run anticlockwise seen from outside.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Mesh {
    pub vertices: Vec<MmPoint3>,
    pub triangles: Vec<[u32; 3]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PartKind {
    /// The base, from z = 0 to B. Its colour is the palette's base colour.
    Base,
    /// One output colour's layer.
    Colour,
}

/// One printable part.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshPart {
    pub kind: PartKind,
    pub colour: OutputColourId,
    pub mesh: Mesh,
}

/// The parts plus the palette that names them and gives their filament slots.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshModel {
    pub palette: OutputPalette,
    pub parts: Vec<MeshPart>,
}

impl MeshModel {
    /// `Base` for the base part, otherwise the colour's name.
    pub fn part_name(&self, part: &MeshPart) -> Option<&str> {
        match part.kind {
            PartKind::Base => Some(BASE_NAME),
            PartKind::Colour => self.palette.get(part.colour).map(|c| c.name.as_str()),
        }
    }

    /// The filament slot, which is also the extruder number. Only colours
    /// with a part take a slot, so the slots have no gaps.
    pub fn slot(&self, part: &MeshPart) -> Option<Slot> {
        self.palette
            .slot(part.colour, |id| self.parts.iter().any(|p| p.colour == id))
    }
}

/// Export settings that aren't part of the model.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExportOptions {
    /// The object or root part name, e.g. the project name.
    pub name: String,
    /// A top-down picture, for formats that embed one.
    pub thumbnail: Option<RgbaImage>,
}

/// One file inside a package such as a 3MF.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageFile {
    /// Relative, with `/` separators and no leading `/`, e.g. `3D/3dmodel.model`.
    pub path: String,
    pub bytes: Vec<u8>,
}
