//! Walking the pixel edges into borders.
//!
//! A vertex is a junction where 3 or 4 edges meet (3 or more areas, under
//! 4-connectivity), or on the image's edge where its two cells differ (2
//! areas meet the image edge). Borders run from junction to junction; edges
//! left over form closed loops with no junction.

use crate::grid::{Dir, Grid, Vertex, step};
use flatart_domain::Label;
use std::collections::HashMap;

/// One border, as the vertices it passes through.
pub struct Chain {
    /// At least 2. A closed chain ends where it starts.
    pub vertices: Vec<Vertex>,
    /// The junctions it starts and ends at, or `None` for a closed loop.
    pub ends: Option<(u32, u32)>,
    pub left: Label,
    pub right: Label,
}

impl Chain {
    pub fn first_dir(&self) -> Dir {
        Dir::between(self.vertices[0], self.vertices[1])
    }

    pub fn last_dir(&self) -> Dir {
        let n = self.vertices.len();
        Dir::between(self.vertices[n - 2], self.vertices[n - 1])
    }
}

pub struct Chains {
    pub junctions: Vec<Vertex>,
    pub chains: Vec<Chain>,
}

fn is_junction(grid: &Grid, v: Vertex) -> bool {
    let degree = grid.degree(v);
    if degree >= 3 {
        return true;
    }
    if degree < 2 {
        return false;
    }
    let (x, y) = v;
    let inside: Vec<_> = [(x - 1, y - 1), (x, y - 1), (x - 1, y), (x, y)]
        .into_iter()
        .filter(|&c| grid.in_image(c))
        .collect();
    inside.len() == 2 && grid.label(inside[0]) != grid.label(inside[1])
}

struct Walker<'g, 'a> {
    grid: &'g Grid<'a>,
    visited: Vec<bool>,
    junction_ids: HashMap<usize, u32>,
}

impl Walker<'_, '_> {
    fn junction(&self, v: Vertex) -> Option<u32> {
        self.junction_ids.get(&self.grid.vertex_index(v)).copied()
    }

    fn unvisited(&self, v: Vertex, d: Dir) -> bool {
        self.grid.has_edge(v, d) && !self.visited[self.grid.edge_index(v, d)]
    }

    /// Walks from `start` along `d` until a junction, or back to `start`.
    fn walk(&mut self, start: Vertex, mut d: Dir) -> Chain {
        let (left, right) = self.grid.side_labels(start, d);
        let mut vertices = vec![start];
        let mut at = start;
        loop {
            self.visited[self.grid.edge_index(at, d)] = true;
            at = step(at, d);
            vertices.push(at);
            if at == start || self.junction(at).is_some() {
                break;
            }
            // Not a junction, so exactly one other edge leaves here.
            match Dir::ALL
                .into_iter()
                .find(|&e| e != d.reverse() && self.grid.has_edge(at, e))
            {
                Some(next) => d = next,
                None => break,
            }
        }
        // A walk from a junction always ends at one; a loop has neither.
        let ends = self.junction(start).zip(self.junction(at));
        Chain {
            vertices,
            ends,
            left,
            right,
        }
    }
}

/// Every border of the grid, in a fixed order: from each junction in row
/// order, then the closed loops from their top-left vertex.
pub fn trace(grid: &Grid) -> Chains {
    let mut junctions = Vec::new();
    let mut junction_ids = HashMap::new();
    for y in 0..=grid.height {
        for x in 0..=grid.width {
            if is_junction(grid, (x, y)) {
                junction_ids.insert(grid.vertex_index((x, y)), junctions.len() as u32);
                junctions.push((x, y));
            }
        }
    }
    let mut walker = Walker {
        grid,
        visited: vec![false; grid.edge_count()],
        junction_ids,
    };
    let mut chains = Vec::new();
    for &j in &junctions {
        for d in Dir::ALL {
            if walker.unvisited(j, d) {
                chains.push(walker.walk(j, d));
            }
        }
    }
    for y in 0..=grid.height {
        for x in 0..=grid.width {
            for d in Dir::ALL {
                if walker.unvisited((x, y), d) {
                    chains.push(walker.walk((x, y), d));
                }
            }
        }
    }
    Chains { junctions, chains }
}
