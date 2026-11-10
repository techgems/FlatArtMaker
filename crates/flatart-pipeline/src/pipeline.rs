//! The stage graph (SPEC "Pipeline", steps 1–9), computed lazily and cached.
//!
//! ```text
//! image ─1 load─ 2 detect ─3 map─ 4 clean up ─5 paint─ 6 printability ─7 trace─┐
//!          edit-log pins ┘   └ edit log, palette ──────────────────────────────┴─ 8 mesh ─ 9 export
//! ```
//!
//! Asking for any result runs only the stages whose inputs changed since
//! their last run.

use crate::PipelineBuilder;
use crate::builder::Ports;
use crate::cache::{Cache, key_of};
use flatart_domain::{
    BorderMap, Detection, EditLog, Error, ExportOptions, LabelMap, LoadedImage, MappedColours,
    MeshModel, Printable, Result, RgbaImage, Settings, Stage, VectorModel,
};

/// The adapter for `stage`, or `NotImplemented` when none is wired in.
fn port<T: ?Sized>(adapter: &Option<Box<T>>, stage: Stage) -> Result<&T> {
    adapter.as_deref().ok_or(Error::NotImplemented(stage))
}

struct Inputs {
    /// The image and its hash, computed once when it's set.
    image: Option<(u64, RgbaImage)>,
    settings: Option<Settings>,
    edits: EditLog,
}

struct Caches {
    loaded: Cache<LoadedImage>,
    detection: Cache<Detection>,
    mapped: Cache<MappedColours>,
    cleaned: Cache<LabelMap>,
    painted: Cache<LabelMap>,
    printable: Cache<Printable>,
    borders: Cache<BorderMap>,
    mesh: Cache<MeshModel>,
}

/// One project's pipeline. Set the inputs, then ask for any stage's result.
pub struct Pipeline {
    ports: Ports,
    inputs: Inputs,
    caches: Caches,
}

impl Pipeline {
    pub fn builder() -> PipelineBuilder {
        PipelineBuilder::default()
    }

    pub(crate) fn new(ports: Ports) -> Self {
        Self {
            ports,
            inputs: Inputs {
                image: None,
                settings: None,
                edits: EditLog::new(),
            },
            caches: Caches {
                loaded: Cache::new(Stage::Load),
                detection: Cache::new(Stage::DetectColours),
                mapped: Cache::new(Stage::MapColours),
                cleaned: Cache::new(Stage::CleanUp),
                painted: Cache::new(Stage::Paint),
                printable: Cache::new(Stage::Printability),
                borders: Cache::new(Stage::Trace),
                mesh: Cache::new(Stage::Mesh),
            },
        }
    }

    pub fn set_image(&mut self, image: RgbaImage) {
        self.inputs.image = Some((key_of(&image), image));
    }

    pub fn set_settings(&mut self, settings: Settings) {
        self.inputs.settings = Some(settings);
    }

    pub fn set_edits(&mut self, edits: EditLog) {
        self.inputs.edits = edits;
    }

    pub fn settings(&self) -> Option<&Settings> {
        self.inputs.settings.as_ref()
    }

    /// The ids of every wired exporter, vector exporters first.
    pub fn exporter_ids(&self) -> Vec<&str> {
        let vector = self.ports.vector_exporters.iter().map(|e| e.id());
        let mesh = self.ports.mesh_exporters.iter().map(|e| e.id());
        vector.chain(mesh).collect()
    }

    fn require_settings(&self) -> Result<&Settings> {
        self.inputs
            .settings
            .as_ref()
            .ok_or(Error::MissingInput("settings"))
    }

    /// Stage 1.
    pub fn loaded(&mut self) -> Result<&LoadedImage> {
        let key = self.ensure_loaded()?;
        self.caches.loaded.get(key)
    }

    /// Stage 2.
    pub fn detection(&mut self) -> Result<&Detection> {
        let key = self.ensure_detection()?;
        self.caches.detection.get(key)
    }

    /// Stage 3.
    pub fn mapped(&mut self) -> Result<&MappedColours> {
        let key = self.ensure_mapped()?;
        self.caches.mapped.get(key)
    }

    /// Stage 4.
    pub fn cleaned(&mut self) -> Result<&LabelMap> {
        let key = self.ensure_cleaned()?;
        self.caches.cleaned.get(key)
    }

    /// Stage 5.
    pub fn painted(&mut self) -> Result<&LabelMap> {
        let key = self.ensure_painted()?;
        self.caches.painted.get(key)
    }

    /// Stage 6.
    pub fn printable(&mut self) -> Result<&Printable> {
        let key = self.ensure_printable()?;
        self.caches.printable.get(key)
    }

    /// Stage 7.
    pub fn border_map(&mut self) -> Result<&BorderMap> {
        let key = self.ensure_borders()?;
        self.caches.borders.get(key)
    }

    /// The traced art with its palette, scale and heights. Assembled on each
    /// call, so the heights can change without re-tracing.
    pub fn vector_model(&mut self) -> Result<VectorModel> {
        let (borders, mapped) = (self.ensure_borders()?, self.ensure_mapped()?);
        let settings = self.require_settings()?;
        Ok(VectorModel {
            borders: self.caches.borders.get(borders)?.clone(),
            palette: self.caches.mapped.get(mapped)?.palette.clone(),
            scale: settings.scale,
            extrusion: settings.extrusion(),
        })
    }

    /// Stage 8.
    pub fn mesh(&mut self) -> Result<&MeshModel> {
        let key = self.ensure_mesh()?;
        self.caches.mesh.get(key)
    }

    /// Stage 9: the file from the exporter with this id. Not cached: exports
    /// run only on a download.
    pub fn export(&mut self, id: &str, options: &ExportOptions) -> Result<Vec<u8>> {
        if self.ports.vector_exporters.is_empty() && self.ports.mesh_exporters.is_empty() {
            return Err(Error::NotImplemented(Stage::Export));
        }
        if let Some(i) = self
            .ports
            .vector_exporters
            .iter()
            .position(|e| e.id() == id)
        {
            let model = self.vector_model()?;
            return self.ports.vector_exporters[i].export(&model, options);
        }
        if let Some(i) = self.ports.mesh_exporters.iter().position(|e| e.id() == id) {
            let key = self.ensure_mesh()?;
            let model = self.caches.mesh.get(key)?;
            return self.ports.mesh_exporters[i].export(model, options);
        }
        Err(Error::UnknownExporter(id.to_owned()))
    }

    // Each `ensure_*` brings its stage up to date and returns its cache key.

    fn ensure_loaded(&mut self) -> Result<u64> {
        let settings = self.require_settings()?.load;
        let (image_key, image) = self
            .inputs
            .image
            .as_ref()
            .ok_or(Error::MissingInput("image"))?;
        let key = key_of((Stage::Load, image_key, settings));
        let raster = port(&self.ports.raster, Stage::Load)?;
        self.caches
            .loaded
            .refresh(key, || raster.load(image, &settings))
    }

    fn ensure_detection(&mut self) -> Result<u64> {
        let up = self.ensure_loaded()?;
        let settings = self.require_settings()?.quant();
        let pins = self.inputs.edits.pins();
        let key = key_of((Stage::DetectColours, up, settings, &pins));
        let quantizer = port(&self.ports.quantizer, Stage::DetectColours)?;
        let loaded = self.caches.loaded.get(up)?;
        self.caches
            .detection
            .refresh(key, || quantizer.quantize(&loaded.image, &settings, &pins))
    }

    fn ensure_mapped(&mut self) -> Result<u64> {
        let up = self.ensure_detection()?;
        let settings = self.require_settings()?.mapping();
        let edits = &self.inputs.edits;
        let key = key_of((Stage::MapColours, up, settings, edits.active()));
        let mapper = port(&self.ports.mapper, Stage::MapColours)?;
        let detection = self.caches.detection.get(up)?;
        self.caches
            .mapped
            .refresh(key, || mapper.map(detection, edits, &settings))
    }

    // Stages 4 and 5 key on the content they use from stage 3 (the label map,
    // the paints), not on stage 3's key. So a paint only re-runs from stage 5,
    // and a rename or colour change doesn't re-run either (SPEC step 5).

    fn ensure_cleaned(&mut self) -> Result<u64> {
        let mapped = self.ensure_mapped()?;
        let settings = self.require_settings()?.clean_up();
        let raster = port(&self.ports.raster, Stage::CleanUp)?;
        let labels = &self.caches.mapped.get(mapped)?.labels;
        let key = key_of((Stage::CleanUp, labels, settings));
        self.caches
            .cleaned
            .refresh(key, || raster.clean_up(labels, &settings))
    }

    fn ensure_painted(&mut self) -> Result<u64> {
        let (mapped, up) = (self.ensure_mapped()?, self.ensure_cleaned()?);
        let raster = port(&self.ports.raster, Stage::Paint)?;
        let paints = &self.caches.mapped.get(mapped)?.paints;
        let key = key_of((Stage::Paint, up, paints));
        let cleaned = self.caches.cleaned.get(up)?;
        self.caches
            .painted
            .refresh(key, || raster.paint(cleaned, paints))
    }

    fn ensure_printable(&mut self) -> Result<u64> {
        let up = self.ensure_painted()?;
        let settings = self.require_settings()?.printability();
        let key = key_of((Stage::Printability, up, settings));
        let raster = port(&self.ports.raster, Stage::Printability)?;
        let painted = self.caches.painted.get(up)?;
        self.caches
            .printable
            .refresh(key, || raster.printability(painted, &settings))
    }

    fn ensure_borders(&mut self) -> Result<u64> {
        let up = self.ensure_printable()?;
        let settings = self.require_settings()?.trace();
        let key = key_of((Stage::Trace, up, settings));
        let tracer = port(&self.ports.tracer, Stage::Trace)?;
        let fitter = port(&self.ports.fitter, Stage::Trace)?;
        let printable = self.caches.printable.get(up)?;
        self.caches
            .borders
            .refresh(key, || tracer.trace(&printable.labels, &settings, fitter))
    }

    fn ensure_mesh(&mut self) -> Result<u64> {
        let (borders, mapped) = (self.ensure_borders()?, self.ensure_mapped()?);
        let settings = self.require_settings()?;
        let (scale, extrusion, mesh_settings) =
            (settings.scale, settings.extrusion(), settings.mesh());
        let mesher = port(&self.ports.mesher, Stage::Mesh)?;
        let triangulator = port(&self.ports.triangulator, Stage::Mesh)?;
        let border_map = self.caches.borders.get(borders)?;
        let palette = &self.caches.mapped.get(mapped)?.palette;
        let key = key_of((
            Stage::Mesh,
            borders,
            palette,
            scale,
            extrusion,
            mesh_settings,
        ));
        self.caches.mesh.refresh(key, || {
            let model = VectorModel {
                borders: border_map.clone(),
                palette: palette.clone(),
                scale,
                extrusion,
            };
            mesher.build(&model, &mesh_settings, triangulator)
        })
    }
}
