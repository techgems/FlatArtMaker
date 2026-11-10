//! The default base colour: the one most common along the art's outer edge.
//!
//! For outlined art that's the outline colour, and for art on a solid
//! background it's the background, so the base reads as part of the design.

use flatart_domain::{Label, LabelMap, OutputColourId};

/// The output colour with the most cells next to outside (or the image
/// border), from `slots`. Ties and no edge cells go to the earlier slot.
/// `None` only when `slots` is empty.
pub(crate) fn edge_majority(labels: &LabelMap, slots: &[OutputColourId]) -> Option<OutputColourId> {
    let mut counts = vec![0u64; slots.len()];
    let (w, h) = (labels.width(), labels.height());
    let outside = |x: Option<u32>, y: Option<u32>| match (x, y) {
        (Some(x), Some(y)) => labels.get(x, y).is_none_or(Label::is_outside),
        _ => true,
    };
    for y in 0..h {
        for x in 0..w {
            let Some(id) = labels.get(x, y).and_then(Label::output_colour) else {
                continue;
            };
            let on_edge = outside(x.checked_sub(1), Some(y))
                || outside(x.checked_add(1), Some(y))
                || outside(Some(x), y.checked_sub(1))
                || outside(Some(x), y.checked_add(1));
            if let (true, Some(slot)) = (on_edge, slots.iter().position(|&s| s == id)) {
                counts[slot] += 1;
            }
        }
    }
    let best = counts
        .iter()
        .enumerate()
        .max_by_key(|&(slot, &count)| (count, std::cmp::Reverse(slot)))?;
    Some(slots[best.0])
}

#[cfg(test)]
mod tests {
    use super::*;
    use flatart_ports::testkit::fixtures::labels;

    fn ids(n: u16) -> Vec<OutputColourId> {
        (0..n).map(OutputColourId).collect()
    }

    #[test]
    fn picks_the_outline_colour() {
        // Colour 1 outlines colour 0, which covers more cells.
        let map = labels(&["11111", "10001", "10001", "10001", "11111"], 1);
        assert_eq!(edge_majority(&map, &ids(2)), Some(OutputColourId(1)));
    }

    #[test]
    fn counts_cells_next_to_outside_inside_the_image() {
        // A hole of outside makes colour 0 an edge colour too.
        let map = labels(&["1111111", "1000001", "100.001", "1000001", "1111111"], 1);
        assert_eq!(edge_majority(&map, &ids(2)), Some(OutputColourId(1)));
        let map = labels(&["..0..", ".000.", "00000"], 1);
        assert_eq!(edge_majority(&map, &ids(2)), Some(OutputColourId(0)));
    }

    #[test]
    fn ties_and_unused_colours_go_to_the_earlier_slot() {
        let map = labels(&["01"], 1);
        let slots = [OutputColourId(1), OutputColourId(0)];
        assert_eq!(edge_majority(&map, &slots), Some(OutputColourId(1)));
        assert_eq!(
            edge_majority(&labels(&["..."], 1), &ids(3)),
            Some(OutputColourId(0))
        );
        assert_eq!(edge_majority(&map, &[]), None);
    }
}
