//! The automatic first pass: merge the most similar detected colours down
//! to the user's colour count, recording the full merge order.

use crate::colour_space::{Lab, delta_e, lab, to_rgb};
use flatart_domain::settings::MappingSettings;
use flatart_domain::{
    DeltaE, DetectedPalette, GroupId, Label, Merge, OutputColourId, Result, Rgb, SimilarityTree,
};

/// One automatic output colour.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct AutoOutput {
    pub id: OutputColourId,
    pub display: Rgb,
}

/// The automatic mapping, before any edit.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Auto {
    /// In slot order; output *n* has `OutputColourId(n)`.
    pub outputs: Vec<AutoOutput>,
    /// The output for each detected colour.
    pub targets: Vec<OutputColourId>,
    pub similarity: SimilarityTree,
}

pub(crate) fn auto_map(detected: &DetectedPalette, settings: &MappingSettings) -> Result<Auto> {
    let similarity = similarity_tree(detected)?;
    let groups = similarity.groups(usize::from(settings.target.get()));
    let mut groups: Vec<Summary> = groups
        .iter()
        .map(|members| Summary::of(detected, members))
        .collect();
    // Slot order: most pixels first; ties keep detection order.
    groups.sort_by_key(|g| (std::cmp::Reverse(g.pixels), g.members[0]));

    let mut targets = vec![OutputColourId(0); detected.colours().len()];
    let outputs = groups
        .iter()
        .enumerate()
        .map(|(slot, group)| {
            // At most 16 groups, so the slot fits in a u16.
            let id = OutputColourId(slot as u16);
            for &m in &group.members {
                targets[usize::from(m.0)] = id;
            }
            AutoOutput {
                id,
                display: to_rgb(group.colour),
            }
        })
        .collect();
    Ok(Auto {
        outputs,
        targets,
        similarity,
    })
}

/// A cut of the tree: one output colour's detected colours.
struct Summary {
    members: Vec<Label>,
    colour: Lab,
    pixels: u64,
}

impl Summary {
    fn of(detected: &DetectedPalette, members: &[Label]) -> Self {
        let colours: Vec<(Lab, u64)> = members
            .iter()
            .map(|l| {
                let c = &detected.colours()[usize::from(l.0)];
                (lab(c.colour), c.pixels)
            })
            .collect();
        Self {
            members: members.to_vec(),
            colour: weighted_mean(&colours),
            pixels: colours.iter().map(|(_, p)| p).sum(),
        }
    }
}

/// The pixel-weighted mean, or the plain mean when nothing has pixels.
fn weighted_mean(colours: &[(Lab, u64)]) -> Lab {
    let total: u64 = colours.iter().map(|(_, p)| p).sum();
    let weight = |p: u64| if total == 0 { 1.0 } else { p as f64 };
    let sum_w: f64 = colours.iter().map(|&(_, p)| weight(p)).sum();
    let mut mean = [0.0; 3];
    for &(c, p) in colours {
        for (m, v) in mean.iter_mut().zip(c) {
            *m += v * weight(p) / sum_w;
        }
    }
    mean
}

/// A group while building the tree.
struct Node {
    id: GroupId,
    colour: Lab,
    pixels: u64,
}

/// Merges the closest pair (by pixel-weighted mean colour) until one group
/// remains. Ties go to the first pair in group order.
fn similarity_tree(detected: &DetectedPalette) -> Result<SimilarityTree> {
    let n = detected.colours().len();
    let mut nodes: Vec<Node> = detected
        .colours()
        .iter()
        .enumerate()
        .map(|(i, c)| Node {
            id: GroupId(i as u16),
            colour: lab(c.colour),
            pixels: c.pixels,
        })
        .collect();
    let mut merges = Vec::with_capacity(n.saturating_sub(1));
    while let Some((i, j, distance)) = closest_pair(&nodes) {
        let b = nodes.remove(j);
        let a = &mut nodes[i];
        merges.push(Merge {
            joined: [a.id, b.id],
            distance: DeltaE(distance),
        });
        a.colour = weighted_mean(&[(a.colour, a.pixels), (b.colour, b.pixels)]);
        a.pixels += b.pixels;
        // `SimilarityTree::new` rejects counts that don't fit a u16.
        a.id = GroupId((n + merges.len() - 1) as u16);
    }
    SimilarityTree::new(n, merges)
}

/// The two closest nodes as `(i, j, ΔE)` with `i < j`; ties go to the
/// first pair found. `None` with fewer than two nodes.
fn closest_pair(nodes: &[Node]) -> Option<(usize, usize, f64)> {
    let mut best: Option<(usize, usize, f64)> = None;
    for i in 0..nodes.len() {
        for j in i + 1..nodes.len() {
            let d = delta_e(nodes[i].colour, nodes[j].colour);
            if best.is_none_or(|(_, _, b)| d < b) {
                best = Some((i, j, d));
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use flatart_domain::{DetectedColour, Oklab};
    use flatart_ports::testkit::fixtures::mapping_settings;

    fn palette(colours: &[(f64, u64)]) -> DetectedPalette {
        let colours = colours
            .iter()
            .map(|&(l, pixels)| DetectedColour {
                colour: Oklab::new(l, 0.0, 0.0).unwrap(),
                pixels,
            })
            .collect();
        DetectedPalette::new(colours).unwrap()
    }

    fn ids(auto: &Auto) -> Vec<u16> {
        auto.targets.iter().map(|t| t.0).collect()
    }

    fn auto(p: &DetectedPalette, target: u8) -> Auto {
        auto_map(p, &mapping_settings(target)).unwrap()
    }

    #[test]
    fn merges_the_most_similar_tones_first() {
        // Two purples 3 ΔE apart, and a far-off yellow.
        let p = palette(&[(0.40, 10), (0.43, 10), (0.90, 5)]);
        assert_eq!(ids(&auto(&p, 3)), [0, 1, 2], "3 wanted: nothing merges");
        assert_eq!(ids(&auto(&p, 2)), [0, 0, 1], "2 wanted: the tones merge");
        assert_eq!(ids(&auto(&p, 1)), [0, 0, 0]);
    }

    #[test]
    fn records_every_merge_with_its_distance() {
        let p = palette(&[(0.40, 10), (0.43, 10), (0.90, 5)]);
        let tree = auto(&p, 3).similarity;
        let steps: Vec<([u16; 2], f64)> = tree
            .merges()
            .iter()
            .map(|m| (m.joined.map(|g| g.0), m.distance.0))
            .collect();
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].0, [0, 1]);
        assert!((steps[0].1 - 3.0).abs() < 1e-9);
        assert_eq!(steps[1].0, [3, 2], "group 3 is the merged purples");
        assert!((steps[1].1 - 48.5).abs() < 1e-9, "from their mean, 0.415");
    }

    #[test]
    fn the_merged_colour_is_weighted_by_pixels() {
        // Black with 3× the pixels of a grey pulls the merged colour dark.
        let p = palette(&[(0.0, 30), (0.4, 10)]);
        let merged = crate::colour_space::from_rgb(auto(&p, 1).outputs[0].display)[0];
        assert!((merged - 0.1).abs() < 0.01, "got L = {merged}");
    }

    #[test]
    fn slots_are_ordered_by_pixel_count() {
        let p = palette(&[(0.1, 1), (0.5, 9), (0.9, 5)]);
        assert_eq!(ids(&auto(&p, 8)), [2, 0, 1]);
    }

    #[test]
    fn colours_without_pixels_still_get_a_target() {
        let p = palette(&[(0.3, 0), (0.31, 0)]);
        assert_eq!(ids(&auto(&p, 1)), [0, 0]);
    }

    #[test]
    fn no_detected_colours_gives_no_outputs() {
        let a = auto(&DetectedPalette::default(), 8);
        assert!(a.outputs.is_empty() && a.targets.is_empty());
        assert!(a.similarity.merges().is_empty());
    }
}
