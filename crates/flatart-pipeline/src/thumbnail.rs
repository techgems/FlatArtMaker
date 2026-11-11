//! A top-down picture of the art, for formats that embed one (the 3MF's
//! `Metadata/thumbnail.png`).

use flatart_domain::{ArtBounds, LabelMap, OutputPalette, Result, Rgba, RgbaImage};

/// Each cell of `labels` in its output colour, outside transparent, cropped
/// to `bounds` (in source pixels) and scaled down (nearest cell) so neither
/// side exceeds `max_side`.
pub fn render(
    labels: &LabelMap,
    bounds: ArtBounds,
    palette: &OutputPalette,
    max_side: u32,
) -> Result<RgbaImage> {
    let up = u32::from(labels.upscale().get());
    let (w, h) = (bounds.width.max(1), bounds.height.max(1));
    let factor = (f64::from(max_side.max(1)) / f64::from(w.max(h))).min(1.0);
    let size = |side: u32| ((f64::from(side) * factor).round() as u32).max(1);
    let (out_w, out_h) = (size(w), size(h));
    let clear = Rgba {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
    let mut pixels = Vec::with_capacity((out_w * out_h) as usize);
    for y in 0..out_h {
        for x in 0..out_w {
            // The source pixel under this pixel's centre, then its cell.
            let sx = bounds.x + ((f64::from(x) + 0.5) / factor) as u32;
            let sy = bounds.y + ((f64::from(y) + 0.5) / factor) as u32;
            let colour = labels
                .get(sx * up + up / 2, sy * up + up / 2)
                .and_then(|l| l.output_colour())
                .and_then(|id| palette.get(id))
                .map_or(clear, |c| Rgba {
                    r: c.display.r,
                    g: c.display.g,
                    b: c.display.b,
                    a: 255,
                });
            pixels.push(colour);
        }
    }
    RgbaImage::new(out_w, out_h, pixels)
}
