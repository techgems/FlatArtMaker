use flatart_domain::settings::{CleanUpSettings, LoadSettings, PrintabilitySettings};
use flatart_domain::{LabelMap, LoadedImage, Paint, Printable, Result, RgbaImage};

/// The pixel stages: load (Pipeline step 1), clean up (4), region painting
/// (5) and printability (6).
///
/// # Contract
/// Every method is deterministic. Label maps keep their size, except that
/// `printability` upscales. Connectivity is 4-way (edge neighbours), as in
/// tracing.
///
/// - **`load`:**
///   - A pixel is outside when its alpha is below `settings.alpha_threshold`,
///     or when its RGB equals `settings.background`. Outside pixels become
///     fully transparent black (0, 0, 0, 0); every other pixel keeps its RGB
///     with alpha 255.
///   - `bounds` is the bounding box of the pixels that aren't outside.
///   - Fails with `InvalidInput` when every pixel is outside.
/// - **`clean_up`:**
///   - A mode filter: each cell takes the most common label in its 3 × 3
///     neighbourhood, keeping its own label on a tie.
///   - Then speckle merging: a connected area smaller than
///     `settings.speckle_area` cells takes the label it shares the longest
///     border with. An area that borders only the outside is kept.
///   - Outside cells never change, and no cell becomes outside. No label
///     appears that wasn't in the input.
/// - **`paint`:**
///   - In order, each paint fills the connected area under its point (in
///     source pixels) with its colour. Nothing else changes.
///   - A paint whose point is outside the image or on an outside cell does
///     nothing.
/// - **`printability`:**
///   - Upscales by repeating cells (nearest neighbour) to `settings.upscale`
///     cells per source pixel. Fails with `InvalidInput` when that isn't a
///     whole multiple of the input's upscale.
///   - With `settings.min_feature` set: removes every part of a colour
///     narrower than that width in mm (one cell is `scale / upscale` mm
///     wide), including islands inside another colour. Removed cells take
///     the other colour that most surrounds them. Parts at least that wide
///     are untouched.
///   - The fix never changes the outline: outside cells stay outside, and a
///     too-thin part with no other colour beside it (a whisker sticking into
///     the outside) keeps its colour.
///   - `thin_features` lists, one per area, the too-thin parts the fix had
///     to keep. It's empty when the fix is off.
///   - No label appears that wasn't in the input.
pub trait RasterStages {
    fn load(&self, image: &RgbaImage, settings: &LoadSettings) -> Result<LoadedImage>;
    fn clean_up(&self, labels: &LabelMap, settings: &CleanUpSettings) -> Result<LabelMap>;
    fn paint(&self, labels: &LabelMap, paints: &[Paint]) -> Result<LabelMap>;
    fn printability(&self, labels: &LabelMap, settings: &PrintabilitySettings)
    -> Result<Printable>;
}
