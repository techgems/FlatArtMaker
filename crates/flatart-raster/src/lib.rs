//! [`RasterStages`](flatart_ports::RasterStages) implementation: the pixel
//! stages between the image and tracing (Pipeline steps 1, 4, 5 and 6).
//!
//! - `load`: marks outside pixels and finds the art bounds.
//! - `clean_up`: the mode filter, then speckle merging.
//! - `paint`: flood-fills each painted area.
//! - `printability`: upscaling, then `thin` (the thin-feature fix).
//! - `grid`: cell neighbours and connected areas, shared by the stages.

mod clean_up;
mod grid;
mod load;
mod paint;
mod printability;
mod stages;
mod thin;

pub use stages::PixelStages;

#[cfg(test)]
mod tests;
