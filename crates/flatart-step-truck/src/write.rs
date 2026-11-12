//! The STEP text: one component per part holding its bodies, then the fixes
//! truck's writer needs.

use crate::brep::Body;
use crate::compact::compact;
use flatart_domain::{Error, Result, Stage};
use truck_assembly::assy::{Assembly, EdgeEntity, NodeEntity};
use truck_modeling::Matrix4;
use truck_stepio::common::PartAttrs;
use truck_stepio::out::{StepDesign, StepDisplay, StepHeaderDescriptor, StepModel};

/// The schema truck writes (not a real schema name), and AP214's.
const TRUCK_SCHEMA: &str = "FILE_SCHEMA(('ISO-10303-203'));";
const AP214_SCHEMA: &str = "FILE_SCHEMA(('AUTOMOTIVE_DESIGN { 1 0 10303 214 1 1 1 1 }'));";

/// A fixed header time: the default reads the clock.
const TIME_STAMP: &str = "2026-01-01T00:00:00";

fn failed(message: impl Into<String>) -> Error {
    Error::StageFailed {
        stage: Stage::Export,
        message: message.into(),
    }
}

/// One named component and its bodies, already in naming order.
pub struct Component {
    pub name: String,
    pub bodies: Vec<Body>,
}

/// `Black`, `Black-1`, `Black-2`, …
fn body_names(name: &str, n: usize) -> impl Iterator<Item = String> + '_ {
    (0..n).map(move |i| match i {
        0 => name.to_owned(),
        _ => format!("{name}-{i}"),
    })
}

/// A name safe inside a STEP string: letters, digits, spaces, `-`, `_`
/// and `.` only.
fn safe(name: &str) -> String {
    let kept: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.'))
        .collect();
    let kept = kept.trim();
    if kept.is_empty() {
        "FlatArt".to_owned()
    } else {
        kept.to_owned()
    }
}

fn attrs(name: &str) -> PartAttrs {
    PartAttrs {
        id: name.into(),
        name: name.into(),
        description: String::new(),
    }
}

/// The whole file: a root part named `root`, holding one component per
/// entry of `components`.
pub fn write(root: &str, components: &[Component]) -> Result<String> {
    let root = safe(root);
    let compressed: Vec<Vec<_>> = components
        .iter()
        .map(|c| c.bodies.iter().map(|b| b.solid.compress()).collect())
        .collect();
    let mut assy = Assembly::<Vec<StepModel<'_, _, _, _>>, PartAttrs, Matrix4, PartAttrs>::new();
    let top = assy.create_node(NodeEntity {
        shape: vec![],
        attrs: attrs(&root),
    });
    for (component, solids) in components.iter().zip(&compressed) {
        let node = assy.create_node(NodeEntity {
            shape: solids.iter().map(StepModel::from).collect(),
            attrs: attrs(&component.name),
        });
        assy.create_edge(
            top,
            node,
            EdgeEntity {
                matrix: Matrix4::from_scale(1.0),
                attrs: attrs(&component.name),
            },
        );
    }
    let header = StepHeaderDescriptor {
        file_name: format!("{root}.step"),
        time_stamp: TIME_STAMP.to_owned(),
        authors: vec![],
        organization: vec![],
        organization_system: "FlatArtMaker".to_owned(),
        authorization: String::new(),
    };
    let text = StepDisplay::new(header, StepDesign::new(assy)).to_string();
    let names: Vec<String> = components
        .iter()
        .flat_map(|c| body_names(&c.name, c.bodies.len()))
        .collect();
    Ok(compact(&patch_schema(&patch_body_names(&text, &names)?)?))
}

/// Fills in the bodies' names, which truck writes empty, in file order
/// (truck writes the bodies in the order they were given).
fn patch_body_names(step: &str, names: &[String]) -> Result<String> {
    const NEEDLE: &str = "= MANIFOLD_SOLID_BREP('',";
    let found = step.matches(NEEDLE).count();
    if found != names.len() {
        return Err(failed(format!(
            "expected {} STEP bodies, truck wrote {found}",
            names.len()
        )));
    }
    let mut out = String::with_capacity(step.len() + names.len() * 16);
    let mut rest = step;
    for name in names {
        let Some(at) = rest.find(NEEDLE) else {
            return Err(failed("a STEP body went missing"));
        };
        out.push_str(&rest[..at]);
        out.push_str(&format!("= MANIFOLD_SOLID_BREP('{name}',"));
        rest = &rest[at + NEEDLE.len()..];
    }
    out.push_str(rest);
    Ok(out)
}

/// Replaces truck's schema line with AP214's.
fn patch_schema(step: &str) -> Result<String> {
    if !step.contains(TRUCK_SCHEMA) {
        return Err(failed("truck's STEP schema line wasn't found"));
    }
    Ok(step.replacen(TRUCK_SCHEMA, AP214_SCHEMA, 1))
}
