//! Weighted k-means with fixed centres.

use crate::histogram::ColourCount;
use crate::lab::{Lab, nearest};

/// Fixed so detection is deterministic. Changing it changes every result.
const SEED: u64 = 0x5EED_F1A7_A27D_E7EC;

/// Lloyd iterations stop here even if assignments still change.
const MAX_ITERATIONS: usize = 100;

/// Clusters `colours` (weighted by pixel count) around the `fixed` centres
/// plus up to `free` centres that move.
///
/// Returns the fixed centres unchanged, in order, followed by the moved ones.
/// Fewer than `free` are added when the colours run out.
pub(crate) fn cluster(colours: &[ColourCount], fixed: &[Lab], free: usize) -> Vec<Lab> {
    let mut centres = seed_centres(colours, fixed, free);
    let mut owner = vec![usize::MAX; colours.len()];
    for _ in 0..MAX_ITERATIONS {
        let mut changed = false;
        for (c, o) in colours.iter().zip(owner.iter_mut()) {
            let n = nearest(c.lab, &centres);
            changed |= *o != n;
            *o = n;
        }
        if !changed {
            break;
        }
        move_to_means(colours, &owner, fixed.len(), &mut centres);
    }
    centres
}

/// k-means++ seeding, weighted by pixel count: each new centre is a colour
/// picked with probability proportional to `pixels × distance²` to the
/// nearest centre so far.
fn seed_centres(colours: &[ColourCount], fixed: &[Lab], free: usize) -> Vec<Lab> {
    let mut centres = fixed.to_vec();
    let mut rng = SplitMix64(SEED);
    let mut d2: Vec<f64> = colours
        .iter()
        .map(|c| match centres.is_empty() {
            true => 1.0,
            false => c.lab.distance2(centres[nearest(c.lab, &centres)]),
        })
        .collect();
    for _ in 0..free {
        let scores: Vec<f64> = colours
            .iter()
            .zip(&d2)
            .map(|(c, d)| c.pixels as f64 * d)
            .collect();
        let total: f64 = scores.iter().sum();
        if total <= 0.0 {
            break; // every colour already sits on a centre
        }
        let Some(pick) = pick_weighted(&scores, rng.next_f64() * total) else {
            break;
        };
        let new = colours[pick].lab;
        centres.push(new);
        for (c, d) in colours.iter().zip(d2.iter_mut()) {
            *d = d.min(c.lab.distance2(new));
        }
    }
    centres
}

/// The first index whose running total passes `target`, skipping zero scores.
fn pick_weighted(scores: &[f64], target: f64) -> Option<usize> {
    let mut running = 0.0;
    let mut last = None;
    for (i, &s) in scores.iter().enumerate().filter(|(_, s)| **s > 0.0) {
        running += s;
        last = Some(i);
        if running > target {
            return last;
        }
    }
    last // rounding left `target` just past the end
}

/// Moves every centre from `first_free` on to the weighted mean of its
/// colours. A centre that lost all its colours stays put.
fn move_to_means(colours: &[ColourCount], owner: &[usize], first_free: usize, centres: &mut [Lab]) {
    let mut sums = vec![([0.0; 3], 0.0); centres.len()];
    for (c, &o) in colours.iter().zip(owner) {
        let w = c.pixels as f64;
        let (sum, weight) = &mut sums[o];
        for (s, v) in sum.iter_mut().zip(c.lab.0) {
            *s += v * w;
        }
        *weight += w;
    }
    for (centre, (sum, weight)) in centres.iter_mut().zip(sums).skip(first_free) {
        if weight > 0.0 {
            *centre = Lab(sum.map(|s| s / weight));
        }
    }
}

/// A small, portable PRNG, so results match on every platform.
struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1).
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn colour(l: f64, pixels: u64) -> ColourCount {
        ColourCount {
            rgb: [0; 3],
            lab: Lab([l, 0.0, 0.0]),
            pixels,
        }
    }

    #[test]
    fn fixed_centres_never_move() {
        let colours = [colour(0.1, 10), colour(0.2, 10), colour(0.9, 10)];
        let fixed = [Lab([0.15, 0.0, 0.0])];
        let centres = cluster(&colours, &fixed, 1);
        assert_eq!(centres[0], fixed[0]);
        assert_eq!(centres.len(), 2);
        assert!((centres[1].0[0] - 0.9).abs() < 1e-12);
    }

    #[test]
    fn means_are_weighted_by_pixel_count() {
        let colours = [colour(0.0, 3), colour(0.4, 1), colour(1.0, 50)];
        let centres = cluster(&colours, &[], 2);
        let mut ls: Vec<f64> = centres.iter().map(|c| c.0[0]).collect();
        ls.sort_by(f64::total_cmp);
        assert!((ls[0] - 0.1).abs() < 1e-12, "got {ls:?}");
        assert!((ls[1] - 1.0).abs() < 1e-12, "got {ls:?}");
    }

    #[test]
    fn stops_adding_centres_when_colours_run_out() {
        let colours = [colour(0.2, 5), colour(0.8, 5)];
        assert_eq!(cluster(&colours, &[], 6).len(), 2);
    }

    #[test]
    fn weighted_pick_skips_zero_scores() {
        assert_eq!(pick_weighted(&[0.0, 2.0, 0.0, 1.0], 0.5), Some(1));
        assert_eq!(pick_weighted(&[0.0, 2.0, 0.0, 1.0], 2.5), Some(3));
        assert_eq!(pick_weighted(&[0.0, 2.0, 0.0, 1.0], 3.0), Some(3));
        assert_eq!(pick_weighted(&[0.0, 0.0], 0.0), None);
    }

    #[test]
    fn rng_is_in_range_and_repeatable() {
        let (mut a, mut b) = (SplitMix64(SEED), SplitMix64(SEED));
        for _ in 0..1000 {
            let x = a.next_f64();
            assert!((0.0..1.0).contains(&x));
            assert_eq!(x.to_bits(), b.next_f64().to_bits());
        }
    }
}
