//! The thin-feature fix: per colour, a morphological opening (shrink, then
//! regrow) by a disc as wide as the minimum feature.
//!
//! A cell survives when some disc that lies wholly inside its colour covers
//! it, so every part narrower than the disc, in any direction, is removed.
//! Removed cells then take the other colour that most surrounds them,
//! growing in from the edges one ring at a time.

use crate::grid::{Grid, areas, most_common, tally};
use flatart_domain::{Label, LabelMap, PxPoint, ThinFeature};

/// The fix's result.
pub struct Fixed {
    pub labels: Vec<Label>,
    /// Too-thin parts with no other colour beside them, which keep their
    /// colour so the outline doesn't change.
    pub kept_thin: Vec<ThinFeature>,
}

/// The disc's diameter in cells: the fewest whole cells at least `min_mm`
/// wide. A part exactly `min_mm` wide is wide enough.
pub fn diameter(min_mm: f64, cell_mm: f64) -> usize {
    let cells = (min_mm / cell_mm - 1e-9).ceil();
    if cells.is_finite() && cells > 1.0 {
        // Saturates for absurd ratios; `fix` clamps to the grid anyway.
        cells as usize
    } else {
        1
    }
}

pub fn fix(map: &LabelMap, diameter: usize) -> Fixed {
    let grid = Grid::new(map.width(), map.height());
    let labels = map.labels();
    let k = diameter.min(grid.width.max(grid.height) + 1);
    let covered = opening(grid, labels, k);
    let kept: Vec<bool> = (0..grid.len())
        .map(|i| covered[i] || labels[i].is_outside())
        .collect();
    let (out, settled) = reassign(grid, labels, &kept);
    // Cells that kept their colour may be wide enough now that the parts
    // around them changed, so judge them on the result.
    let wide_now = opening(grid, &out, k);
    let thin: Vec<bool> = (0..grid.len())
        .map(|i| !settled[i] && !wide_now[i])
        .collect();
    let kept_thin = report(map, &out, &thin, &wide_now, k);
    Fixed {
        labels: out,
        kept_thin,
    }
}

/// For each row of a `k` × `k` box, the first and last columns inside a
/// disc of diameter `k` (cell centres within the circle).
fn disc_rows(k: usize) -> Vec<(usize, usize)> {
    let r = k as f64 / 2.0;
    (0..k)
        .map(|row| {
            let dy = row as f64 + 0.5 - r;
            let half = (r * r - dy * dy).max(0.0).sqrt() + 1e-9;
            // Columns c with |c + 0.5 - r| <= half; never empty, as the
            // middle column is always inside.
            let first = (r - half - 0.5).ceil().max(0.0) as usize;
            let last = ((r + half - 0.5).floor() as usize).min(k - 1);
            (first, last)
        })
        .collect()
}

/// Whether each cell is covered by a disc of diameter `k` that lies wholly
/// inside one (non-outside) label.
fn opening(grid: Grid, labels: &[Label], k: usize) -> Vec<bool> {
    let (w, h) = (grid.width, grid.height);
    // How many cells from each one rightwards share its label.
    let mut run = vec![0usize; grid.len()];
    for y in 0..h {
        for x in (0..w).rev() {
            let i = y * w + x;
            run[i] = if x + 1 < w && labels[i + 1] == labels[i] {
                run[i + 1] + 1
            } else {
                1
            };
        }
    }
    let rows = disc_rows(k);
    // Per row, +1 where a covering disc's span starts and -1 after it ends.
    let mut diff = vec![0i32; (w + 1) * h];
    if k <= w && k <= h {
        for y0 in 0..=h - k {
            for x0 in 0..=w - k {
                let label = labels[y0 * w + x0 + rows[0].0];
                if label.is_outside() {
                    continue;
                }
                let fits = rows.iter().enumerate().all(|(r, &(a, b))| {
                    let i = (y0 + r) * w + x0 + a;
                    labels[i] == label && run[i] > b - a
                });
                if fits {
                    for (r, &(a, b)) in rows.iter().enumerate() {
                        let row = (y0 + r) * (w + 1);
                        diff[row + x0 + a] += 1;
                        diff[row + x0 + b + 1] -= 1;
                    }
                }
            }
        }
    }
    let mut covered = vec![false; grid.len()];
    for y in 0..h {
        let mut depth = 0;
        for x in 0..w {
            depth += diff[y * (w + 1) + x];
            covered[y * w + x] = depth > 0;
        }
    }
    covered
}

/// Gives each removed cell the other colour that most surrounds it, one ring
/// at a time from the kept cells inwards. Returns the labels and which cells
/// ended up settled (kept or reassigned).
fn reassign(grid: Grid, labels: &[Label], kept: &[bool]) -> (Vec<Label>, Vec<bool>) {
    let mut out = labels.to_vec();
    let mut settled = kept.to_vec();
    let mut pending: Vec<usize> = (0..grid.len()).filter(|&i| !settled[i]).collect();
    let mut counts = Vec::with_capacity(8);
    loop {
        let mut assigned = Vec::new();
        for &i in &pending {
            counts.clear();
            for n in grid.ring(i) {
                let l = out[n];
                if settled[n] && !l.is_outside() && l != labels[i] {
                    tally(&mut counts, l);
                }
            }
            if let Some(l) = most_common(&counts) {
                assigned.push((i, l));
            }
        }
        if assigned.is_empty() {
            return (out, settled);
        }
        for (i, l) in assigned {
            out[i] = l;
            settled[i] = true;
        }
        pending.retain(|&i| !settled[i]);
    }
}

/// The number of distinct values in `values`' range, max − min + 1.
fn span(values: impl Iterator<Item = usize> + Clone) -> usize {
    let max = values.clone().max().unwrap_or(0);
    let min = values.min().unwrap_or(0);
    max - min + 1
}

/// One `ThinFeature` per area of `thin` cells (too thin, and kept their
/// colour).
///
/// Leaves out the few cells a disc can't reach in a convex corner of an
/// otherwise wide part (spanning less than half the disc, touching wide
/// cells of the same colour): those are rounding, not features.
fn report(
    map: &LabelMap,
    out: &[Label],
    thin: &[bool],
    wide: &[bool],
    k: usize,
) -> Vec<ThinFeature> {
    let grid = Grid::new(map.width(), map.height());
    let up = f64::from(map.upscale().get());
    let (thin, _) = areas(grid, out, |i| thin[i]);
    thin.into_iter()
        .filter(|area| {
            let touches_wide = area.cells.iter().any(|&i| {
                grid.edge_neighbours(i)
                    .any(|n| wide[n] && out[n] == area.label)
            });
            let xs = area.cells.iter().map(|&i| i % grid.width);
            let ys = area.cells.iter().map(|&i| i / grid.width);
            !touches_wide || span(xs).max(span(ys)) * 2 > k
        })
        .filter_map(|area| {
            let i = area.cells[0];
            let (x, y) = ((i % grid.width) as f64, (i / grid.width) as f64);
            area.label.output_colour().map(|colour| ThinFeature {
                at: PxPoint::new((x + 0.5) / up, (y + 0.5) / up),
                colour,
            })
        })
        .collect()
}
