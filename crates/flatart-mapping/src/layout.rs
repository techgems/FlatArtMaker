//! Where each detected colour sits in the image: how many of its pixels are
//! interior, and how long a border it shares with each other colour.
//!
//! A detected colour with no interior pixel is an *edge tone*: an in-between
//! colour that edge smoothing (anti-aliasing) makes along a border.

use flatart_domain::LabelMap;

/// Per detected colour (indexed by label).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Layout {
    /// Pixels whose 8 neighbours all have the same colour (and none is
    /// outside or off the image).
    pub interior: Vec<u64>,
    /// `borders[a][b]`: pixel edges where colour `a` meets colour `b`
    /// (4-connected). Symmetric; the diagonal is 0.
    pub borders: Vec<Vec<u64>>,
}

impl Layout {
    /// `colours` is the detected palette's size; labels at or beyond it
    /// (outside included) count as no colour.
    pub fn of(labels: &LabelMap, colours: usize) -> Self {
        let (w, h) = (labels.width() as usize, labels.height() as usize);
        let cells = labels.labels();
        let at = |x: usize, y: usize| -> Option<usize> {
            let l = usize::from(cells[y * w + x].0);
            (l < colours).then_some(l)
        };
        let mut interior = vec![0; colours];
        let mut borders = vec![vec![0; colours]; colours];
        for y in 0..h {
            for x in 0..w {
                let Some(c) = at(x, y) else { continue };
                if x + 1 < w
                    && let Some(r) = at(x + 1, y)
                    && r != c
                {
                    borders[c][r] += 1;
                    borders[r][c] += 1;
                }
                if y + 1 < h
                    && let Some(d) = at(x, y + 1)
                    && d != c
                {
                    borders[c][d] += 1;
                    borders[d][c] += 1;
                }
                let inside = x > 0 && y > 0 && x + 1 < w && y + 1 < h;
                if inside
                    && (y - 1..=y + 1).all(|ny| (x - 1..=x + 1).all(|nx| at(nx, ny) == Some(c)))
                {
                    interior[c] += 1;
                }
            }
        }
        Self { interior, borders }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flatart_domain::Label;
    use flatart_domain::settings::TraceUpscale;

    fn map(rows: &[&str]) -> LabelMap {
        let cells = rows
            .iter()
            .flat_map(|r| r.chars())
            .map(|c| match c {
                '.' => Label::OUTSIDE,
                d => Label(d.to_digit(10).unwrap() as u16),
            })
            .collect();
        LabelMap::new(
            rows[0].len() as u32,
            rows.len() as u32,
            TraceUpscale::new(1).unwrap(),
            cells,
        )
        .unwrap()
    }

    #[test]
    fn a_one_pixel_band_between_two_areas_is_an_edge_tone() {
        let m = map(&[
            "00000", "00000", "00000", "11111", "22222", "22222", "22222",
        ]);
        let l = Layout::of(&m, 3);
        assert_eq!(l.interior, [3, 0, 3]);
        assert_eq!(l.borders[1], [5, 0, 5]);
        assert_eq!(l.borders[0][2], 0);
    }

    #[test]
    fn outside_and_the_image_edge_are_not_interior() {
        let m = map(&["000", "000", "000"]);
        assert_eq!(Layout::of(&m, 1).interior, [1]);
        let m = map(&["000", "000", "00."]);
        assert_eq!(Layout::of(&m, 1).interior, [0]);
    }
}
