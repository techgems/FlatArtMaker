//! Writes the spike STEP files to `out/` and validates them by re-reading with truck-stepio.

use std::path::Path;
use step_spike::*;
use truck_modeling::{BoundedCurve, Point3, Solid, Surface, Tolerance};
use truck_stepio::r#in::Table;
use truck_topology::shell::ShellCondition;
use truck_topology::Solid as TSolid;

fn check_solid(name: &str, solid: &Solid) {
    let shells = solid.boundaries();
    let faces: usize = shells.iter().map(|s| s.len()).sum();
    let closed = shells
        .iter()
        .all(|s| s.shell_condition() == ShellCondition::Closed);
    // Outward check on the caps: the plane at the solid's lowest z must face -Z, the highest +Z.
    let zs: Vec<f64> = shells[0].vertex_iter().map(|v| v.point().z).collect();
    let zmin = zs.iter().copied().fold(f64::INFINITY, f64::min);
    let mut caps_ok = true;
    for face in shells[0].face_iter() {
        if let Surface::Plane(plane) = face.oriented_surface() {
            let n = plane.normal();
            if n.z.abs() > 0.999 {
                let z = plane.origin().z;
                caps_ok &= (n.z > 0.0) == (z > zmin + 1e-9);
            }
        }
    }
    println!(
        "  {name}: shells={} faces={faces} closed={closed} geometric_consistent={} caps_outward={caps_ok}",
        shells.len(),
        solid.is_geometric_consistent()
    );
    assert!(
        closed && solid.is_geometric_consistent() && caps_ok,
        "{name} invalid"
    );
}

fn reread(label: &str, step: &str) {
    println!("re-read {label}:");
    let table = Table::from_step(step).expect("ruststep parse failed");
    let mut breps: Vec<_> = table.manifold_solid_brep.iter().collect();
    breps.sort_by_key(|(k, _)| **k);
    println!("  MANIFOLD_SOLID_BREP count = {}", breps.len());
    for (idx, brep) in breps {
        let csolid = table
            .to_compressed_solid(brep)
            .unwrap_or_else(|e| panic!("#{idx}: {e:?}"));
        let faces: usize = csolid.boundaries.iter().map(|s| s.faces.len()).sum();
        let solid = TSolid::<Point3, _, _>::extract(csolid).expect("topology extract failed");
        let closed = solid
            .boundaries()
            .iter()
            .all(|s| s.shell_condition() == ShellCondition::Closed);
        let (lo, hi) = solid.boundaries()[0].vertex_iter().fold(
            ([f64::MAX; 3], [f64::MIN; 3]),
            |(mut lo, mut hi), v| {
                let p = v.point();
                for (i, c) in [p.x, p.y, p.z].into_iter().enumerate() {
                    lo[i] = lo[i].min(c);
                    hi[i] = hi[i].max(c);
                }
                (lo, hi)
            },
        );
        // `is_geometric_consistent` is not available for the reader's geometry types, so check
        // that every edge curve starts and ends on its vertices.
        let edges_ok = solid.boundaries()[0].edge_iter().all(|e| {
            let c = e.oriented_curve();
            c.front().near(&e.front().point()) && c.back().near(&e.back().point())
        });
        println!(
            "  #{idx} name='{}' shells={} faces={faces} closed={closed} edge_endpoints_ok={edges_ok} vertex bbox={lo:?}..{hi:?}",
            brep.label,
            solid.boundaries().len(),
        );
        assert!(closed && edges_ok);
    }
    let assy = table.step_assy().expect("step_assy failed");
    for node in assy.all_nodes() {
        let attrs = node.attrs();
        println!(
            "  PRODUCT id='{}' name='{}' items={} children={}",
            attrs.id,
            attrs.name,
            node.shape().len(),
            node.edges().count()
        );
    }
}

fn header_summary(step: &str) {
    let first = step.lines().next().unwrap_or_default();
    let schema = step
        .lines()
        .find(|l| l.starts_with("FILE_SCHEMA"))
        .unwrap_or("<none>");
    println!("  first line: {first}\n  {schema}");
}

fn main() {
    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join("out");
    std::fs::create_dir_all(&out).unwrap();

    let solids = build_solids().unwrap();
    println!("built solids:");
    for (s, n) in solids.iter().zip(NAMES) {
        check_solid(n, s);
    }

    let raw_single = single_part_raw(&solids);
    let raw_assy = assembly(&solids);
    let single = single_part_named().unwrap();
    let assy = assembly_named().unwrap();
    assert_eq!(single, single_part_named().unwrap(), "not deterministic");

    for (file, text) in [
        ("single-part.truck-raw.step", &raw_single),
        ("assembly.truck-raw.step", &raw_assy),
        ("single-part-named.step", &single),
        ("assembly-named.step", &assy),
    ] {
        std::fs::write(out.join(file), text).unwrap();
        println!("wrote out/{file} ({} bytes)", text.len());
        header_summary(text);
    }

    reread("single-part-named.step", &single);
    reread("assembly-named.step", &assy);
    reread("single-part.truck-raw.step", &raw_single);

    let (base, groups) = build_groups().unwrap();
    println!("built colour groups:");
    check_solid("Base", &base);
    for g in &groups {
        for (s, n) in g.solids.iter().zip(body_names(g.name, g.solids.len())) {
            check_solid(&n, s);
        }
    }
    let bodies = grouped_bodies_named().unwrap();
    let subs = grouped_subcomponents_named().unwrap();
    assert_eq!(bodies, grouped_bodies_named().unwrap(), "not deterministic");
    for (file, text) in [
        ("grouped-bodies.step", &bodies),
        ("grouped-subcomponents.step", &subs),
    ] {
        std::fs::write(out.join(file), text).unwrap();
        println!("wrote out/{file} ({} bytes)", text.len());
        header_summary(text);
        reread(file, text);
    }
}
