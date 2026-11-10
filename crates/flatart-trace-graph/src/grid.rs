//! The cell grid and the lattice of vertices between cells.
//!
//! Vertex (x, y) is the top-left corner of cell (x, y), so vertices run from
//! 0 to width and 0 to height. A pixel edge joins two neighbouring vertices
//! and exists where the cells on its two sides have different labels.
//! Everything beyond the image counts as outside.

use flatart_domain::{Label, LabelMap};

/// A direction on screen (y down).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dir {
    Right,
    Down,
    Left,
    Up,
}

impl Dir {
    pub const ALL: [Dir; 4] = [Dir::Right, Dir::Down, Dir::Left, Dir::Up];

    pub fn delta(self) -> (i64, i64) {
        match self {
            Dir::Right => (1, 0),
            Dir::Down => (0, 1),
            Dir::Left => (-1, 0),
            Dir::Up => (0, -1),
        }
    }

    /// A quarter turn clockwise on screen.
    pub fn right(self) -> Dir {
        match self {
            Dir::Right => Dir::Down,
            Dir::Down => Dir::Left,
            Dir::Left => Dir::Up,
            Dir::Up => Dir::Right,
        }
    }

    pub fn left(self) -> Dir {
        self.right().right().right()
    }

    pub fn reverse(self) -> Dir {
        self.right().right()
    }

    /// The direction from `a` to the neighbouring vertex `b`.
    pub fn between(a: Vertex, b: Vertex) -> Dir {
        match (b.0 - a.0, b.1 - a.1) {
            (1, 0) => Dir::Right,
            (0, 1) => Dir::Down,
            (-1, 0) => Dir::Left,
            _ => Dir::Up,
        }
    }
}

/// A vertex of the lattice.
pub type Vertex = (i64, i64);

/// A cell's position.
pub type Cell = (i64, i64);

pub fn step(v: Vertex, d: Dir) -> Vertex {
    let (dx, dy) = d.delta();
    (v.0 + dx, v.1 + dy)
}

pub struct Grid<'a> {
    pub width: i64,
    pub height: i64,
    labels: &'a [Label],
}

impl<'a> Grid<'a> {
    pub fn new(map: &'a LabelMap) -> Self {
        Self {
            width: i64::from(map.width()),
            height: i64::from(map.height()),
            labels: map.labels(),
        }
    }

    pub fn in_image(&self, c: Cell) -> bool {
        (0..self.width).contains(&c.0) && (0..self.height).contains(&c.1)
    }

    /// The cell's label; outside beyond the image.
    pub fn label(&self, c: Cell) -> Label {
        if self.in_image(c) {
            self.labels[(c.1 * self.width + c.0) as usize]
        } else {
            Label::OUTSIDE
        }
    }

    pub fn vertex_index(&self, v: Vertex) -> usize {
        (v.1 * (self.width + 1) + v.0) as usize
    }

    fn has_vertex(&self, v: Vertex) -> bool {
        (0..=self.width).contains(&v.0) && (0..=self.height).contains(&v.1)
    }

    /// The cells on the left and right of the edge leaving `v` along `d`.
    pub fn sides(&self, v: Vertex, d: Dir) -> (Cell, Cell) {
        let (x, y) = v;
        match d {
            Dir::Right => ((x, y - 1), (x, y)),
            Dir::Down => ((x, y), (x - 1, y)),
            Dir::Left => ((x - 1, y), (x - 1, y - 1)),
            Dir::Up => ((x - 1, y - 1), (x, y - 1)),
        }
    }

    /// The labels on the left and right of the edge leaving `v` along `d`.
    pub fn side_labels(&self, v: Vertex, d: Dir) -> (Label, Label) {
        let (left, right) = self.sides(v, d);
        (self.label(left), self.label(right))
    }

    /// Whether a pixel edge leaves `v` along `d`.
    pub fn has_edge(&self, v: Vertex, d: Dir) -> bool {
        if !self.has_vertex(v) || !self.has_vertex(step(v, d)) {
            return false;
        }
        let (left, right) = self.side_labels(v, d);
        left != right
    }

    pub fn degree(&self, v: Vertex) -> usize {
        Dir::ALL.iter().filter(|&&d| self.has_edge(v, d)).count()
    }

    pub fn edge_count(&self) -> usize {
        let (w, h) = (self.width, self.height);
        (w * (h + 1) + (w + 1) * h) as usize
    }

    /// The same number for an edge walked either way.
    pub fn edge_index(&self, v: Vertex, d: Dir) -> usize {
        let (w, h) = (self.width, self.height);
        match d {
            Dir::Right => (v.1 * w + v.0) as usize,
            Dir::Left => (v.1 * w + v.0 - 1) as usize,
            Dir::Down => (w * (h + 1) + v.1 * (w + 1) + v.0) as usize,
            Dir::Up => (w * (h + 1) + (v.1 - 1) * (w + 1) + v.0) as usize,
        }
    }
}
