//! How similar the detected colours are: the order the automatic mapping
//! merges them in. Edge tones (colours only found along borders) fold into
//! a neighbour first, then the closest pair merges, down to one group.
//!
//! The colour-mapping editor uses it to show which tones belong together and
//! what would merge next if the user asks for fewer colours.

use crate::{DeltaE, Error, Label, Result};
use serde::Serialize;

/// A group of detected colours. `0..n` are the `n` detected colours
/// themselves (the same numbers as their labels); `n + i` is the group
/// formed by merge `i`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[serde(transparent)]
pub struct GroupId(pub u16);

/// One step: two groups become one.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
pub struct Merge {
    pub joined: [GroupId; 2],
    /// How far apart the two groups' colours were (pixel-weighted means).
    pub distance: DeltaE,
}

/// The full merge order over the detected colours.
///
/// The first [`edge_tones`](Self::edge_tones) merges are edge-tone folds:
/// an in-between colour that edge smoothing made joins a neighbour. Folds
/// are always applied, so they never count as colours.
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
#[cfg_attr(feature = "tsify", derive(tsify::Tsify))]
#[serde(rename_all = "camelCase")]
pub struct SimilarityTree {
    colours: usize,
    edge_tones: usize,
    merges: Vec<Merge>,
}

impl SimilarityTree {
    /// `edge_tones` is how many of the first merges are edge-tone folds.
    ///
    /// Fails unless there are exactly `colours - 1` merges (none for zero
    /// colours), each joins two groups that exist and haven't been merged
    /// yet, and fewer folds than colours (a fold needs a colour to join).
    pub fn new(colours: usize, edge_tones: usize, merges: Vec<Merge>) -> Result<Self> {
        let invalid = |reason: String| Error::InvalidValue {
            what: "similarity tree",
            reason,
        };
        if merges.len() != colours.saturating_sub(1) {
            return Err(invalid(format!(
                "{} merges for {colours} colours",
                merges.len()
            )));
        }
        if edge_tones > 0 && edge_tones >= colours {
            return Err(invalid(format!(
                "{edge_tones} edge tones for {colours} colours"
            )));
        }
        if colours + merges.len() > usize::from(u16::MAX) {
            return Err(invalid(format!("{colours} colours is too many")));
        }
        let mut used = vec![false; colours + merges.len()];
        for (i, merge) in merges.iter().enumerate() {
            let [a, b] = merge.joined.map(|g| usize::from(g.0));
            let exists = |g: usize| g < colours + i;
            if a == b || !exists(a) || !exists(b) || used[a] || used[b] {
                return Err(invalid(format!("merge {i} joins invalid groups")));
            }
            used[a] = true;
            used[b] = true;
        }
        Ok(Self {
            colours,
            edge_tones,
            merges,
        })
    }

    /// How many detected colours the tree covers.
    pub fn colours(&self) -> usize {
        self.colours
    }

    /// How many detected colours are edge tones: the first this many merges
    /// fold them into a neighbour.
    pub fn edge_tones(&self) -> usize {
        self.edge_tones
    }

    /// In order: the edge-tone folds, then closest pair first.
    pub fn merges(&self) -> &[Merge] {
        &self.merges
    }

    /// The groups left after merging down to `count` (at least 1), as
    /// detected labels. Edge tones are always folded, so fewer than `count`
    /// groups remain when fewer colours aren't edge tones. Groups are
    /// ordered by their lowest label, and labels within a group ascend.
    pub fn groups(&self, count: usize) -> Vec<Vec<Label>> {
        let steps = self
            .colours
            .saturating_sub(count.max(1))
            .max(self.edge_tones);
        let mut members: Vec<Vec<Label>> =
            (0..self.colours).map(|i| vec![Label(i as u16)]).collect();
        for merge in &self.merges[..steps] {
            let [a, b] = merge
                .joined
                .map(|g| std::mem::take(&mut members[usize::from(g.0)]));
            let mut joined = [a, b].concat();
            joined.sort_unstable();
            members.push(joined);
        }
        let mut groups: Vec<Vec<Label>> = members.into_iter().filter(|m| !m.is_empty()).collect();
        groups.sort_unstable_by_key(|g| g[0]);
        groups
    }

    /// The merge that takes `count` groups down to `count - 1`, if any.
    /// `None` while `count` is above the colours that aren't edge tones,
    /// since lowering it there changes nothing.
    pub fn next_merge(&self, count: usize) -> Option<&Merge> {
        let step = self.colours.checked_sub(count)?;
        if step < self.edge_tones {
            return None;
        }
        self.merges.get(step)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn merge(a: u16, b: u16, distance: f64) -> Merge {
        Merge {
            joined: [GroupId(a), GroupId(b)],
            distance: DeltaE(distance),
        }
    }

    /// 0 and 2 join first (group 4), then 1 and 3 (group 5), then both.
    fn tree() -> SimilarityTree {
        SimilarityTree::new(
            4,
            0,
            vec![merge(0, 2, 1.0), merge(1, 3, 5.0), merge(4, 5, 30.0)],
        )
        .unwrap()
    }

    fn labels(groups: Vec<Vec<Label>>) -> Vec<Vec<u16>> {
        groups
            .into_iter()
            .map(|g| g.into_iter().map(|l| l.0).collect())
            .collect()
    }

    #[test]
    fn cuts_at_any_count() {
        let t = tree();
        assert_eq!(labels(t.groups(4)), [[0].as_slice(), &[1], &[2], &[3]]);
        assert_eq!(labels(t.groups(3)), [[0, 2].as_slice(), &[1], &[3]]);
        assert_eq!(labels(t.groups(2)), [[0, 2], [1, 3]]);
        assert_eq!(labels(t.groups(1)), [[0, 1, 2, 3]]);
        assert_eq!(labels(t.groups(0)), [[0, 1, 2, 3]], "at least one group");
        assert_eq!(labels(t.groups(9)).len(), 4);
    }

    #[test]
    fn next_merge_is_the_one_a_lower_count_adds() {
        let t = tree();
        assert_eq!(t.next_merge(4), Some(&merge(0, 2, 1.0)));
        assert_eq!(t.next_merge(2), Some(&merge(4, 5, 30.0)));
        assert_eq!(t.next_merge(1), None);
        assert_eq!(t.next_merge(9), None);
    }

    #[test]
    fn rejects_malformed_trees() {
        assert!(SimilarityTree::new(0, 0, vec![]).is_ok());
        assert!(SimilarityTree::new(1, 0, vec![]).is_ok());
        assert!(SimilarityTree::new(2, 0, vec![]).is_err(), "too few merges");
        assert!(SimilarityTree::new(2, 0, vec![merge(0, 0, 1.0)]).is_err());
        assert!(SimilarityTree::new(2, 0, vec![merge(0, 2, 1.0)]).is_err());
        let reused = vec![merge(0, 1, 1.0), merge(0, 2, 1.0)];
        assert!(SimilarityTree::new(3, 0, reused).is_err());
        let two = || vec![merge(0, 1, 1.0)];
        assert!(SimilarityTree::new(2, 1, two()).is_ok());
        assert!(
            SimilarityTree::new(2, 2, two()).is_err(),
            "a fold needs a colour to join"
        );
    }

    /// Like `tree()`, but 2 and 3 are edge tones folded into 0 and 1.
    fn folded() -> SimilarityTree {
        SimilarityTree::new(
            4,
            2,
            vec![merge(0, 2, 40.0), merge(1, 3, 35.0), merge(4, 5, 30.0)],
        )
        .unwrap()
    }

    #[test]
    fn edge_tones_are_always_folded() {
        let t = folded();
        assert_eq!(t.edge_tones(), 2);
        assert_eq!(labels(t.groups(9)), [[0, 2], [1, 3]]);
        assert_eq!(labels(t.groups(3)), [[0, 2], [1, 3]]);
        assert_eq!(labels(t.groups(1)), [[0, 1, 2, 3]]);
    }

    #[test]
    fn folds_are_never_previewed_as_the_next_merge() {
        let t = folded();
        assert_eq!(t.next_merge(4), None);
        assert_eq!(t.next_merge(3), None);
        assert_eq!(t.next_merge(2), Some(&merge(4, 5, 30.0)));
        assert_eq!(t.next_merge(1), None);
    }
}
