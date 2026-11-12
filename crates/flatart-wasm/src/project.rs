//! One project's pipeline, in plain Rust. [`crate::Engine`] wraps it for
//! JavaScript; keeping the logic here lets it be tested natively.

use crate::types::{LoadedImageInfo, MappedInfo, MeshPartInfo, Report, Slicer};
use crate::wiring;
use flatart_domain::settings::PrintScale;
use flatart_domain::{
    EditLog, Error, ExportOptions, Label, Mesh, Mm, MmPerPx, Result, Rgba, RgbaImage, Settings,
    VectorModel,
};
use flatart_pipeline::{Pipeline, THUMBNAIL_SIDE};

/// The default print size: the art's longest side (SPEC "X/Y size").
const DEFAULT_LONGEST_SIDE: Mm = Mm(100.0);

/// One open project. Its pipeline caches every stage, so a call re-runs only
/// what its inputs changed.
pub struct Project {
    pipeline: Pipeline,
}

impl Project {
    pub fn new() -> Result<Self> {
        Ok(Self {
            pipeline: wiring::pipeline()?,
        })
    }

    /// Starts the project over on a new image: no edits, and the SPEC
    /// defaults sized to [`DEFAULT_LONGEST_SIDE`]. `rgba` is 4 bytes per
    /// pixel, row by row, straight alpha.
    pub fn load_image(&mut self, rgba: &[u8], width: u32, height: u32) -> Result<LoadedImageInfo> {
        let (chunks, rest) = rgba.as_chunks::<4>();
        if !rest.is_empty() {
            return Err(Error::InvalidInput(format!(
                "{} bytes isn't a whole number of RGBA pixels",
                rgba.len()
            )));
        }
        let pixels = chunks
            .iter()
            .map(|&[r, g, b, a]| Rgba { r, g, b, a })
            .collect();
        self.pipeline
            .set_image(RgbaImage::new(width, height, pixels)?);
        self.pipeline.set_edits(EditLog::new());
        // The art's bounds need the load settings; any scale will do for them.
        self.pipeline
            .set_settings(Settings::with_defaults(PrintScale::new(MmPerPx(1.0))?));
        let bounds = self.pipeline.loaded()?.bounds;
        let scale =
            PrintScale::from_longest_side(DEFAULT_LONGEST_SIDE, bounds.width, bounds.height)?;
        let settings = Settings::with_defaults(scale);
        self.pipeline.set_settings(settings.clone());
        Ok(LoadedImageInfo { bounds, settings })
    }

    pub fn set_settings(&mut self, settings: Settings) {
        self.pipeline.set_settings(settings);
    }

    pub fn apply_edits(&mut self, log: EditLog) {
        self.pipeline.set_edits(log);
    }

    pub fn mapped(&mut self) -> Result<MappedInfo> {
        let detected = self.pipeline.detection()?.palette.clone();
        let mapped = self.pipeline.mapped()?;
        Ok(MappedInfo {
            width: mapped.labels.width(),
            height: mapped.labels.height(),
            outside: Label::OUTSIDE.0,
            palette: mapped.palette.clone(),
            detected,
            mapping: mapped.mapping.clone(),
            similarity: mapped.similarity.clone(),
        })
    }

    /// The output label map, one `u16` per cell: an `OutputColourId`, or
    /// [`MappedInfo::outside`].
    pub fn mapped_labels(&mut self) -> Result<Vec<u16>> {
        let labels = self.pipeline.mapped()?.labels.labels();
        Ok(labels.iter().map(|l| l.0).collect())
    }

    pub fn vector_model(&mut self) -> Result<VectorModel> {
        self.pipeline.vector_model()
    }

    pub fn report(&mut self) -> Result<Report> {
        let dropped_edits = self.pipeline.mapped()?.dropped_edits.clone();
        let thin_features = self.pipeline.printable()?.thin_features.clone();
        Ok(Report {
            thin_features,
            dropped_edits,
        })
    }

    pub fn mesh_parts(&mut self) -> Result<Vec<MeshPartInfo>> {
        let model = self.pipeline.mesh()?;
        Ok(model
            .parts
            .iter()
            .map(|part| MeshPartInfo {
                kind: part.kind,
                colour: part.colour,
                name: model.part_name(part).unwrap_or_default().to_owned(),
                display: model
                    .palette
                    .get(part.colour)
                    .map_or(flatart_domain::Rgb { r: 0, g: 0, b: 0 }, |c| c.display),
                slot: model.slot(part).map(|s| s.get()),
            })
            .collect())
    }

    fn part_mesh(&mut self, part: usize) -> Result<&Mesh> {
        let parts = &self.pipeline.mesh()?.parts;
        let count = parts.len();
        parts
            .get(part)
            .map(|p| &p.mesh)
            .ok_or_else(|| Error::InvalidInput(format!("no part {part}: there are {count}")))
    }

    /// The part's vertices as x, y, z triples in mm, z up.
    pub fn mesh_positions(&mut self, part: usize) -> Result<Vec<f32>> {
        let mesh = self.part_mesh(part)?;
        Ok(mesh
            .vertices
            .iter()
            .flat_map(|v| [v.x.0 as f32, v.y.0 as f32, v.z.0 as f32])
            .collect())
    }

    /// The part's triangles as vertex-index triples, anticlockwise seen from
    /// outside.
    pub fn mesh_indices(&mut self, part: usize) -> Result<Vec<u32>> {
        let mesh = self.part_mesh(part)?;
        Ok(mesh.triangles.iter().flatten().copied().collect())
    }

    /// The same file the CLI writes for the same image, settings and edits.
    pub fn export_3mf(&mut self, slicer: Slicer, name: &str) -> Result<Vec<u8>> {
        let options = ExportOptions {
            name: name.to_owned(),
            thumbnail: Some(self.pipeline.thumbnail(THUMBNAIL_SIDE)?),
        };
        self.pipeline.export(slicer.exporter(), &options)
    }

    pub fn export_svg(&mut self, name: &str) -> Result<Vec<u8>> {
        let options = ExportOptions {
            name: name.to_owned(),
            thumbnail: None,
        };
        self.pipeline.export("svg", &options)
    }
}
