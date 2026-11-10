//! User settings (SPEC "Controls"), and the per-stage views the ports receive.
//!
//! Every value type rejects invalid values on construction and on deserialising.

use crate::bounded::{bounded_f64, bounded_int};
use crate::{Degrees, Error, Mm, MmPerPx, Px, Result, Rgb};
use serde::{Deserialize, Serialize};

bounded_f64!(
    /// A Z height: the colour layer height C, or the base height B. Never 0 or negative.
    LayerHeight(Mm), "layer height", min = 0.0, min_inclusive = false, max = 10.0
);

bounded_f64!(
    /// The smallest X/Y feature width the thin-feature fix allows.
    MinFeatureWidth(Mm), "minimum feature width", min = 0.25, min_inclusive = true, max = 1.2
);

bounded_f64!(
    /// The X/Y size, stored as mm per source pixel so the aspect ratio can't drift.
    PrintScale(MmPerPx), "print scale", min = 0.0, min_inclusive = false, max = f64::MAX
);

bounded_f64!(
    /// How far a fitted curve may stray from the traced pixel border.
    SmoothingTolerance(Px), "smoothing tolerance", min = 0.1, min_inclusive = true, max = 2.0
);

bounded_f64!(
    /// A turn sharper than this stays a sharp corner.
    CornerThreshold(Degrees), "corner threshold", min = 0.0, min_inclusive = true, max = 180.0
);

bounded_f64!(
    /// How far mesh edges may stray from the curves.
    CurveTolerance(Mm), "curve tolerance", min = 0.005, min_inclusive = true, max = 0.2
);

bounded_int!(
    /// The most colours detection may find.
    DetectedColourLimit(u8 as "u8"), "detected colour limit", min = 2, max = 32
);

bounded_int!(
    /// How many output colours the user wants. The automatic mapping merges
    /// the most similar detected colours until no more than this remain.
    OutputColourTarget(u8 as "u8"), "output colour target", min = 1, max = 16
);

bounded_int!(
    /// Connected blobs smaller than this many pixels are merged away.
    SpeckleArea(u32 as "u32"), "speckle area", min = 0, max = 500
);

bounded_int!(
    /// Label-map cells per source pixel, in each direction.
    TraceUpscale(u8 as "u8"), "trace upscale", min = 1, max = 4
);

impl LayerHeight {
    /// The base height slider's value: 0 means no base.
    pub fn optional(value: Mm) -> Result<Option<Self>> {
        if value.0 == 0.0 {
            Ok(None)
        } else {
            Self::new(value).map(Some)
        }
    }
}

impl PrintScale {
    /// The scale that makes the art's longer side `longest` long.
    pub fn from_longest_side(longest: Mm, width_px: u32, height_px: u32) -> Result<Self> {
        let side = width_px.max(height_px);
        if side == 0 {
            return Err(Error::InvalidValue {
                what: "print scale",
                reason: "the art has no pixels".into(),
            });
        }
        Self::new(MmPerPx(longest.0 / f64::from(side)))
    }

    /// The printed size of an area `width_px` × `height_px` source pixels.
    pub fn size(self, width_px: u32, height_px: u32) -> (Mm, Mm) {
        (
            Mm(self.0 * f64::from(width_px)),
            Mm(self.0 * f64::from(height_px)),
        )
    }
}

/// All user settings. Versioned, so older saved settings can be migrated.
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub version: u32,
    /// C: the colour layer height.
    pub colour_height: LayerHeight,
    /// B: the base height. `None` means no base.
    pub base_height: Option<LayerHeight>,
    pub scale: PrintScale,
    /// `None` means "Fix thin features" is off.
    pub min_feature: Option<MinFeatureWidth>,
    pub load: LoadSettings,
    pub advanced: Advanced,
}

impl Settings {
    pub const CURRENT_VERSION: u32 = 1;

    /// The SPEC defaults, for an image whose size gives `scale`.
    pub fn with_defaults(scale: PrintScale) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            colour_height: LayerHeight::new_unchecked(1.0),
            base_height: Some(LayerHeight::new_unchecked(3.4)),
            scale,
            min_feature: Some(MinFeatureWidth::new_unchecked(0.25)),
            load: LoadSettings::default(),
            advanced: Advanced::default(),
        }
    }

    pub fn quant(&self) -> QuantSettings {
        QuantSettings {
            max_colours: self.advanced.detected_colours,
        }
    }

    pub fn mapping(&self) -> MappingSettings {
        MappingSettings {
            target: self.advanced.output_colours,
        }
    }

    pub fn clean_up(&self) -> CleanUpSettings {
        CleanUpSettings {
            speckle_area: self.advanced.speckle_area,
        }
    }

    pub fn printability(&self) -> PrintabilitySettings {
        PrintabilitySettings {
            upscale: self.advanced.trace_upscale,
            scale: self.scale,
            min_feature: self.min_feature,
        }
    }

    pub fn trace(&self) -> TraceSettings {
        TraceSettings {
            fit: FitSettings {
                tolerance: self.advanced.smoothing,
                corner: self.advanced.corner,
            },
        }
    }

    pub fn mesh(&self) -> MeshSettings {
        MeshSettings {
            curve_tolerance: self.advanced.curve_tolerance,
        }
    }

    pub fn extrusion(&self) -> Extrusion {
        Extrusion {
            colour: self.colour_height,
            base: self.base_height,
            fill_base_holes: self.advanced.fill_base_holes,
        }
    }
}

/// How the Load stage decides what is outside the piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadSettings {
    /// Pixels with alpha below this are outside.
    pub alpha_threshold: u8,
    /// For images with no transparency: pixels of this colour are outside.
    pub background: Option<Rgb>,
}

impl Default for LoadSettings {
    fn default() -> Self {
        Self {
            alpha_threshold: 128,
            background: None,
        }
    }
}

/// The collapsed "Advanced" section.
#[derive(Debug, Clone, Copy, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Advanced {
    pub detected_colours: DetectedColourLimit,
    pub output_colours: OutputColourTarget,
    pub speckle_area: SpeckleArea,
    pub trace_upscale: TraceUpscale,
    pub smoothing: SmoothingTolerance,
    pub corner: CornerThreshold,
    pub curve_tolerance: CurveTolerance,
    pub fill_base_holes: bool,
}

impl Default for Advanced {
    fn default() -> Self {
        Self {
            detected_colours: DetectedColourLimit::new_unchecked(24),
            output_colours: OutputColourTarget::new_unchecked(8),
            speckle_area: SpeckleArea::new_unchecked(16),
            trace_upscale: TraceUpscale::new_unchecked(2),
            smoothing: SmoothingTolerance::new_unchecked(0.5),
            corner: CornerThreshold::new_unchecked(60.0),
            curve_tolerance: CurveTolerance::new_unchecked(0.02),
            fill_base_holes: true,
        }
    }
}

/// Settings for `ColorQuantizer`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuantSettings {
    pub max_colours: DetectedColourLimit,
}

/// Settings for the automatic colour mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MappingSettings {
    pub target: OutputColourTarget,
}

/// Settings for the clean-up stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CleanUpSettings {
    pub speckle_area: SpeckleArea,
}

/// Settings for the printability stage.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct PrintabilitySettings {
    pub upscale: TraceUpscale,
    pub scale: PrintScale,
    pub min_feature: Option<MinFeatureWidth>,
}

/// Settings for `CurveFitter`.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct FitSettings {
    pub tolerance: SmoothingTolerance,
    pub corner: CornerThreshold,
}

/// Settings for `BorderTracer`.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct TraceSettings {
    pub fit: FitSettings,
}

/// Settings for the mesh builder.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct MeshSettings {
    pub curve_tolerance: CurveTolerance,
}

/// The Z heights: colour layer from B to B + C, base from 0 to B.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct Extrusion {
    pub colour: LayerHeight,
    pub base: Option<LayerHeight>,
    pub fill_base_holes: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scale() -> PrintScale {
        PrintScale::new(MmPerPx(0.1)).unwrap()
    }

    #[test]
    fn colour_height_cannot_be_zero_or_negative() {
        assert!(LayerHeight::new(Mm(0.0)).is_err());
        assert!(LayerHeight::new(Mm(-1.0)).is_err());
        assert!(LayerHeight::new(Mm(0.05)).is_ok());
        assert!(LayerHeight::new(Mm(10.5)).is_err());
    }

    #[test]
    fn base_height_zero_means_no_base_and_negative_is_rejected() {
        assert_eq!(LayerHeight::optional(Mm(0.0)).unwrap(), None);
        assert!(LayerHeight::optional(Mm(-0.05)).is_err());
        assert!(LayerHeight::optional(Mm(3.4)).unwrap().is_some());
    }

    #[test]
    fn min_feature_width_has_a_floor_of_quarter_mm() {
        assert!(MinFeatureWidth::new(Mm(0.2)).is_err());
        assert!(MinFeatureWidth::new(Mm(0.25)).is_ok());
        assert!(MinFeatureWidth::new(Mm(1.25)).is_err());
    }

    #[test]
    fn values_a_float_step_past_a_bound_are_clamped() {
        let width = MinFeatureWidth::new(Mm(0.25 + 19.0 * 0.05)).unwrap();
        assert_eq!(width.get(), Mm(1.2));
    }

    #[test]
    fn non_finite_values_are_rejected() {
        assert!(LayerHeight::new(Mm(f64::NAN)).is_err());
        assert!(PrintScale::new(MmPerPx(f64::INFINITY)).is_err());
        assert!(PrintScale::new(MmPerPx(0.0)).is_err());
    }

    #[test]
    fn integer_settings_are_range_checked() {
        assert!(DetectedColourLimit::new(1).is_err());
        assert!(DetectedColourLimit::new(33).is_err());
        assert!(OutputColourTarget::new(16).is_ok());
        assert!(OutputColourTarget::new(17).is_err());
        assert!(TraceUpscale::new(0).is_err());
        assert!(SpeckleArea::new(501).is_err());
    }

    #[test]
    fn print_scale_keeps_the_aspect_ratio() {
        let scale = PrintScale::from_longest_side(Mm(100.0), 400, 300).unwrap();
        let (x, y) = scale.size(400, 300);
        assert!((x.0 - 100.0).abs() < 1e-9);
        assert!((y.0 - 75.0).abs() < 1e-9);
    }

    #[test]
    fn defaults_are_valid_values() {
        let s = Settings::with_defaults(scale());
        let a = s.advanced;
        assert!(LayerHeight::new(s.colour_height.get()).is_ok());
        assert!(DetectedColourLimit::new(a.detected_colours.get()).is_ok());
        assert!(CurveTolerance::new(a.curve_tolerance.get()).is_ok());
        assert!(SmoothingTolerance::new(a.smoothing.get()).is_ok());
    }

    #[test]
    fn settings_round_trip_through_json() {
        let settings = Settings::with_defaults(scale());
        let json = serde_json::to_string(&settings).unwrap();
        assert_eq!(serde_json::from_str::<Settings>(&json).unwrap(), settings);
    }

    #[test]
    fn deserialising_rejects_invalid_values() {
        let mut json = serde_json::to_value(Settings::with_defaults(scale())).unwrap();
        json["colourHeight"] = 0.0.into();
        assert!(serde_json::from_value::<Settings>(json.clone()).is_err());
        json["colourHeight"] = 1.0.into();
        json["minFeature"] = 0.1.into();
        assert!(serde_json::from_value::<Settings>(json.clone()).is_err());
        json["minFeature"] = serde_json::Value::Null;
        json["baseHeight"] = (-1.0).into();
        assert!(serde_json::from_value::<Settings>(json).is_err());
    }
}
