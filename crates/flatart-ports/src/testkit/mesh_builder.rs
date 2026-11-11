//! Conformance suite for [`MeshBuilder`].
//!
//! Built on the nested fixture: a 4 × 4 px square of `Black` (the base
//! colour) around a 2 × 2 px `White` island, at 10 mm per pixel, with a
//! 2 mm base and a 1 mm colour layer.

use super::mesh::{assert_watertight, signed_volume};
use super::{fixtures, ok};
use crate::{MeshBuilder, Triangulator};
use flatart_domain::settings::{CurveTolerance, MeshSettings};
use flatart_domain::{
    Face, Label, Loop, LoopStep, Mesh, MeshModel, Mm, OutputColourId, PartKind, VectorModel,
};
use std::collections::HashSet;

/// Runs every check on `builder`, triangulating with `triangulator`.
pub fn run(builder: &dyn MeshBuilder, triangulator: &dyn Triangulator) {
    let settings = MeshSettings {
        curve_tolerance: CurveTolerance::new(Mm(0.02)).expect("valid tolerance"),
    };
    let build = |model: &VectorModel, case: &str| {
        let built = ok(builder.build(model, &settings, triangulator), case);
        for (i, part) in built.parts.iter().enumerate() {
            assert_watertight(&part.mesh, &format!("{case}: part {i}"));
        }
        let again = ok(builder.build(model, &settings, triangulator), case);
        assert!(built == again, "{case}: not deterministic");
        built
    };

    // With a base: the base, then White. Black is the base colour, so its
    // ring rises inside the base.
    let model = fixtures::vector_model();
    let built = build(&model, "with base");
    check_parts(
        &built,
        &[(PartKind::Base, 0), (PartKind::Colour, 1)],
        "with base",
    );
    check_footprint(&built, 40.0, "with base");
    check_heights(&built.parts[0].mesh, 0.0, 3.0, "with base: Base");
    check_heights(&built.parts[1].mesh, 2.0, 3.0, "with base: White");
    // 40 × 40 × 2 slab, plus the 12 px² ring 1 mm higher.
    check_volume(&built.parts[0].mesh, 3200.0 + 1200.0, "with base: Base");
    check_volume(&built.parts[1].mesh, 400.0, "with base: White");
    check_shared(&built.parts[1].mesh, &built.parts[0].mesh, "with base");

    // No base: one part per colour, from z = 0.
    let flat = VectorModel {
        extrusion: flatart_domain::settings::Extrusion {
            base: None,
            ..model.extrusion
        },
        ..model.clone()
    };
    let built = build(&flat, "no base");
    check_parts(
        &built,
        &[(PartKind::Colour, 0), (PartKind::Colour, 1)],
        "no base",
    );
    check_footprint(&built, 40.0, "no base");
    check_heights(&built.parts[0].mesh, 0.0, 1.0, "no base: Black");
    check_heights(&built.parts[1].mesh, 0.0, 1.0, "no base: White");
    check_volume(&built.parts[0].mesh, 1200.0, "no base: Black");
    check_shared(&built.parts[1].mesh, &built.parts[0].mesh, "no base");

    // A hole of outside in the middle: filled in the base only when asked.
    for (fill, volume) in [(true, 3600.0 + 800.0), (false, 3600.0)] {
        let case = format!("outside hole, fill {fill}");
        let holed = ring_model(&model, fill);
        let built = build(&holed, &case);
        check_parts(&built, &[(PartKind::Base, 0)], &case);
        check_volume(&built.parts[0].mesh, volume, &case);
    }
}

/// The nested fixture with the island made outside: a ring with a hole.
fn ring_model(model: &VectorModel, fill: bool) -> VectorModel {
    let mut borders = model.borders.clone();
    borders.borders[1].right = Label::OUTSIDE;
    borders.faces.truncate(1);
    let step = |reversed| LoopStep {
        border: flatart_domain::BorderId(1),
        reversed,
    };
    borders.faces[0] = Face {
        colour: OutputColourId(0),
        outer: borders.faces[0].outer.clone(),
        holes: vec![Loop {
            steps: vec![step(true)],
        }],
    };
    VectorModel {
        borders,
        extrusion: flatart_domain::settings::Extrusion {
            fill_base_holes: fill,
            ..model.extrusion
        },
        ..model.clone()
    }
}

fn check_parts(model: &MeshModel, expected: &[(PartKind, u16)], case: &str) {
    let got: Vec<_> = model.parts.iter().map(|p| (p.kind, p.colour.0)).collect();
    assert_eq!(got, expected, "{case}: wrong parts (kind, colour)");
}

fn xy_bounds<'a>(meshes: impl Iterator<Item = &'a Mesh>) -> (f64, f64, f64, f64) {
    let mut b = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for v in meshes.flat_map(|m| &m.vertices) {
        b = (
            b.0.min(v.x.0),
            b.1.min(v.y.0),
            b.2.max(v.x.0),
            b.3.max(v.y.0),
        );
    }
    b
}

fn check_footprint(model: &MeshModel, side: f64, case: &str) {
    let (x0, y0, x1, y1) = xy_bounds(model.parts.iter().map(|p| &p.mesh));
    for (what, got, want) in [
        ("min x", x0, 0.0),
        ("min y", y0, 0.0),
        ("max x", x1, side),
        ("max y", y1, side),
    ] {
        assert!(
            (got - want).abs() <= 0.01,
            "{case}: footprint {what} is {got}, expected {want}"
        );
    }
}

fn check_heights(mesh: &Mesh, low: f64, high: f64, case: &str) {
    let zs = mesh.vertices.iter().map(|v| v.z.0);
    let (min, max) = zs.fold((f64::MAX, f64::MIN), |(a, b), z| (a.min(z), b.max(z)));
    assert!(
        (min - low).abs() < 1e-9 && (max - high).abs() < 1e-9,
        "{case}: runs from z = {min} to {max}, expected {low} to {high}"
    );
}

fn check_volume(mesh: &Mesh, expected: f64, case: &str) {
    let volume = signed_volume(mesh);
    assert!(
        (volume - expected).abs() <= 1e-6 * expected,
        "{case}: volume {volume} mm³, expected {expected}"
    );
}

/// Every point of `part`'s outline appears, exactly, in `neighbour`.
fn check_shared(part: &Mesh, neighbour: &Mesh, case: &str) {
    let key = |x: f64, y: f64| (x.to_bits(), y.to_bits());
    let theirs: HashSet<_> = neighbour
        .vertices
        .iter()
        .map(|v| key(v.x.0, v.y.0))
        .collect();
    for v in &part.vertices {
        assert!(
            theirs.contains(&key(v.x.0, v.y.0)),
            "{case}: point ({}, {}) isn't shared exactly with the neighbouring part",
            v.x.0,
            v.y.0
        );
    }
}
