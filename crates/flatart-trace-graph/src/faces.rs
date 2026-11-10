//! Assembling faces from the borders.
//!
//! Every loop walks with its area on the right. Each border can be walked
//! forwards (its right label on the right) or backwards (its left label on
//! the right). At a junction the walk turns as far right as it can while
//! keeping its area on the right, which is what keeps diagonal cells of one
//! label apart (4-connectivity). A loop running clockwise on screen is an
//! area's outer edge; anticlockwise, one of its holes.

use crate::chains::Chain;
use crate::grid::{Cell, Dir, Grid, Vertex};
use flatart_domain::{BorderId, Error, Face, Label, Loop, LoopStep, Result, Stage};
use std::collections::HashMap;

/// A border walked one way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Walk {
    border: usize,
    reversed: bool,
}

impl Walk {
    fn step(self) -> LoopStep {
        LoopStep {
            border: BorderId(self.border as u32),
            reversed: self.reversed,
        }
    }
}

struct Walks<'c> {
    chains: &'c [Chain],
    /// The walk that leaves a vertex along a direction.
    starting: HashMap<(Vertex, Dir), Walk>,
}

impl<'c> Walks<'c> {
    fn new(chains: &'c [Chain]) -> Self {
        let mut starting = HashMap::with_capacity(chains.len() * 2);
        for (border, chain) in chains.iter().enumerate() {
            let n = chain.vertices.len();
            starting.insert(
                (chain.vertices[0], chain.first_dir()),
                Walk {
                    border,
                    reversed: false,
                },
            );
            starting.insert(
                (chain.vertices[n - 1], chain.last_dir().reverse()),
                Walk {
                    border,
                    reversed: true,
                },
            );
        }
        Self { chains, starting }
    }

    /// The label on the walk's right.
    fn area(&self, w: Walk) -> Label {
        let chain = &self.chains[w.border];
        if w.reversed { chain.left } else { chain.right }
    }

    /// Where the walk ends, and the direction it arrives in.
    fn end(&self, w: Walk) -> (Vertex, Dir) {
        let chain = &self.chains[w.border];
        let n = chain.vertices.len();
        if w.reversed {
            (chain.vertices[0], chain.first_dir().reverse())
        } else {
            (chain.vertices[n - 1], chain.last_dir())
        }
    }

    /// The cell on the right of the walk's first edge.
    fn first_cell(&self, grid: &Grid, w: Walk) -> Cell {
        let chain = &self.chains[w.border];
        let n = chain.vertices.len();
        let (start, dir) = if w.reversed {
            (chain.vertices[n - 1], chain.last_dir().reverse())
        } else {
            (chain.vertices[0], chain.first_dir())
        };
        grid.sides(start, dir).1
    }

    /// Twice the signed area the walk sweeps (shoelace, y down), so a
    /// clockwise loop on screen sums to a positive number.
    fn swept(&self, w: Walk) -> i64 {
        let v = &self.chains[w.border].vertices;
        let sum: i64 = v
            .windows(2)
            .map(|p| p[0].0 * p[1].1 - p[1].0 * p[0].1)
            .sum();
        if w.reversed { -sum } else { sum }
    }

    /// The walk after `w` around `area`.
    fn next(&self, grid: &Grid, w: Walk, area: Label) -> Option<Walk> {
        let (at, d) = self.end(w);
        let (ahead_left, ahead_right) = grid.side_labels(at, d);
        let turn = if ahead_right != area {
            d.right()
        } else if ahead_left != area {
            d
        } else {
            d.left()
        };
        self.starting.get(&(at, turn)).copied()
    }
}

/// 4-connected areas of non-outside labels: each cell's area number, and
/// each area's label, numbered in row order of their first cell.
fn areas(grid: &Grid) -> (Vec<u32>, Vec<Label>) {
    let (w, h) = (grid.width, grid.height);
    let mut area = vec![u32::MAX; (w * h) as usize];
    let mut labels = Vec::new();
    let mut stack = Vec::new();
    for start in 0..w * h {
        let cell = (start % w, start / w);
        let label = grid.label(cell);
        if label.is_outside() || area[start as usize] != u32::MAX {
            continue;
        }
        let id = labels.len() as u32;
        labels.push(label);
        area[start as usize] = id;
        stack.push(cell);
        while let Some((x, y)) = stack.pop() {
            for n in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                if grid.in_image(n) && grid.label(n) == label {
                    let i = (n.1 * w + n.0) as usize;
                    if area[i] == u32::MAX {
                        area[i] = id;
                        stack.push(n);
                    }
                }
            }
        }
    }
    (area, labels)
}

fn failed(message: &str) -> Error {
    Error::StageFailed {
        stage: Stage::Trace,
        message: message.into(),
    }
}

/// One face per 4-connected area, in row order of their first cell.
pub fn assemble(grid: &Grid, chains: &[Chain]) -> Result<Vec<Face>> {
    let walks = Walks::new(chains);
    let (area_of, area_labels) = areas(grid);
    let mut outers: Vec<Option<Loop>> = vec![None; area_labels.len()];
    let mut holes: Vec<Vec<Loop>> = vec![Vec::new(); area_labels.len()];
    let mut used = vec![[false; 2]; chains.len()];

    for border in 0..chains.len() {
        for reversed in [false, true] {
            let start = Walk { border, reversed };
            let label = walks.area(start);
            if label.is_outside() || used[border][usize::from(reversed)] {
                continue;
            }
            let mut steps = Vec::new();
            let mut swept = 0;
            let mut w = start;
            loop {
                used[w.border][usize::from(w.reversed)] = true;
                steps.push(w.step());
                swept += walks.swept(w);
                w = walks
                    .next(grid, w, label)
                    .ok_or_else(|| failed("a loop ran into a vertex with no border leaving it"))?;
                if w == start {
                    break;
                }
                if steps.len() > 2 * chains.len() {
                    return Err(failed("a loop never closed"));
                }
            }
            let cell = walks.first_cell(grid, start);
            let area = area_of[(cell.1 * grid.width + cell.0) as usize] as usize;
            let lp = Loop { steps };
            if swept > 0 {
                if outers[area].replace(lp).is_some() {
                    return Err(failed("an area has two outer loops"));
                }
            } else {
                holes[area].push(lp);
            }
        }
    }

    area_labels
        .into_iter()
        .zip(outers)
        .zip(holes)
        .map(|((label, outer), holes)| {
            let colour = label
                .output_colour()
                .ok_or_else(|| failed("an area is outside"))?;
            let outer = outer.ok_or_else(|| failed("an area has no outer loop"))?;
            Ok(Face {
                colour,
                outer,
                holes,
            })
        })
        .collect()
}
