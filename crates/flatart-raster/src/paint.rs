//! Stage 5: fills the connected area under each paint point.

use crate::grid::Grid;
use flatart_domain::{Label, LabelMap, Paint, PxPoint, Result};

pub fn paint(map: &LabelMap, paints: &[Paint]) -> Result<LabelMap> {
    let grid = Grid::new(map.width(), map.height());
    let mut labels = map.labels().to_vec();
    let up = f64::from(map.upscale().get());
    for p in paints {
        if let Some(start) = cell_at(grid, p.at, up) {
            fill(grid, &mut labels, start, Label::from(p.colour));
        }
    }
    LabelMap::new(map.width(), map.height(), map.upscale(), labels)
}

/// The cell under a point in source pixels, if it's on the grid.
fn cell_at(grid: Grid, at: PxPoint, up: f64) -> Option<usize> {
    let (x, y) = ((at.x.0 * up).floor(), (at.y.0 * up).floor());
    let on_grid = |v: f64, size: usize| v.is_finite() && v >= 0.0 && v < size as f64;
    (on_grid(x, grid.width) && on_grid(y, grid.height))
        .then(|| y as usize * grid.width + x as usize)
}

/// Flood-fills the 4-connected area of `start`'s label. Outside never fills.
fn fill(grid: Grid, labels: &mut [Label], start: usize, colour: Label) {
    let old = labels[start];
    if old.is_outside() || old == colour {
        return;
    }
    labels[start] = colour;
    let mut stack = vec![start];
    while let Some(i) = stack.pop() {
        for n in grid.edge_neighbours(i) {
            if labels[n] == old {
                labels[n] = colour;
                stack.push(n);
            }
        }
    }
}
