//! Ties the stages together into `RasterStages`.

use crate::{clean_up, load, paint, printability};
use flatart_domain::settings::{CleanUpSettings, LoadSettings, PrintabilitySettings};
use flatart_domain::{LabelMap, LoadedImage, Paint, Printable, Result, RgbaImage};
use flatart_ports::RasterStages;

/// The in-house pixel stages. Connectivity is 4-way throughout, matching
/// tracing.
#[derive(Debug, Clone, Copy, Default)]
pub struct PixelStages;

impl RasterStages for PixelStages {
    fn load(&self, image: &RgbaImage, settings: &LoadSettings) -> Result<LoadedImage> {
        load::load(image, settings)
    }

    fn clean_up(&self, labels: &LabelMap, settings: &CleanUpSettings) -> Result<LabelMap> {
        clean_up::clean_up(labels, settings)
    }

    fn paint(&self, labels: &LabelMap, paints: &[Paint]) -> Result<LabelMap> {
        paint::paint(labels, paints)
    }

    fn printability(
        &self,
        labels: &LabelMap,
        settings: &PrintabilitySettings,
    ) -> Result<Printable> {
        printability::printability(labels, settings)
    }
}
