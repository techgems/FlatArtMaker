//! Golden tests: Jolteon through the whole wired pipeline.
//!
//! Each case's expected outputs are recorded in `fixtures/jolteon/expected/`
//! as a summary: each file's size and FNV-1a hash (so any change in the
//! bytes shows), plus each part's name, slot, triangle count, volume and
//! height. When a case differs, the actual files are written to
//! `target/golden/` to inspect.
//!
//! After a deliberate change, re-record with
//! `UPDATE_GOLDEN=1 cargo test -p flatart-cli --test golden`.

use flatart_cli::{Job, Pipeline, read_png};
use flatart_domain::{EditLog, ExportOptions, MeshModel, PartKind};
use flatart_ports::testkit::mesh::{assert_model_watertight, signed_volume};
use std::fmt::Write;
use std::path::PathBuf;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/jolteon");
const EXPORTERS: [(&str, &str); 3] = [
    ("3mf-prusa", "prusa.3mf"),
    ("3mf-bambu", "bambu.3mf"),
    ("svg", "svg"),
];

fn jolteon() -> Job {
    let bytes =
        std::fs::read(format!("{FIXTURES}/jolteon.png")).expect("fixtures/jolteon/jolteon.png");
    Job::new(read_png(&bytes).unwrap())
}

/// 64-bit FNV-1a: small, stable and good enough to notice any change.
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

fn z_range(model: &MeshModel, part: usize) -> (f64, f64) {
    model.parts[part]
        .mesh
        .vertices
        .iter()
        .fold((f64::MAX, f64::MIN), |(lo, hi), v| {
            (lo.min(v.z.0), hi.max(v.z.0))
        })
}

/// Exports every file and summarises them and the mesh.
fn summarise(pipeline: &mut Pipeline) -> (String, Vec<(String, Vec<u8>)>) {
    let options = ExportOptions {
        name: "jolteon".into(),
        thumbnail: Some(pipeline.thumbnail(256).unwrap()),
    };
    let mut summary = String::new();
    let mut files = Vec::new();
    for (id, extension) in EXPORTERS {
        let bytes = pipeline.export(id, &options).unwrap();
        let _ = writeln!(
            summary,
            "{id:<10} {:>9} bytes  fnv1a {:016x}",
            bytes.len(),
            fnv1a(&bytes)
        );
        files.push((extension.to_owned(), bytes));
    }
    let model = pipeline.mesh().unwrap().clone();
    for (i, part) in model.parts.iter().enumerate() {
        let (lo, hi) = z_range(&model, i);
        let _ = writeln!(
            summary,
            "{:<6} {:<14} slot {:>2}  {:>7} triangles  {:>12.3} mm³  z {lo:.2}–{hi:.2}",
            match part.kind {
                PartKind::Base => "base",
                PartKind::Colour => "colour",
            },
            model.part_name(part).unwrap(),
            model.slot(part).unwrap().get(),
            part.mesh.triangles.len(),
            signed_volume(&part.mesh),
        );
    }
    (summary, files)
}

/// Compares `summary` with the recorded one, or records it.
fn check(case: &str, summary: &str, files: &[(String, Vec<u8>)]) {
    let expected_path = PathBuf::from(format!("{FIXTURES}/expected/{case}.txt"));
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::create_dir_all(expected_path.parent().unwrap()).unwrap();
        std::fs::write(&expected_path, summary).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&expected_path).unwrap_or_default();
    if expected.replace("\r\n", "\n") != summary {
        let out = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../target/golden"));
        std::fs::create_dir_all(&out).unwrap();
        for (extension, bytes) in files {
            std::fs::write(out.join(format!("{case}.{extension}")), bytes).unwrap();
        }
        panic!(
            "{case}: outputs differ from {}.\nExpected:\n{expected}\nActual:\n{summary}\n\
             The actual files are in {}. If the change is intended, re-record with \
             UPDATE_GOLDEN=1 cargo test -p flatart-cli --test golden",
            expected_path.display(),
            out.display()
        );
    }
}

#[test]
fn jolteon_with_the_defaults() {
    let mut pipeline = jolteon().pipeline().unwrap();
    let model = pipeline.mesh().unwrap().clone();
    assert_model_watertight(&model);
    assert_eq!(model.parts[0].kind, PartKind::Base);
    let (summary, files) = summarise(&mut pipeline);
    check("defaults", &summary, &files);
}

/// The area the parts cover, from their volumes: (volume / height).
fn area(model: &MeshModel, part: usize) -> f64 {
    let (lo, hi) = z_range(model, part);
    signed_volume(&model.parts[part].mesh) / (hi - lo)
}

#[test]
fn jolteon_with_no_base() {
    let flat = Job {
        base: 0.0,
        ..jolteon()
    };
    let mut pipeline = flat.pipeline().unwrap();
    let model = pipeline.mesh().unwrap().clone();
    assert_model_watertight(&model);
    assert!(
        model.parts.iter().all(|p| p.kind == PartKind::Colour),
        "B = 0 must have no base part"
    );
    for i in 0..model.parts.len() {
        let (lo, hi) = z_range(&model, i);
        assert_eq!(
            (lo, hi),
            (0.0, flat.layer),
            "part {i} must run from z = 0 to C"
        );
    }

    // No gaps: the colours together cover exactly the footprint the base
    // covers with a base (its holes left open).
    let mut based = jolteon().pipeline().unwrap();
    let mut settings = based.settings().unwrap().clone();
    settings.advanced.fill_base_holes = false;
    based.set_settings(settings);
    let with_base = based.mesh().unwrap().clone();
    let base_colour = with_base.parts[0].colour;
    let b = jolteon().base;
    let base_colour_area = model
        .parts
        .iter()
        .position(|p| p.colour == base_colour)
        .map_or(0.0, |i| area(&model, i));
    let footprint = (signed_volume(&with_base.parts[0].mesh) - base_colour_area * flat.layer) / b;
    let covered: f64 = (0..model.parts.len()).map(|i| area(&model, i)).sum();
    assert!(
        (covered - footprint).abs() <= 1e-6 * footprint,
        "the colours cover {covered} mm² but the footprint is {footprint} mm²"
    );

    let (summary, files) = summarise(&mut pipeline);
    check("no-base", &summary, &files);
}

#[test]
fn jolteon_with_a_recorded_edit_log() {
    let json =
        std::fs::read(format!("{FIXTURES}/edits.json")).expect("fixtures/jolteon/edits.json");
    let edits: EditLog = serde_json::from_slice(&json).unwrap();
    let job = Job { edits, ..jolteon() };
    let mut pipeline = job.pipeline().unwrap();
    assert!(pipeline.mapped().unwrap().dropped_edits.is_empty());
    let model = pipeline.mesh().unwrap().clone();
    assert_model_watertight(&model);
    let names: Vec<_> = model
        .parts
        .iter()
        .filter_map(|p| model.part_name(p))
        .collect();
    assert!(
        names.contains(&"Orange"),
        "mapping: orange gets its own part: {names:?}"
    );
    assert!(
        names.contains(&"Pink"),
        "painting: the tongue gets its own part: {names:?}"
    );
    let (summary, files) = summarise(&mut pipeline);
    check("edits", &summary, &files);
}
