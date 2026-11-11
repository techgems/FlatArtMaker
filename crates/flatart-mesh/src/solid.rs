//! One closed part, built from regions and how high each one goes.
//!
//! Each region with material is a flat slab from its bottom to its top. Its
//! top and bottom are its triangles; walls stand on the borders wherever
//! the material on the two sides differs, split at every height in the part
//! so walls meeting at a junction share their vertical edges.
//!
//! A vertex is identified by (point, height, region, corner). Where the
//! surface carries on across a border (both regions' bottoms at the same
//! height, say), the two sides' vertices are merged. Where a region only
//! touches itself or another at a point, they stay apart, so every edge is
//! used once each way.

use crate::flatten::Outline;
use crate::regions::{Regions, RingPoint};
use flatart_domain::{Error, Mesh, MmPoint3, Result, Stage};
use std::collections::HashMap;

/// (point, height index, region, corner).
type Key = (u32, u8, u32, u32);

fn failed(message: &str) -> Error {
    Error::StageFailed {
        stage: Stage::Mesh,
        message: message.into(),
    }
}

/// Interned vertex keys, merged with union-find.
#[derive(Default)]
struct Keys {
    keys: Vec<Key>,
    ids: HashMap<Key, u32>,
    parent: Vec<u32>,
}

impl Keys {
    fn id(&mut self, key: Key) -> u32 {
        if let Some(&id) = self.ids.get(&key) {
            return id;
        }
        let id = self.keys.len() as u32;
        self.keys.push(key);
        self.ids.insert(key, id);
        self.parent.push(id);
        id
    }

    fn root(&mut self, mut id: u32) -> u32 {
        while self.parent[id as usize] != id {
            let up = self.parent[self.parent[id as usize] as usize];
            self.parent[id as usize] = up;
            id = up;
        }
        id
    }

    fn join(&mut self, a: u32, b: u32) {
        let (ra, rb) = (self.root(a), self.root(b));
        if ra != rb {
            // The lower id wins, so the result doesn't depend on the order.
            let (lo, hi) = (ra.min(rb), ra.max(rb));
            self.parent[hi as usize] = lo;
        }
    }
}

/// What one part is made of.
pub struct Part<'a> {
    pub outline: &'a Outline,
    pub regions: &'a Regions,
    /// Each region's triangles (empty for regions not in this part).
    pub triangles: &'a [Vec<[RingPoint; 3]>],
    /// Each region's bottom and top, or `None` where the part has nothing.
    pub heights: &'a [Option<(f64, f64)>],
}

impl Part<'_> {
    fn levels(&self) -> Vec<f64> {
        let mut levels: Vec<f64> = self
            .heights
            .iter()
            .flatten()
            .flat_map(|&(lo, hi)| [lo, hi])
            .collect();
        levels.sort_by(f64::total_cmp);
        levels.dedup();
        levels
    }

    /// Whether `region` has material between levels `s` and `s + 1`.
    fn solid_in(&self, region: Option<u32>, levels: &[f64], s: usize) -> bool {
        let Some(r) = region else { return false };
        match self.heights[r as usize] {
            Some((lo, hi)) => s + 1 < levels.len() && lo <= levels[s] && levels[s + 1] <= hi,
            None => false,
        }
    }

    /// The region on each side of `border`, if it's in this part.
    fn sides(&self, border: usize) -> [Option<u32>; 2] {
        self.regions.sides[border].map(|s| s.filter(|&r| self.heights[r as usize].is_some()))
    }

    /// Builds the closed mesh.
    pub fn build(&self) -> Result<Mesh> {
        let levels = self.levels();
        if levels.len() < 2 || levels.len() > usize::from(u8::MAX) {
            return Err(failed("a part needs between 2 and 255 heights"));
        }
        let level = |z: f64| levels.iter().position(|&l| l == z).unwrap_or(0) as u8;
        let mut keys = Keys::default();
        let mut triangles: Vec<[u32; 3]> = Vec::new();

        // Tops and bottoms.
        for (r, tris) in self.triangles.iter().enumerate() {
            let Some((lo, hi)) = self.heights[r] else {
                continue;
            };
            let (lo, hi) = (level(lo), level(hi));
            for tri in tris {
                let at = |keys: &mut Keys, c: RingPoint, l: u8| {
                    keys.id((c.point, l, r as u32, c.corner))
                };
                let top = tri.map(|c| at(&mut keys, c, hi));
                let bottom = tri.map(|c| at(&mut keys, c, lo));
                triangles.push(top);
                triangles.push([bottom[0], bottom[2], bottom[1]]);
            }
        }

        // Walls, and merging the two sides where the surface carries on.
        for border in 0..self.outline.borders.len() {
            let [left, right] = self.sides(border);
            if left.is_none() && right.is_none() {
                continue;
            }
            let ids = &self.outline.borders[border];
            let key = |keys: &mut Keys, region: u32, i: usize, l: usize| {
                let corner = self.regions.corner(self.outline, region, border, i);
                keys.id((ids[i], l as u8, region, corner))
            };
            for i in 0..ids.len() - 1 {
                for s in 0..levels.len() - 1 {
                    let (on_left, on_right) = (
                        self.solid_in(left, &levels, s),
                        self.solid_in(right, &levels, s),
                    );
                    if on_left == on_right {
                        continue;
                    }
                    let side = if on_left { left } else { right };
                    let Some(side) = side else { continue };
                    let [a0, b0, a1, b1] = [(i, s), (i + 1, s), (i, s + 1), (i + 1, s + 1)]
                        .map(|(p, l)| key(&mut keys, side, p, l));
                    if on_left {
                        // Material on the left, so the wall faces right.
                        triangles.push([a0, b0, b1]);
                        triangles.push([a0, b1, a1]);
                    } else {
                        triangles.push([a0, b1, b0]);
                        triangles.push([a0, a1, b1]);
                    }
                }
            }
            for i in 0..ids.len() {
                for l in 0..levels.len() {
                    // The surfaces meeting along this border at height l.
                    let mut meeting: Vec<u32> = Vec::with_capacity(4);
                    let below = |r| l > 0 && self.solid_in(r, &levels, l - 1);
                    let above = |r| self.solid_in(r, &levels, l);
                    for side in [left, right].into_iter().flatten() {
                        if below(Some(side)) != above(Some(side)) {
                            meeting.push(key(&mut keys, side, i, l));
                        }
                    }
                    for (l_solid, r_solid) in
                        [(below(left), below(right)), (above(left), above(right))]
                    {
                        if l_solid != r_solid {
                            let side = if l_solid { left } else { right };
                            if let Some(side) = side {
                                meeting.push(key(&mut keys, side, i, l));
                            }
                        }
                    }
                    match meeting.as_slice() {
                        [a, b] => keys.join(*a, *b),
                        [_, _, _, _] => {
                            return Err(failed("two parts of one mesh meet along an edge"));
                        }
                        _ => {}
                    }
                }
            }
        }

        // Number the vertices in the order the triangles first use them.
        let mut index: HashMap<u32, u32> = HashMap::new();
        let mut vertices = Vec::new();
        let mut out = Vec::with_capacity(triangles.len());
        for tri in triangles {
            let mapped = tri.map(|k| {
                let root = keys.root(k);
                *index.entry(root).or_insert_with(|| {
                    let (point, l, _, _) = keys.keys[root as usize];
                    let (x, y) = self.outline.points[point as usize];
                    vertices.push(MmPoint3::new(x, y, levels[usize::from(l)]));
                    (vertices.len() - 1) as u32
                })
            });
            if mapped[0] == mapped[1] || mapped[1] == mapped[2] || mapped[0] == mapped[2] {
                return Err(failed("a triangle collapsed onto a repeated vertex"));
            }
            out.push(mapped);
        }
        Ok(Mesh {
            vertices,
            triangles: out,
        })
    }
}
