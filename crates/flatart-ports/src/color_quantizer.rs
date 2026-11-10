use flatart_domain::settings::QuantSettings;
use flatart_domain::{Detection, Oklab, Result, RgbaImage};

/// Detects the colours in an image (Pipeline step 2).
///
/// # Contract
/// - **Input:** the Load stage's image, where alpha is 0 for outside pixels and
///   anything else for pixels in the piece.
/// - **Label map:** same size as the image, `upscale` 1. Outside pixels get
///   `Label::OUTSIDE`; every other pixel gets the index of a palette entry.
/// - **Palette:** at most `settings.max_colours` entries, plus one per pin
///   when the pins push it over. Each entry's `pixels` equals the number of
///   cells with its label.
/// - **Pins:** every pin appears in the palette with exactly its OKLab value.
/// - **Assignment:** pixels with identical RGB get the same label. When the
///   image has no more distinct colours than the limit, distinct colours get
///   distinct labels.
/// - **Deterministic:** the same input always gives the same output.
pub trait ColorQuantizer {
    fn quantize(
        &self,
        image: &RgbaImage,
        settings: &QuantSettings,
        pins: &[Oklab],
    ) -> Result<Detection>;
}
