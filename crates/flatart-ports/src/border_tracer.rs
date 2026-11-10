use crate::CurveFitter;
use flatart_domain::settings::TraceSettings;
use flatart_domain::{BorderMap, LabelMap, Result};

/// Builds the shared-border map from an output label map (Pipeline step 7).
///
/// # Contract
/// - **Coordinates:** source-image pixels. Cell edges of a map with upscale
///   *k* land on multiples of 1/*k*. `width` and `height` are the map's size
///   divided by *k*.
/// - **Borders:** one per run of cell edges between the same two labels,
///   split at junctions. `left != right`. Open borders start and end exactly
///   at their junctions' points; closed ones end where they start.
/// - **Curves:** every border's curve comes from `fitter`, with
///   `settings.fit`. The tracer never fits curves itself. Where those curves
///   would cross, the tracer may refit the borders involved more tightly
///   (a smaller tolerance, down to the pixel edges with every turn a corner).
/// - **No crossings:** borders only meet at their ends.
/// - **Faces:** one per 4-connected area of a non-outside label, with that
///   label's colour. Each loop is closed: each step ends exactly where the
///   next one starts.
/// - **Shared:** a border is used by exactly one loop of each non-outside
///   label on its sides, and only by faces of those labels.
/// - **Deterministic.**
pub trait BorderTracer {
    fn trace(
        &self,
        labels: &LabelMap,
        settings: &TraceSettings,
        fitter: &dyn CurveFitter,
    ) -> Result<BorderMap>;
}
