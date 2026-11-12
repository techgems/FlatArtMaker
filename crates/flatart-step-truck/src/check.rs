//! Reads a STEP file back and checks it (feature `check`, for tests).
//!
//! - **Structure**, without truck: every `#` reference exists, and every
//!   edge is used exactly once in each direction.
//! - **Solids**, with truck's reader: each body is one closed shell whose
//!   edges start and end on their vertices.
//! - **Names**: the products and the bodies, in file order.
//!
//! Only reading is checked here; whether CAD tools open the file cleanly is
//! checked by hand (Fusion).

use std::collections::{BTreeMap, HashMap};
use truck_modeling::{BoundedCurve, Point3, Tolerance};
use truck_stepio::r#in::Table;
use truck_topology::Solid;
use truck_topology::shell::ShellCondition;

/// What a file holds.
#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    /// The schema line's schema.
    pub schema: String,
    /// Each product's name, in file order. The root comes first.
    pub products: Vec<String>,
    /// Each body's name, in file order.
    pub bodies: Vec<String>,
    /// The lowest and highest vertex coordinates (mm).
    pub bounds: ([f64; 3], [f64; 3]),
}

/// The `#` numbers in `text`.
fn refs(text: &str) -> impl Iterator<Item = u64> + '_ {
    text.split('#').skip(1).filter_map(|s| {
        let digits: String = s.chars().take_while(char::is_ascii_digit).collect();
        digits.parse().ok()
    })
}

/// Every entity's number and text, joined onto one line.
fn entities(step: &str) -> Result<BTreeMap<u64, String>, String> {
    let data = step
        .split_once("DATA;")
        .and_then(|(_, rest)| rest.rsplit_once("ENDSEC;"))
        .ok_or("no DATA section")?
        .0;
    let mut out = BTreeMap::new();
    for chunk in data.split(';') {
        let chunk = chunk.split_whitespace().collect::<Vec<_>>().join(" ");
        if chunk.is_empty() {
            continue;
        }
        let (id, body) = chunk
            .strip_prefix('#')
            .and_then(|c| c.split_once(" = "))
            .ok_or_else(|| format!("can't read entity `{chunk}`"))?;
        let id: u64 = id.parse().map_err(|_| format!("bad entity id `{id}`"))?;
        out.insert(id, body.to_owned());
    }
    Ok(out)
}

/// Every reference resolves, and every edge is used once each way.
fn structure(step: &str) -> Result<(), String> {
    let ents = entities(step)?;
    for body in ents.values() {
        if let Some(missing) = refs(body).find(|r| !ents.contains_key(r)) {
            return Err(format!("#{missing} is referenced but missing"));
        }
    }
    // A loop's edges count reversed when its face bound is `.F.`.
    let mut bound_forward = HashMap::new();
    for body in ents.values() {
        if body.starts_with("FACE_BOUND(") || body.starts_with("FACE_OUTER_BOUND(") {
            let lp = refs(body).next().ok_or("a face bound has no loop")?;
            bound_forward.insert(lp, body.contains(".T."));
        }
    }
    let mut uses: HashMap<u64, (u32, u32)> = HashMap::new();
    for (id, body) in &ents {
        if !body.starts_with("EDGE_LOOP(") {
            continue;
        }
        let forward_loop = *bound_forward.get(id).ok_or("an edge loop has no face")?;
        for oe in refs(body) {
            let oriented = &ents[&oe];
            let edge = refs(oriented)
                .next()
                .ok_or("an oriented edge has no edge")?;
            let forward = oriented.trim_end().ends_with(".T.)") == forward_loop;
            let entry = uses.entry(edge).or_default();
            if forward {
                entry.0 += 1;
            } else {
                entry.1 += 1;
            }
        }
    }
    for (id, body) in &ents {
        if body.starts_with("EDGE_CURVE(") && uses.get(id) != Some(&(1, 1)) {
            return Err(format!("edge #{id} isn't used once each way"));
        }
    }
    Ok(())
}

/// Reads `step` back and checks it. Fails on the first problem found.
pub fn check(step: &str) -> Result<Summary, String> {
    if !step.starts_with("ISO-10303-21;") || !step.trim_end().ends_with("END-ISO-10303-21;") {
        return Err("not an ISO 10303-21 file".into());
    }
    let schema = step
        .lines()
        .find_map(|l| l.strip_prefix("FILE_SCHEMA(('"))
        .and_then(|l| l.split_once("'"))
        .ok_or("no schema")?
        .0
        .to_owned();
    structure(step)?;
    let table = Table::from_step(step).ok_or("truck can't parse the file")?;
    let mut breps: Vec<_> = table.manifold_solid_brep.iter().collect();
    breps.sort_by_key(|(k, _)| **k);
    let mut bodies = Vec::new();
    let (mut lo, mut hi) = ([f64::MAX; 3], [f64::MIN; 3]);
    for (idx, brep) in breps {
        let compressed = table
            .to_compressed_solid(brep)
            .map_err(|e| format!("body #{idx}: {e:?}"))?;
        let solid = Solid::<Point3, _, _>::extract(compressed)
            .map_err(|e| format!("body #{idx}: {e:?}"))?;
        let shells = solid.boundaries();
        if shells.len() != 1 || shells[0].shell_condition() != ShellCondition::Closed {
            return Err(format!(
                "body #{idx} `{}` isn't one closed shell",
                brep.label
            ));
        }
        let ends_ok = shells[0].edge_iter().all(|e| {
            let c = e.oriented_curve();
            c.front().near(&e.front().point()) && c.back().near(&e.back().point())
        });
        if !ends_ok {
            return Err(format!("body #{idx}: an edge doesn't end on its vertices"));
        }
        for v in shells[0].vertex_iter() {
            let p = v.point();
            for (i, c) in [p.x, p.y, p.z].into_iter().enumerate() {
                lo[i] = lo[i].min(c);
                hi[i] = hi[i].max(c);
            }
        }
        bodies.push(brep.label.clone());
    }
    let products = entities(step)?
        .into_values()
        .filter_map(|body| {
            let rest = body.strip_prefix("PRODUCT('")?;
            Some(rest.split_once('\'')?.0.to_owned())
        })
        .collect();
    Ok(Summary {
        schema,
        products,
        bodies,
        bounds: (lo, hi),
    })
}
