//! Trivial implementations that exercise the suites, so they compile, run and
//! accept correct output. Tracing and triangulation have no trivial correct
//! implementation, so their suites are checked against hand-made results.

use super::{
    border_tracer, color_quantizer, curve_fitter, exporter, fixtures, mesh, slicer_dialect,
    triangulator,
};
use crate::{
    BorderTracer, ColorQuantizer, CurveFitter, Exporter, MODEL_PATH, SlicerDialect, Triangulator,
};
use flatart_domain::settings::{FitSettings, QuantSettings};
use flatart_domain::{
    CubicBezier, DetectedColour, DetectedPalette, Detection, Error, ExportOptions, Label, LabelMap,
    MeshModel, Oklab, PackageFile, PxPoint, Result, RgbaImage, VectorModel,
};

/// Joins the points with straight segments: exact, so always within tolerance.
struct PolylineFitter;

impl CurveFitter for PolylineFitter {
    fn fit(&self, points: &[PxPoint], _: &FitSettings) -> Result<Vec<CubicBezier>> {
        Ok(points
            .windows(2)
            .map(|w| CubicBezier::line(w[0], w[1]))
            .collect())
    }
}

/// One palette entry per distinct RGB, after the pins. Not real OKLab.
struct ExactQuantizer;

impl ColorQuantizer for ExactQuantizer {
    fn quantize(
        &self,
        image: &RgbaImage,
        settings: &QuantSettings,
        pins: &[Oklab],
    ) -> Result<Detection> {
        let mut colours: Vec<DetectedColour> = pins
            .iter()
            .map(|&colour| DetectedColour { colour, pixels: 0 })
            .collect();
        let mut labels = Vec::with_capacity(image.pixels().len());
        for p in image.pixels() {
            if p.a == 0 {
                labels.push(Label::OUTSIDE);
                continue;
            }
            let lab = Oklab::new(
                f64::from(p.r) / 255.0,
                f64::from(p.g) / 255.0,
                f64::from(p.b) / 255.0,
            )?;
            let index = match colours.iter().position(|c| c.colour == lab) {
                Some(i) => i,
                None => {
                    colours.push(DetectedColour {
                        colour: lab,
                        pixels: 0,
                    });
                    colours.len() - 1
                }
            };
            colours[index].pixels += 1;
            labels.push(Label(index as u16));
        }
        if colours.len() > usize::from(settings.max_colours.get()) + pins.len() {
            return Err(Error::InvalidInput("too many colours".into()));
        }
        Ok(Detection {
            palette: DetectedPalette::new(colours)?,
            labels: LabelMap::new(
                image.width(),
                image.height(),
                flatart_domain::settings::TraceUpscale::new(1)?,
                labels,
            )?,
        })
    }
}

/// Writes the model's debug text.
struct DebugExporter;

impl<M: std::fmt::Debug> Exporter<M> for DebugExporter {
    fn id(&self) -> &str {
        "debug"
    }
    fn file_extension(&self) -> &str {
        "txt"
    }
    fn mime_type(&self) -> &str {
        "text/plain"
    }
    fn export(&self, model: &M, options: &ExportOptions) -> Result<Vec<u8>> {
        Ok(format!("{}: {model:?}", options.name).into_bytes())
    }
}

/// Lists each part's name and slot.
struct ListDialect;

impl SlicerDialect for ListDialect {
    fn id(&self) -> &str {
        "list"
    }
    fn parts(&self, model: &MeshModel) -> Result<Vec<PackageFile>> {
        let lines: Vec<String> = model
            .parts
            .iter()
            .map(|p| {
                let name = model.part_name(p).unwrap_or("?");
                let slot = model.slot(p).map_or(0, |s| s.get());
                format!("{name} {slot}")
            })
            .collect();
        Ok(vec![
            PackageFile {
                path: MODEL_PATH.into(),
                bytes: lines.join("\n").into_bytes(),
            },
            PackageFile {
                path: "Metadata/list.config".into(),
                bytes: b"list".to_vec(),
            },
        ])
    }
}

#[test]
fn curve_fitter_suite_accepts_an_exact_fitter() {
    curve_fitter::run(&PolylineFitter);
}

#[test]
fn color_quantizer_suite_accepts_an_exact_quantizer() {
    color_quantizer::run(&ExactQuantizer);
}

#[test]
fn exporter_suites_accept_a_deterministic_exporter() {
    exporter::run_vector(&DebugExporter);
    exporter::run_mesh(&DebugExporter);
}

#[test]
fn slicer_dialect_suite_accepts_a_minimal_dialect() {
    slicer_dialect::run(&ListDialect);
}

#[test]
fn border_tracer_checks_accept_the_hand_made_map() {
    border_tracer::check(
        &fixtures::nested_labels(),
        &fixtures::nested_border_map(),
        "hand-made",
    );
    // The suite itself compiles against the interface.
    let _: fn(&dyn BorderTracer, &dyn CurveFitter) = border_tracer::run;
}

#[test]
#[should_panic(expected = "4-connected areas")]
fn border_tracer_checks_reject_a_missing_face() {
    let mut map = fixtures::nested_border_map();
    // Drop the island's face and its border, so only the face count is wrong.
    map.faces.pop();
    map.faces[0].holes.clear();
    map.borders.pop();
    border_tracer::check(&fixtures::nested_labels(), &map, "missing face");
}

#[test]
fn triangulator_checks_accept_a_hand_made_triangulation() {
    let (_, square) = triangulator::cases().remove(0);
    triangulator::check(&square, &[[0, 1, 2], [0, 2, 3]], "hand-made square");
    let _: fn(&dyn Triangulator) = triangulator::run;
}

#[test]
#[should_panic(expected = "clockwise")]
fn triangulator_checks_reject_clockwise_triangles() {
    let (_, square) = triangulator::cases().remove(0);
    triangulator::check(&square, &[[0, 2, 1], [0, 3, 2]], "clockwise");
}

#[test]
#[should_panic(expected = "cover")]
fn triangulator_checks_reject_missing_coverage() {
    let (_, square) = triangulator::cases().remove(0);
    triangulator::check(&square, &[[0, 1, 2]], "half");
}

#[test]
fn fixture_meshes_are_watertight() {
    mesh::assert_model_watertight(&fixtures::mesh_model());
}

#[test]
#[should_panic(expected = "open")]
fn watertight_check_rejects_an_open_mesh() {
    let mut model = fixtures::mesh_model();
    model.parts[0].mesh.triangles.pop();
    mesh::assert_model_watertight(&model);
}

#[test]
fn fixture_vector_model_is_consistent() {
    let model: VectorModel = fixtures::vector_model();
    assert!(model.palette.base().is_some());
    border_tracer::check(&fixtures::nested_labels(), &model.borders, "vector model");
}

#[test]
fn upscaled_fixture_repeats_cells() {
    let map = fixtures::upscaled(&fixtures::diagonal_labels(), 2);
    assert_eq!((map.width(), map.height()), (4, 4));
    assert_eq!(map.get(1, 1), map.get(0, 0));
    assert_ne!(map.get(2, 0), map.get(1, 0));
}
