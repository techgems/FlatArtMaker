//! Stage 4: the mode filter, then speckle merging.

use crate::grid::{Grid, areas, most_common, tally};
use flatart_domain::settings::CleanUpSettings;
use flatart_domain::{Label, LabelMap, Result};

pub fn clean_up(map: &LabelMap, settings: &CleanUpSettings) -> Result<LabelMap> {
    let grid = Grid::new(map.width(), map.height());
    let mut labels = mode_filter(grid, map.labels());
    merge_speckles(grid, &mut labels, settings.speckle_area.get() as usize);
    LabelMap::new(map.width(), map.height(), map.upscale(), labels)
}

/// Each inside cell takes the most common inside label in its 3 × 3 block,
/// keeping its own on a tie. Removes the stray in-between colours that edge
/// smoothing leaves.
pub fn mode_filter(grid: Grid, labels: &[Label]) -> Vec<Label> {
    let mut counts = Vec::with_capacity(9);
    (0..grid.len())
        .map(|i| {
            let own = labels[i];
            if own.is_outside() {
                return own;
            }
            counts.clear();
            tally(&mut counts, own);
            for n in grid.ring(i) {
                if !labels[n].is_outside() {
                    tally(&mut counts, labels[n]);
                }
            }
            let own_count = counts[0].1;
            match most_common(&counts) {
                Some(best) if counts.iter().any(|&(l, n)| l == best && n > own_count) => best,
                _ => own,
            }
        })
        .collect()
}

/// Merges each connected area smaller than `min_cells` into the neighbouring
/// area it shares the longest border with, smallest first. Areas that border
/// only the outside are kept.
pub fn merge_speckles(grid: Grid, labels: &mut [Label], min_cells: usize) {
    if min_cells == 0 {
        return;
    }
    let (mut areas, area_of) = areas(grid, labels, |i| !labels[i].is_outside());
    let mut parent: Vec<usize> = (0..areas.len()).collect();
    let mut order: Vec<usize> = (0..areas.len())
        .filter(|&a| areas[a].cells.len() < min_cells)
        .collect();
    order.sort_by_key(|&a| (areas[a].cells.len(), areas[a].cells[0]));

    let mut borders: Vec<(usize, usize)> = Vec::new();
    for small in order {
        if parent[small] != small || areas[small].cells.len() >= min_cells {
            continue;
        }
        borders.clear();
        for &i in &areas[small].cells {
            for n in grid.edge_neighbours(i) {
                if area_of[n] == usize::MAX {
                    continue;
                }
                let other = root(&mut parent, area_of[n]);
                if other == small {
                    continue;
                }
                match borders.iter_mut().find(|(a, _)| *a == other) {
                    Some((_, len)) => *len += 1,
                    None => borders.push((other, 1)),
                }
            }
        }
        // Longest border; ties go to the lower label, then the earlier area.
        let Some(&(target, _)) = borders.iter().max_by(|a, b| {
            a.1.cmp(&b.1)
                .then(areas[b.0].label.cmp(&areas[a.0].label))
                .then(b.0.cmp(&a.0))
        }) else {
            continue;
        };
        let label = areas[target].label;
        // Areas of the target's label that touched `small` now join it too.
        let joined: Vec<usize> = borders
            .iter()
            .map(|&(a, _)| a)
            .filter(|&a| a != target && areas[a].label == label)
            .chain([small])
            .collect();
        for a in joined {
            parent[a] = target;
            let cells = std::mem::take(&mut areas[a].cells);
            for &i in &cells {
                labels[i] = label;
            }
            areas[target].cells.extend(cells);
        }
    }
}

fn root(parent: &mut [usize], mut a: usize) -> usize {
    while parent[a] != a {
        parent[a] = parent[parent[a]];
        a = parent[a];
    }
    a
}
