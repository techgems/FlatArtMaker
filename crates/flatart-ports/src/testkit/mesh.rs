//! Mesh checks, reused by the mesh builder (task 10) and 3MF tests.

use flatart_domain::{Mesh, MeshModel};
use std::collections::HashMap;

/// Panics unless `mesh` is a closed, consistently oriented surface facing
/// outward: every edge is used exactly once in each direction, no triangle
/// is degenerate, and the enclosed volume is positive.
pub fn assert_watertight(mesh: &Mesh, name: &str) {
    assert!(!mesh.triangles.is_empty(), "{name}: mesh has no triangles");
    let vertex_count = mesh.vertices.len();
    let mut edges: HashMap<(u32, u32), u32> = HashMap::new();
    for (t, tri) in mesh.triangles.iter().enumerate() {
        assert!(
            tri.iter().all(|&i| (i as usize) < vertex_count),
            "{name}: triangle {t} points past the vertex list"
        );
        assert!(
            tri[0] != tri[1] && tri[1] != tri[2] && tri[0] != tri[2],
            "{name}: triangle {t} repeats a vertex"
        );
        for k in 0..3 {
            *edges.entry((tri[k], tri[(k + 1) % 3])).or_default() += 1;
        }
    }
    for (&(a, b), &count) in &edges {
        assert_eq!(count, 1, "{name}: edge {a}→{b} is used {count} times");
        assert!(
            edges.contains_key(&(b, a)),
            "{name}: edge {a}→{b} has no opposite edge, so the mesh is open"
        );
    }
    let volume = signed_volume(mesh);
    assert!(
        volume > 0.0,
        "{name}: volume is {volume}, so the triangles face inward"
    );
}

/// Every part of `model` passes [`assert_watertight`].
pub fn assert_model_watertight(model: &MeshModel) {
    for (i, part) in model.parts.iter().enumerate() {
        let name = model.part_name(part).unwrap_or("?");
        assert_watertight(&part.mesh, &format!("part {i} ({name})"));
    }
}

/// The enclosed volume, positive when triangles face outward.
pub fn signed_volume(mesh: &Mesh) -> f64 {
    mesh.triangles
        .iter()
        .map(|tri| {
            let [a, b, c] = tri.map(|i| {
                let v = mesh.vertices[i as usize];
                [v.x.0, v.y.0, v.z.0]
            });
            let cross = [
                b[1] * c[2] - b[2] * c[1],
                b[2] * c[0] - b[0] * c[2],
                b[0] * c[1] - b[1] * c[0],
            ];
            (a[0] * cross[0] + a[1] * cross[1] + a[2] * cross[2]) / 6.0
        })
        .sum()
}
