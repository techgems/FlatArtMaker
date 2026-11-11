//! One face as SVG path data.

use crate::place::{Placement, number};
use flatart_domain::{BorderMap, CubicBezier, Face, Loop};
use std::fmt::Write;

/// The loop's curves in walking order.
fn curves<'a>(map: &'a BorderMap, lp: &'a Loop) -> impl Iterator<Item = CubicBezier> + 'a {
    lp.steps.iter().flat_map(move |step| {
        let curve = map
            .border(step.border)
            .map_or(&[][..], |b| b.curve.as_slice());
        let walked: Vec<CubicBezier> = if step.reversed {
            curve.iter().rev().map(CubicBezier::reversed).collect()
        } else {
            curve.to_vec()
        };
        walked
    })
}

fn point(out: &mut String, place: &Placement, p: flatart_domain::PxPoint) {
    let (x, y) = place.place(p);
    let _ = write!(out, "{} {}", number(x), number(y));
}

/// `M … C … Z` for the outer loop, then each hole. With `fill-rule=evenodd`
/// the holes are cut out whichever way they run.
pub fn face(map: &BorderMap, face: &Face, place: &Placement) -> String {
    let mut d = String::new();
    for lp in std::iter::once(&face.outer).chain(&face.holes) {
        let mut first = true;
        for c in curves(map, lp) {
            if first {
                d.push('M');
                point(&mut d, place, c.from);
                first = false;
            }
            d.push('C');
            point(&mut d, place, c.ctrl1);
            d.push(' ');
            point(&mut d, place, c.ctrl2);
            d.push(' ');
            point(&mut d, place, c.to);
        }
        if !first {
            d.push('Z');
        }
    }
    d
}
