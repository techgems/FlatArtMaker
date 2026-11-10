//! Cell indexing, neighbours and connected areas on a row-major grid.

use flatart_domain::Label;

/// The size of a row-major grid of cells.
#[derive(Debug, Clone, Copy)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
}

impl Grid {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width: width as usize,
            height: height as usize,
        }
    }

    pub fn len(self) -> usize {
        self.width * self.height
    }

    /// The edge neighbours of cell `i` (4-connectivity).
    pub fn edge_neighbours(self, i: usize) -> impl Iterator<Item = usize> {
        let (x, y) = (i % self.width, i / self.width);
        let w = self.width;
        [
            (x > 0).then(|| i - 1),
            (x + 1 < w).then(|| i + 1),
            (y > 0).then(|| i - w),
            (y + 1 < self.height).then(|| i + w),
        ]
        .into_iter()
        .flatten()
    }

    /// Every cell in the 3 × 3 block around cell `i` except `i`, clipped to
    /// the grid.
    pub fn ring(self, i: usize) -> impl Iterator<Item = usize> {
        let (x, y) = (i % self.width, i / self.width);
        let xs = x.saturating_sub(1)..=(x + 1).min(self.width - 1);
        let ys = y.saturating_sub(1)..=(y + 1).min(self.height - 1);
        ys.flat_map(move |ny| xs.clone().map(move |nx| ny * self.width + nx))
            .filter(move |&n| n != i)
    }
}

/// One connected area of a single label.
#[derive(Debug, Clone)]
pub struct Area {
    pub label: Label,
    /// Its cells, the first in scan order first.
    pub cells: Vec<usize>,
}

/// The connected areas (4-connectivity) of every cell where `include` holds,
/// in scan order of their first cell. Returns the areas and each cell's area
/// index (`usize::MAX` where `include` doesn't hold).
pub fn areas(
    grid: Grid,
    labels: &[Label],
    include: impl Fn(usize) -> bool,
) -> (Vec<Area>, Vec<usize>) {
    let mut area_of = vec![usize::MAX; grid.len()];
    let mut areas = Vec::new();
    let mut stack = Vec::new();
    for start in 0..grid.len() {
        if area_of[start] != usize::MAX || !include(start) {
            continue;
        }
        let (label, id) = (labels[start], areas.len());
        let mut cells = Vec::new();
        area_of[start] = id;
        stack.push(start);
        while let Some(i) = stack.pop() {
            cells.push(i);
            for n in grid.edge_neighbours(i) {
                if area_of[n] == usize::MAX && labels[n] == label && include(n) {
                    area_of[n] = id;
                    stack.push(n);
                }
            }
        }
        cells.sort_unstable();
        areas.push(Area { label, cells });
    }
    (areas, area_of)
}

/// The label among `counts` with the highest count, lowest label on a tie.
pub fn most_common(counts: &[(Label, usize)]) -> Option<Label> {
    counts
        .iter()
        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))
        .map(|&(label, _)| label)
}

/// Adds one to `label`'s count.
pub fn tally(counts: &mut Vec<(Label, usize)>, label: Label) {
    match counts.iter_mut().find(|(l, _)| *l == label) {
        Some((_, n)) => *n += 1,
        None => counts.push((label, 1)),
    }
}
