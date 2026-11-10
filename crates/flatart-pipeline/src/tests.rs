//! Pipeline tests with fake ports that count their calls.

use super::*;
use flatart_domain::settings::{
    CleanUpSettings, DetectedColourLimit, FitSettings, LayerHeight, LoadSettings, MappingSettings,
    MeshSettings, PrintScale, PrintabilitySettings, QuantSettings, TraceSettings, TraceUpscale,
};
use flatart_domain::{
    ArtBounds, BorderMap, ColourMapping, CubicBezier, DetectedColour, DetectedPalette, Detection,
    EditLog, EditOp, Error, ExportOptions, Label, LabelMap, LoadedImage, MappedColours, MeshModel,
    Mm, MmPerPx, Oklab, Paint, PolygonWithHoles, Printable, PxPoint, Result, RgbaImage, Settings,
    Stage, VectorModel,
};
use flatart_ports::testkit::fixtures;
use flatart_ports::{
    BorderTracer, ColorQuantizer, ColourMapper, CurveFitter, Exporter, MeshBuilder, RasterStages,
    Triangulator,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Call counts per fake method, shared by every fake.
#[derive(Clone, Default)]
struct Calls(Rc<RefCell<HashMap<&'static str, usize>>>);

impl Calls {
    fn hit(&self, name: &'static str) {
        *self.0.borrow_mut().entry(name).or_default() += 1;
    }

    fn get(&self, name: &str) -> usize {
        self.0.borrow().get(name).copied().unwrap_or(0)
    }

    fn snapshot(&self) -> HashMap<&'static str, usize> {
        self.0.borrow().clone()
    }
}

struct FakeRaster(Calls);

impl RasterStages for FakeRaster {
    fn load(&self, image: &RgbaImage, _: &LoadSettings) -> Result<LoadedImage> {
        self.0.hit("load");
        Ok(LoadedImage {
            image: image.clone(),
            bounds: ArtBounds {
                x: 0,
                y: 0,
                width: image.width(),
                height: image.height(),
            },
        })
    }
    fn clean_up(&self, labels: &LabelMap, _: &CleanUpSettings) -> Result<LabelMap> {
        self.0.hit("clean_up");
        Ok(labels.clone())
    }
    fn paint(&self, labels: &LabelMap, _: &[Paint]) -> Result<LabelMap> {
        self.0.hit("paint");
        Ok(labels.clone())
    }
    fn printability(&self, labels: &LabelMap, _: &PrintabilitySettings) -> Result<Printable> {
        self.0.hit("printability");
        Ok(Printable {
            labels: labels.clone(),
            thin_features: vec![],
        })
    }
}

struct FakeQuantizer(Calls);

impl ColorQuantizer for FakeQuantizer {
    fn quantize(&self, image: &RgbaImage, _: &QuantSettings, _: &[Oklab]) -> Result<Detection> {
        self.0.hit("quantize");
        let labels = image
            .pixels()
            .iter()
            .map(|p| if p.a == 0 { Label::OUTSIDE } else { Label(0) })
            .collect();
        Ok(Detection {
            palette: DetectedPalette::new(vec![DetectedColour {
                colour: Oklab::new(0.5, 0.0, 0.0)?,
                pixels: 1,
            }])?,
            labels: LabelMap::new(image.width(), image.height(), TraceUpscale::new(1)?, labels)?,
        })
    }
}

struct FakeMapper(Calls);

impl ColourMapper for FakeMapper {
    fn map(
        &self,
        _: &DetectedPalette,
        labels: &LabelMap,
        _: &EditLog,
        _: &MappingSettings,
    ) -> Result<MappedColours> {
        self.0.hit("map");
        Ok(MappedColours {
            palette: fixtures::palette(),
            mapping: ColourMapping::default(),
            labels: labels.clone(),
            paints: vec![],
            dropped_edits: vec![],
        })
    }
}

struct FakeTracer(Calls);

impl BorderTracer for FakeTracer {
    fn trace(&self, _: &LabelMap, _: &TraceSettings, _: &dyn CurveFitter) -> Result<BorderMap> {
        self.0.hit("trace");
        Ok(fixtures::nested_border_map())
    }
}

struct FakeFitter;

impl CurveFitter for FakeFitter {
    fn fit(&self, points: &[PxPoint], _: &FitSettings) -> Result<Vec<CubicBezier>> {
        Ok(points
            .windows(2)
            .map(|w| CubicBezier::line(w[0], w[1]))
            .collect())
    }
}

struct FakeMesher(Calls);

impl MeshBuilder for FakeMesher {
    fn build(&self, _: &VectorModel, _: &MeshSettings, _: &dyn Triangulator) -> Result<MeshModel> {
        self.0.hit("mesh");
        Ok(fixtures::mesh_model())
    }
}

struct FakeTriangulator;

impl Triangulator for FakeTriangulator {
    fn triangulate(&self, _: &PolygonWithHoles) -> Result<Vec<[u32; 3]>> {
        Ok(vec![])
    }
}

/// Writes the colour height it saw (vector) or the part count (mesh).
struct FakeExporter(&'static str);

impl Exporter<VectorModel> for FakeExporter {
    fn id(&self) -> &str {
        self.0
    }
    fn file_extension(&self) -> &str {
        "txt"
    }
    fn mime_type(&self) -> &str {
        "text/plain"
    }
    fn export(&self, model: &VectorModel, _: &ExportOptions) -> Result<Vec<u8>> {
        Ok(model.extrusion.colour.get().0.to_string().into_bytes())
    }
}

impl Exporter<MeshModel> for FakeExporter {
    fn id(&self) -> &str {
        self.0
    }
    fn file_extension(&self) -> &str {
        "txt"
    }
    fn mime_type(&self) -> &str {
        "text/plain"
    }
    fn export(&self, model: &MeshModel, _: &ExportOptions) -> Result<Vec<u8>> {
        Ok(model.parts.len().to_string().into_bytes())
    }
}

fn settings() -> Settings {
    Settings::with_defaults(PrintScale::new(MmPerPx(0.5)).unwrap())
}

fn full_pipeline(calls: &Calls) -> Pipeline {
    let mut pipeline = Pipeline::builder()
        .raster_stages(Box::new(FakeRaster(calls.clone())))
        .color_quantizer(Box::new(FakeQuantizer(calls.clone())))
        .colour_mapper(Box::new(FakeMapper(calls.clone())))
        .border_tracer(Box::new(FakeTracer(calls.clone())))
        .curve_fitter(Box::new(FakeFitter))
        .mesh_builder(Box::new(FakeMesher(calls.clone())))
        .triangulator(Box::new(FakeTriangulator))
        .vector_exporter(Box::new(FakeExporter("vector")))
        .mesh_exporter(Box::new(FakeExporter("mesh")))
        .build()
        .unwrap();
    pipeline.set_image(fixtures::two_colour_image());
    pipeline.set_settings(settings());
    pipeline
}

#[test]
fn reports_crate_version() {
    assert_eq!(engine_info().version, env!("CARGO_PKG_VERSION"));
}

#[test]
fn missing_inputs_are_reported() {
    let mut pipeline = Pipeline::builder().build().unwrap();
    assert_eq!(
        pipeline.loaded().unwrap_err(),
        Error::MissingInput("settings")
    );
    pipeline.set_settings(settings());
    assert_eq!(pipeline.loaded().unwrap_err(), Error::MissingInput("image"));
}

#[test]
fn stages_without_an_adapter_are_not_implemented() {
    let mut pipeline = Pipeline::builder().build().unwrap();
    pipeline.set_image(fixtures::two_colour_image());
    pipeline.set_settings(settings());
    assert_eq!(
        pipeline.mesh().unwrap_err(),
        Error::NotImplemented(Stage::Load)
    );
    assert_eq!(
        pipeline
            .export("svg", &ExportOptions::default())
            .unwrap_err(),
        Error::NotImplemented(Stage::Export)
    );

    let calls = Calls::default();
    let mut pipeline = Pipeline::builder()
        .raster_stages(Box::new(FakeRaster(calls.clone())))
        .build()
        .unwrap();
    pipeline.set_image(fixtures::two_colour_image());
    pipeline.set_settings(settings());
    assert!(pipeline.loaded().is_ok());
    assert_eq!(
        pipeline.mesh().unwrap_err(),
        Error::NotImplemented(Stage::DetectColours)
    );
}

#[test]
fn each_stage_runs_once_until_its_inputs_change() {
    let calls = Calls::default();
    let mut pipeline = full_pipeline(&calls);
    pipeline.mesh().unwrap();
    let first = calls.snapshot();
    for stage in [
        "load",
        "quantize",
        "map",
        "clean_up",
        "paint",
        "printability",
        "trace",
        "mesh",
    ] {
        assert_eq!(first.get(stage), Some(&1), "{stage} should run once");
    }
    pipeline.mesh().unwrap();
    pipeline.border_map().unwrap();
    assert_eq!(
        calls.snapshot(),
        first,
        "nothing changed, so nothing re-runs"
    );
}

#[test]
fn a_height_change_only_rebuilds_the_mesh() {
    let calls = Calls::default();
    let mut pipeline = full_pipeline(&calls);
    pipeline.mesh().unwrap();
    let mut changed = settings();
    changed.colour_height = LayerHeight::new(Mm(2.0)).unwrap();
    pipeline.set_settings(changed);
    pipeline.mesh().unwrap();
    assert_eq!(calls.get("trace"), 1);
    assert_eq!(calls.get("mesh"), 2);
    let bytes = pipeline
        .export("vector", &ExportOptions::default())
        .unwrap();
    assert_eq!(bytes, b"2");
}

#[test]
fn a_detection_change_reruns_detection_and_everything_after() {
    let calls = Calls::default();
    let mut pipeline = full_pipeline(&calls);
    pipeline.mesh().unwrap();
    let mut changed = settings();
    changed.advanced.detected_colours = DetectedColourLimit::new(12).unwrap();
    pipeline.set_settings(changed);
    pipeline.mesh().unwrap();
    assert_eq!(calls.get("load"), 1);
    for stage in ["quantize", "map", "clean_up", "paint", "trace", "mesh"] {
        assert_eq!(calls.get(stage), 2, "{stage} should re-run");
    }
}

#[test]
fn a_new_pin_reruns_detection() {
    let calls = Calls::default();
    let mut pipeline = full_pipeline(&calls);
    pipeline.detection().unwrap();
    let mut edits = EditLog::new();
    edits.push(EditOp::PinDetected {
        colour: Oklab::new(0.3, 0.1, 0.0).unwrap(),
    });
    pipeline.set_edits(edits);
    pipeline.detection().unwrap();
    assert_eq!(calls.get("quantize"), 2);
}

#[test]
fn exports_go_to_the_exporter_with_that_id() {
    let calls = Calls::default();
    let mut pipeline = full_pipeline(&calls);
    assert_eq!(pipeline.exporter_ids(), ["vector", "mesh"]);
    let options = ExportOptions::default();
    assert_eq!(pipeline.export("mesh", &options).unwrap(), b"2");
    assert_eq!(pipeline.export("vector", &options).unwrap(), b"1");
    assert_eq!(
        pipeline.export("step", &options).unwrap_err(),
        Error::UnknownExporter("step".into())
    );
}

#[test]
fn duplicate_exporter_ids_are_rejected() {
    let result = Pipeline::builder()
        .vector_exporter(Box::new(FakeExporter("same")))
        .mesh_exporter(Box::new(FakeExporter("same")))
        .build();
    assert!(result.is_err());
}
