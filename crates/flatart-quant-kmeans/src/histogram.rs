//! The distinct opaque colours of an image and how many pixels have each.

use crate::lab::Lab;
use flatart_domain::RgbaImage;
use std::collections::HashMap;

/// One distinct colour.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ColourCount {
    pub rgb: [u8; 3],
    pub lab: Lab,
    pub pixels: u64,
}

/// Every distinct RGB among pixels with non-zero alpha, sorted by RGB so the
/// order (and every sum over it) is the same on every run and platform.
pub(crate) fn histogram(image: &RgbaImage) -> Vec<ColourCount> {
    let mut counts: HashMap<[u8; 3], u64> = HashMap::new();
    for p in image.pixels().iter().filter(|p| p.a != 0) {
        *counts.entry([p.r, p.g, p.b]).or_default() += 1;
    }
    let mut colours: Vec<ColourCount> = counts
        .into_iter()
        .map(|(rgb, pixels)| ColourCount {
            rgb,
            lab: Lab::from_srgb(rgb),
            pixels,
        })
        .collect();
    colours.sort_unstable_by_key(|c| c.rgb);
    colours
}

#[cfg(test)]
mod tests {
    use super::*;
    use flatart_domain::Rgba;

    #[test]
    fn counts_opaque_colours_in_rgb_order() {
        let px = |r, a| Rgba { r, g: 0, b: 0, a };
        let image = RgbaImage::new(4, 1, vec![px(9, 255), px(1, 1), px(9, 128), px(5, 0)]).unwrap();
        let found: Vec<_> = histogram(&image)
            .iter()
            .map(|c| (c.rgb[0], c.pixels))
            .collect();
        assert_eq!(found, [(1, 1), (9, 2)]);
    }
}
