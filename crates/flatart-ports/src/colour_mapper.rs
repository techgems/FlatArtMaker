use flatart_domain::settings::MappingSettings;
use flatart_domain::{Detection, EditLog, MappedColours, Result};

/// Maps detected colours to output colours: the automatic first pass, then
/// the edit log's active operations in order (Pipeline step 3).
///
/// # Contract
/// - **Palette:** at most `MAX_OUTPUT_COLOURS` (16) output colours, in slot
///   order. Colours nothing maps or paints to are kept (the editor shows them
///   with a warning).
/// - **Automatic pass:** the most similar pair of groups (pixel-weighted mean
///   colours, smallest ΔE) is merged, over and over, until at most
///   `settings.target` groups remain: the user's colour count. Group *n* in
///   slot order gets `OutputColourId(n)`; added colours use
///   `OutputColourId::FIRST_ADDED` and up.
/// - **Similarity:** `similarity` is that merge order carried on down to one
///   group, over every detected colour. With no edits, two detected colours
///   share an output exactly when `similarity.groups(target)` puts them in
///   the same group.
/// - **Mapping:** one target per detected colour, each in the palette.
/// - **Label map:** same size and upscale as `detection.labels`. Outside
///   stays outside; every other cell is the target of its detected label.
/// - **Paints:** the active `PaintRegion`s, in order, after any delete
///   moved their colour. Every paint colour is in the palette. They aren't
///   applied here: stage 5 does that after clean-up.
/// - **Base:** an explicit `SetBase` wins; otherwise the colour most common
///   along the art's outer edge. `None` only when the palette is empty.
/// - **Stable IDs:** a `Map` names a detected colour by its OKLab value and
///   applies to the nearest current detected colour within the adapter's ΔE
///   limit, so edits survive changes to detection.
/// - **Dropped edits:** an active operation that can't apply (no matching
///   detected colour, unknown output colour, a 17th colour, a taken name, …)
///   is skipped and reported with its log position, in log order. It never
///   fails the whole mapping.
/// - **Undo and redo:** only `edits.active()` counts.
/// - **Deterministic:** the same input always gives the same output.
pub trait ColourMapper {
    fn map(
        &self,
        detection: &Detection,
        edits: &EditLog,
        settings: &MappingSettings,
    ) -> Result<MappedColours>;
}
