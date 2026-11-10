//! Stage 1: marks outside pixels and finds the art bounds.

use flatart_domain::settings::LoadSettings;
use flatart_domain::{ArtBounds, Error, LoadedImage, Result, Rgba, RgbaImage};

const OUTSIDE: Rgba = Rgba {
    r: 0,
    g: 0,
    b: 0,
    a: 0,
};

fn is_outside(pixel: Rgba, settings: &LoadSettings) -> bool {
    pixel.a < settings.alpha_threshold
        || settings
            .background
            .is_some_and(|bg| (pixel.r, pixel.g, pixel.b) == (bg.r, bg.g, bg.b))
}

pub fn load(image: &RgbaImage, settings: &LoadSettings) -> Result<LoadedImage> {
    let width = image.width() as usize;
    let mut bounds: Option<(usize, usize, usize, usize)> = None;
    let pixels = image
        .pixels()
        .iter()
        .enumerate()
        .map(|(i, &p)| {
            if is_outside(p, settings) {
                return OUTSIDE;
            }
            let (x, y) = (i % width, i / width);
            bounds = Some(match bounds {
                None => (x, y, x, y),
                Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
            });
            Rgba { a: 255, ..p }
        })
        .collect();
    let (x0, y0, x1, y1) = bounds.ok_or_else(|| {
        Error::InvalidInput("every pixel is outside the piece: nothing to print".into())
    })?;
    // Each coordinate is below the image's u32 width or height.
    let px = |v: usize| v as u32;
    Ok(LoadedImage {
        image: RgbaImage::new(image.width(), image.height(), pixels)?,
        bounds: ArtBounds {
            x: px(x0),
            y: px(y0),
            width: px(x1 - x0 + 1),
            height: px(y1 - y0 + 1),
        },
    })
}
