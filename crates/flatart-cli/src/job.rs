//! What to make, and a pipeline set up to make it.

use crate::wiring;
use flatart_domain::settings::{LayerHeight, MinFeatureWidth, OutputColourTarget, PrintScale};
use flatart_domain::{EditLog, Error, Mm, MmPerPx, Result, RgbaImage, Settings};
use flatart_pipeline::Pipeline;

/// Which slicer the 3MF is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slicer {
    Prusa,
    Bambu,
}

impl Slicer {
    /// The pipeline's exporter id.
    pub fn exporter(self) -> &'static str {
        match self {
            Slicer::Prusa => "3mf-prusa",
            Slicer::Bambu => "3mf-bambu",
        }
    }
}

/// Which library fits the curves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Fitter {
    /// kurbo (the default).
    #[default]
    Kurbo,
    /// visioncortex, the fitting behind vtracer.
    VisionCortex,
}

/// "Fix thin features": off, or on with the smallest width in mm.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThinFix {
    Off,
    MinWidth(f64),
}

/// One run: the image, the settings the CLI exposes, and the edits.
#[derive(Debug, Clone)]
pub struct Job {
    pub image: RgbaImage,
    pub output_colours: u8,
    /// B in mm; 0 means no base.
    pub base: f64,
    /// C in mm.
    pub layer: f64,
    /// The art's width in mm.
    pub size_x: f64,
    pub thin_fix: ThinFix,
    pub edits: EditLog,
    pub fitter: Fitter,
}

impl Job {
    /// The SPEC defaults for `image`.
    pub fn new(image: RgbaImage) -> Self {
        Self {
            image,
            output_colours: 8,
            base: 3.4,
            layer: 1.0,
            size_x: 100.0,
            thin_fix: ThinFix::MinWidth(0.25),
            edits: EditLog::new(),
            fitter: Fitter::default(),
        }
    }

    fn settings(&self, scale: PrintScale) -> Result<Settings> {
        let mut settings = Settings::with_defaults(scale);
        settings.advanced.output_colours = OutputColourTarget::new(self.output_colours)?;
        settings.base_height = LayerHeight::optional(Mm(self.base))?;
        settings.colour_height = LayerHeight::new(Mm(self.layer))?;
        settings.min_feature = match self.thin_fix {
            ThinFix::Off => None,
            ThinFix::MinWidth(w) => Some(MinFeatureWidth::new(Mm(w))?),
        };
        Ok(settings)
    }

    /// A wired pipeline with the image, edits and settings set. The scale
    /// comes from the art's width once the image is loaded.
    pub fn pipeline(&self) -> Result<Pipeline> {
        if !(self.size_x.is_finite() && self.size_x > 0.0) {
            return Err(Error::InvalidInput(format!(
                "the X size must be above 0 mm, got {}",
                self.size_x
            )));
        }
        let mut pipeline = wiring::pipeline(self.fitter)?;
        pipeline.set_image(self.image.clone());
        pipeline.set_edits(self.edits.clone());
        pipeline.set_settings(self.settings(PrintScale::new(MmPerPx(1.0))?)?);
        let width = pipeline.loaded()?.bounds.width;
        let scale = PrintScale::new(MmPerPx(self.size_x / f64::from(width)))?;
        pipeline.set_settings(self.settings(scale)?);
        Ok(pipeline)
    }
}
