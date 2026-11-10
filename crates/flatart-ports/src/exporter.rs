use flatart_domain::{ExportOptions, Result};

/// Writes a model as a file (Pipeline step 9). `M` is `VectorModel` (SVG,
/// STEP) or `MeshModel` (3MF).
///
/// # Contract
/// - **Identity:** `id` is unique among the wired exporters, e.g. `svg`,
///   `step` or `3mf-prusa`. `file_extension` is lowercase with no dot.
///   `mime_type` is a media type such as `model/3mf`.
/// - **Bytes only:** returns the whole file and writes nothing anywhere.
/// - **Deterministic:** the same model and options give identical bytes
///   (no timestamps from the clock, no random IDs).
pub trait Exporter<M> {
    fn id(&self) -> &str;
    fn file_extension(&self) -> &str;
    fn mime_type(&self) -> &str;
    fn export(&self, model: &M, options: &ExportOptions) -> Result<Vec<u8>>;
}
