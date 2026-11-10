//! The `ColorQuantizer` implementation: histogram → centres → palette and labels.

use crate::histogram::{ColourCount, histogram};
use crate::kmeans;
use crate::lab::{Lab, nearest};
use flatart_domain::settings::{QuantSettings, TraceUpscale};
use flatart_domain::{
    DetectedColour, DetectedPalette, Detection, Label, LabelMap, Oklab, Result, RgbaImage,
};
use flatart_ports::ColorQuantizer;
use std::cmp::Reverse;
use std::collections::HashMap;

/// Detects colours with weighted k-means in OKLab.
///
/// - When the image has no more distinct colours than the limit, each one
///   becomes its own detected colour and k-means doesn't run.
/// - Pins count toward the limit; k-means fills the remaining slots.
/// - The palette is sorted by pixel count, largest first. Centres no pixel
///   chose are dropped, except pins.
#[derive(Debug, Clone, Copy, Default)]
pub struct KmeansQuantizer;

impl ColorQuantizer for KmeansQuantizer {
    fn quantize(
        &self,
        image: &RgbaImage,
        settings: &QuantSettings,
        pins: &[Oklab],
    ) -> Result<Detection> {
        let colours = histogram(image);
        let pins = unique(pins);
        let fixed: Vec<Lab> = pins.iter().map(|&p| Lab::from_domain(p)).collect();
        let limit = usize::from(settings.max_colours.get());
        let centres = if colours.len() <= limit {
            let own = colours.iter().map(|c| c.lab).filter(|l| !fixed.contains(l));
            fixed.iter().copied().chain(own).collect()
        } else {
            kmeans::cluster(&colours, &fixed, limit.saturating_sub(fixed.len()))
        };
        build(image, &colours, &pins, &centres)
    }
}

/// `pins` without repeats, in their original order.
fn unique(pins: &[Oklab]) -> Vec<Oklab> {
    let mut seen = Vec::with_capacity(pins.len());
    for &p in pins {
        if !seen.contains(&p) {
            seen.push(p);
        }
    }
    seen
}

/// Assigns every colour to its nearest centre and builds the result.
/// `centres` starts with the pins, in the same order.
fn build(
    image: &RgbaImage,
    colours: &[ColourCount],
    pins: &[Oklab],
    centres: &[Lab],
) -> Result<Detection> {
    let owner: Vec<usize> = colours.iter().map(|c| nearest(c.lab, centres)).collect();
    let mut pixels = vec![0u64; centres.len()];
    for (c, &o) in colours.iter().zip(&owner) {
        pixels[o] += c.pixels;
    }

    let mut order: Vec<usize> = (0..centres.len())
        .filter(|&i| i < pins.len() || pixels[i] > 0)
        .collect();
    order.sort_by_key(|&i| Reverse(pixels[i])); // stable: ties keep centre order
    let entries = order
        .iter()
        .map(|&i| {
            let colour = match pins.get(i) {
                Some(&pin) => pin,
                None => centres[i].to_domain()?,
            };
            Ok(DetectedColour {
                colour,
                pixels: pixels[i],
            })
        })
        .collect::<Result<Vec<_>>>()?;
    // Validates the count, so every position below fits in a label.
    let palette = DetectedPalette::new(entries)?;

    let mut label_of_centre = vec![Label::OUTSIDE; centres.len()];
    for (position, &i) in order.iter().enumerate() {
        label_of_centre[i] = Label(position as u16);
    }
    let label_of_rgb: HashMap<[u8; 3], Label> = colours
        .iter()
        .zip(&owner)
        .map(|(c, &o)| (c.rgb, label_of_centre[o]))
        .collect();
    let labels = image
        .pixels()
        .iter()
        .map(|p| match p.a {
            0 => Label::OUTSIDE,
            _ => label_of_rgb[&[p.r, p.g, p.b]],
        })
        .collect();
    let labels = LabelMap::new(image.width(), image.height(), TraceUpscale::new(1)?, labels)?;
    Ok(Detection { palette, labels })
}
