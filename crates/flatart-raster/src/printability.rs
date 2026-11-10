//! Stage 6: upscaling, then the thin-feature fix.

use crate::thin;
use flatart_domain::settings::{PrintabilitySettings, TraceUpscale};
use flatart_domain::{Error, LabelMap, Printable, Result};

pub fn printability(map: &LabelMap, settings: &PrintabilitySettings) -> Result<Printable> {
    let map = upscale(map, settings.upscale)?;
    let Some(min) = settings.min_feature else {
        return Ok(Printable {
            labels: map,
            thin_features: Vec::new(),
        });
    };
    let cell_mm = settings.scale.get().0 / f64::from(settings.upscale.get());
    let fixed = thin::fix(&map, thin::diameter(min.get().0, cell_mm));
    Ok(Printable {
        labels: LabelMap::new(map.width(), map.height(), map.upscale(), fixed.labels)?,
        thin_features: fixed.kept_thin,
    })
}

/// Repeats each cell so `to` cells span one source pixel.
fn upscale(map: &LabelMap, to: TraceUpscale) -> Result<LabelMap> {
    let (from, to_n) = (map.upscale().get(), to.get());
    if to_n % from != 0 {
        return Err(Error::InvalidInput(format!(
            "can't upscale a label map from ×{from} to ×{to_n}: not a whole multiple"
        )));
    }
    let k = u32::from(to_n / from);
    if k == 1 {
        return Ok(map.clone());
    }
    let (w, h) = (map.width() * k, map.height() * k);
    let src = map.labels();
    let src_w = map.width() as usize;
    let k = k as usize;
    let labels = (0..h as usize)
        .flat_map(|y| (0..w as usize).map(move |x| src[(y / k) * src_w + x / k]))
        .collect();
    LabelMap::new(w, h, to, labels)
}
