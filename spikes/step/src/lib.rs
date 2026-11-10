//! Task 03 spike: named solid straight extrusions written as STEP with `truck`.
//!
//! Shapes (mm): a 40 x 40 base (z 0..3.4) and two "Black" islands (z 3.4..4.4).
//! The first island has a hole bounded by a closed loop of cubic Bezier curves.
//! Everything is a `tsweep` of a planar face along +Z.

use truck_assembly::assy::{Assembly, EdgeEntity, NodeEntity};
use truck_modeling::{builder, Edge, Face, Matrix4, Point3, Solid, Vector3, Vertex, Wire};
use truck_stepio::common::PartAttrs;
use truck_stepio::out::{StepDesign, StepDisplay, StepHeaderDescriptor, StepModel};
use truck_topology::compress::CompressedSolid;

pub const BASE_HEIGHT: f64 = 3.4;
pub const LAYER_HEIGHT: f64 = 1.0;

/// Names in output order. Index 0 is the base.
pub const NAMES: [&str; 3] = ["Base", "Black", "Black-1"];

/// Schema written by truck (hard-coded, not a real schema name) and the AP214 one we patch in.
pub const TRUCK_SCHEMA: &str = "FILE_SCHEMA(('ISO-10303-203'));";
pub const AP214_SCHEMA: &str = "FILE_SCHEMA(('AUTOMOTIVE_DESIGN { 1 0 10303 214 1 1 1 1 }'));";

type CSolid = CompressedSolid<Point3, truck_modeling::Curve, truck_modeling::Surface>;

fn polygon(points: &[(f64, f64)], z: f64) -> Wire {
    let v: Vec<Vertex> = points
        .iter()
        .map(|&(x, y)| builder::vertex(Point3::new(x, y, z)))
        .collect();
    (0..v.len())
        .map(|i| builder::line(&v[i], &v[(i + 1) % v.len()]))
        .collect()
}

/// Closed, counter-clockwise loop of four cubic Bezier segments (an egg-ish blob, not a circle,
/// so the curves are genuinely free-form).
fn bezier_blob(cx: f64, cy: f64, rx: f64, ry: f64, z: f64) -> Wire {
    const K: f64 = 0.552_284_749_8; // circle approximation constant
    let p = |x: f64, y: f64| Point3::new(cx + x, cy + y, z);
    // Anchor points: east, north, west, south. North bulges a bit to make it asymmetric.
    let anchors = [p(rx, 0.0), p(0.0, ry * 1.3), p(-rx, 0.0), p(0.0, -ry)];
    let v: Vec<Vertex> = anchors.iter().map(|&a| builder::vertex(a)).collect();
    let ctrl = [
        [p(rx, K * ry * 1.3), p(K * rx, ry * 1.3)],
        [p(-K * rx, ry * 1.3), p(-rx, K * ry * 1.3)],
        [p(-rx, -K * ry), p(-K * rx, -ry)],
        [p(K * rx, -ry), p(rx, -K * ry)],
    ];
    (0..4)
        .map(|i| -> Edge { builder::bezier(&v[i], &v[(i + 1) % 4], ctrl[i].to_vec()) })
        .collect()
}

fn extrude(face: &Face, height: f64) -> Solid {
    builder::tsweep(face, Vector3::new(0.0, 0.0, height))
}

fn planar_face(wires: Vec<Wire>) -> Result<Face, String> {
    builder::try_attach_plane(wires).map_err(|e| format!("try_attach_plane: {e:?}"))
}

/// Builds `Base`, `Black` (with the Bezier hole) and `Black-1`, in [`NAMES`] order.
pub fn build_solids() -> Result<Vec<Solid>, String> {
    let base = planar_face(vec![polygon(
        &[(0.0, 0.0), (40.0, 0.0), (40.0, 40.0), (0.0, 40.0)],
        0.0,
    )])?;

    let z = BASE_HEIGHT;
    let mut hole = bezier_blob(20.0, 11.0, 6.0, 3.0, z);
    hole.invert(); // inner loops run clockwise
    let island_a = planar_face(vec![
        polygon(&[(5.0, 4.0), (35.0, 4.0), (35.0, 19.0), (5.0, 19.0)], z),
        hole,
    ])?;
    let island_b = planar_face(vec![polygon(
        &[
            (5.0, 23.0),
            (35.0, 23.0),
            (35.0, 36.0),
            (20.0, 32.0),
            (5.0, 36.0),
        ],
        z,
    )])?;

    Ok(vec![
        extrude(&base, BASE_HEIGHT),
        extrude(&island_a, LAYER_HEIGHT),
        extrude(&island_b, LAYER_HEIGHT),
    ])
}

fn header(file_name: &str) -> StepHeaderDescriptor {
    StepHeaderDescriptor {
        file_name: file_name.to_owned(),
        // Fixed: `Default` reads the clock (chrono::Utc::now), which breaks determinism.
        time_stamp: "2026-01-01T00:00:00".to_owned(),
        authors: vec![],
        organization: vec![],
        organization_system: "FlatArtMaker step spike".to_owned(),
        authorization: String::new(),
    }
}

fn compress(solids: &[Solid]) -> Vec<CSolid> {
    solids.iter().map(Solid::compress).collect()
}

/// Strategy (a), raw truck output: one PRODUCT whose SHAPE_REPRESENTATION holds all solids.
/// truck writes every `MANIFOLD_SOLID_BREP` with an empty name.
pub fn single_part_raw(solids: &[Solid]) -> String {
    let compressed = compress(solids);
    let models: Vec<StepModel<'_, _, _, _>> = compressed.iter().map(StepModel::from).collect();
    let mut assy = Assembly::<Vec<_>, PartAttrs, Matrix4, PartAttrs>::new();
    assy.create_node(NodeEntity {
        shape: models,
        attrs: PartAttrs {
            id: "FlatArt".into(),
            name: "FlatArt".into(),
            description: String::new(),
        },
    });
    StepDisplay::new(header("single-part-named.step"), StepDesign::new(assy)).to_string()
}

/// Strategy (b): an assembly root with one child PRODUCT per solid, named via `PartAttrs`.
pub fn assembly(solids: &[Solid]) -> String {
    let compressed = compress(solids);
    let mut assy = Assembly::<Option<StepModel<'_, _, _, _>>, PartAttrs, Matrix4, PartAttrs>::new();
    let root = assy.create_node(NodeEntity {
        shape: None,
        attrs: PartAttrs {
            id: "FlatArt".into(),
            name: "FlatArt".into(),
            description: String::new(),
        },
    });
    for (solid, name) in compressed.iter().zip(NAMES) {
        let child = assy.create_node(NodeEntity {
            shape: Some(StepModel::from(solid)),
            attrs: PartAttrs {
                id: name.into(),
                name: name.into(),
                description: String::new(),
            },
        });
        assy.create_edge(
            root,
            child,
            EdgeEntity {
                matrix: Matrix4::from_scale(1.0),
                attrs: PartAttrs {
                    id: name.into(),
                    name: name.into(),
                    description: String::new(),
                },
            },
        );
    }
    StepDisplay::new(header("assembly-named.step"), StepDesign::new(assy)).to_string()
}

/// Fills the empty names of the `MANIFOLD_SOLID_BREP` lines, in file order.
pub fn patch_brep_names(step: &str, names: &[&str]) -> Result<String, String> {
    const NEEDLE: &str = "= MANIFOLD_SOLID_BREP('',";
    let found = step.matches(NEEDLE).count();
    if found != names.len() {
        return Err(format!(
            "expected {} MANIFOLD_SOLID_BREP, found {found}",
            names.len()
        ));
    }
    let mut out = String::with_capacity(step.len() + 64);
    let mut rest = step;
    for name in names {
        let at = rest.find(NEEDLE).expect("counted above");
        out.push_str(&rest[..at]);
        out.push_str(&format!("= MANIFOLD_SOLID_BREP('{name}',"));
        rest = &rest[at + NEEDLE.len()..];
    }
    out.push_str(rest);
    Ok(out)
}

/// Replaces truck's hard-coded `'ISO-10303-203'` schema with the AP214 schema name.
pub fn patch_schema_ap214(step: &str) -> Result<String, String> {
    if !step.contains(TRUCK_SCHEMA) {
        return Err("truck schema line not found".into());
    }
    Ok(step.replacen(TRUCK_SCHEMA, AP214_SCHEMA, 1))
}

/// Strategy (a) as delivered: names patched in, schema patched to AP214.
pub fn single_part_named() -> Result<String, String> {
    let solids = build_solids()?;
    patch_schema_ap214(&patch_brep_names(&single_part_raw(&solids), &NAMES)?)
}

/// Strategy (b) as delivered: names from truck, schema patched to AP214.
pub fn assembly_named() -> Result<String, String> {
    patch_schema_ap214(&assembly(&build_solids()?))
}

// --- Colour groups ---------------------------------------------------------
// A 100 x 60 base plus five colours, each with two islands. The first island of
// every colour has a Bezier hole. Two STEP layouts are written:
//   (c) one component per colour whose single representation holds all its bodies;
//   (d) one component per colour holding one sub-component per body.

pub const COLOURS: [&str; 5] = ["Black", "White", "Red", "Yellow", "Blue"];

/// A named colour group: the colour name and its bodies, largest first.
pub struct Group {
    pub name: &'static str,
    pub solids: Vec<Solid>,
}

/// `Black`, `Black-1`, `Black-2`, ... for a group of `n` bodies.
pub fn body_names(colour: &str, n: usize) -> Vec<String> {
    (0..n)
        .map(|i| match i {
            0 => colour.to_owned(),
            _ => format!("{colour}-{i}"),
        })
        .collect()
}

/// Builds the base and the five colour groups.
pub fn build_groups() -> Result<(Solid, Vec<Group>), String> {
    let base = planar_face(vec![polygon(
        &[(0.0, 0.0), (100.0, 0.0), (100.0, 60.0), (0.0, 60.0)],
        0.0,
    )])?;
    let z = BASE_HEIGHT;
    let mut groups = Vec::new();
    for (i, name) in COLOURS.into_iter().enumerate() {
        let x0 = 2.5 + i as f64 * 19.5;
        let x1 = x0 + 16.0;
        let mut hole = bezier_blob((x0 + x1) / 2.0, 15.0, 4.0, 4.0, z);
        hole.invert();
        let big = planar_face(vec![
            polygon(&[(x0, 4.0), (x1, 4.0), (x1, 28.0), (x0, 28.0)], z),
            hole,
        ])?;
        let small = planar_face(vec![polygon(
            &[(x0, 33.0), (x1, 33.0), (x1, 55.0), ((x0 + x1) / 2.0, 50.0), (x0, 55.0)],
            z,
        )])?;
        groups.push(Group {
            name,
            solids: vec![extrude(&big, LAYER_HEIGHT), extrude(&small, LAYER_HEIGHT)],
        });
    }
    Ok((extrude(&base, BASE_HEIGHT), groups))
}

fn attrs(name: &str) -> PartAttrs {
    PartAttrs {
        id: name.into(),
        name: name.into(),
        description: String::new(),
    }
}

fn identity(name: &str) -> EdgeEntity<Matrix4, PartAttrs> {
    EdgeEntity {
        matrix: Matrix4::from_scale(1.0),
        attrs: attrs(name),
    }
}

/// Body names in the order truck writes the bodies: `Base`, then each group in order.
pub fn grouped_body_names(groups: &[Group]) -> Vec<String> {
    std::iter::once("Base".to_owned())
        .chain(groups.iter().flat_map(|g| body_names(g.name, g.solids.len())))
        .collect()
}

/// Layout (c): root -> `Base` and one component per colour; a colour component's single
/// representation holds all of that colour's bodies.
pub fn grouped_bodies(base: &Solid, groups: &[Group]) -> String {
    let base_c = base.compress();
    let groups_c: Vec<Vec<CSolid>> = groups.iter().map(|g| compress(&g.solids)).collect();
    let mut assy = Assembly::<Vec<StepModel<'_, _, _, _>>, PartAttrs, Matrix4, PartAttrs>::new();
    let root = assy.create_node(NodeEntity {
        shape: vec![],
        attrs: attrs("FlatArt"),
    });
    let base_node = assy.create_node(NodeEntity {
        shape: vec![StepModel::from(&base_c)],
        attrs: attrs("Base"),
    });
    assy.create_edge(root, base_node, identity("Base"));
    for (group, solids) in groups.iter().zip(&groups_c) {
        let node = assy.create_node(NodeEntity {
            shape: solids.iter().map(StepModel::from).collect(),
            attrs: attrs(group.name),
        });
        assy.create_edge(root, node, identity(group.name));
    }
    StepDisplay::new(header("grouped-bodies.step"), StepDesign::new(assy)).to_string()
}

/// Layout (d): root -> `Base` and one component per colour; each colour component holds one
/// named sub-component per body.
pub fn grouped_subcomponents(base: &Solid, groups: &[Group]) -> String {
    let base_c = base.compress();
    let groups_c: Vec<Vec<CSolid>> = groups.iter().map(|g| compress(&g.solids)).collect();
    let mut assy = Assembly::<Option<StepModel<'_, _, _, _>>, PartAttrs, Matrix4, PartAttrs>::new();
    let root = assy.create_node(NodeEntity {
        shape: None,
        attrs: attrs("FlatArt"),
    });
    let base_node = assy.create_node(NodeEntity {
        shape: Some(StepModel::from(&base_c)),
        attrs: attrs("Base"),
    });
    assy.create_edge(root, base_node, identity("Base"));
    for (group, solids) in groups.iter().zip(&groups_c) {
        let colour = assy.create_node(NodeEntity {
            shape: None,
            attrs: attrs(group.name),
        });
        assy.create_edge(root, colour, identity(group.name));
        for (solid, name) in solids.iter().zip(body_names(group.name, solids.len())) {
            let body = assy.create_node(NodeEntity {
                shape: Some(StepModel::from(solid)),
                attrs: attrs(&name),
            });
            assy.create_edge(colour, body, identity(&name));
        }
    }
    StepDisplay::new(header("grouped-subcomponents.step"), StepDesign::new(assy)).to_string()
}

/// Layout (c) as delivered: body names patched in, schema patched to AP214.
pub fn grouped_bodies_named() -> Result<String, String> {
    let (base, groups) = build_groups()?;
    let names = grouped_body_names(&groups);
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    patch_schema_ap214(&patch_brep_names(&grouped_bodies(&base, &groups), &names)?)
}

/// Layout (d) as delivered: names from truck, schema patched to AP214.
pub fn grouped_subcomponents_named() -> Result<String, String> {
    let (base, groups) = build_groups()?;
    patch_schema_ap214(&grouped_subcomponents(&base, &groups))
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn single_part_step() -> Result<String, String> {
        super::single_part_named()
    }

    #[wasm_bindgen]
    pub fn assembly_step() -> Result<String, String> {
        super::assembly_named()
    }
}
