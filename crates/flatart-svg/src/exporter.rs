//! `Exporter<VectorModel>` for SVG.

use crate::document;
use flatart_domain::{ExportOptions, Result, VectorModel};
use flatart_ports::Exporter;

/// Writes the art as an SVG, sized in mm.
#[derive(Debug, Clone, Copy, Default)]
pub struct SvgExporter;

impl Exporter<VectorModel> for SvgExporter {
    fn id(&self) -> &str {
        "svg"
    }

    fn file_extension(&self) -> &str {
        "svg"
    }

    fn mime_type(&self) -> &str {
        "image/svg+xml"
    }

    fn export(&self, model: &VectorModel, options: &ExportOptions) -> Result<Vec<u8>> {
        Ok(document::write(model, &options.name).into_bytes())
    }
}
