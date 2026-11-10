//! Conformance suites for [`Exporter`].

use super::{fixtures, ok};
use crate::Exporter;
use flatart_domain::{ExportOptions, MeshModel, VectorModel};

/// Runs every check on a vector exporter (SVG, STEP).
pub fn run_vector(exporter: &dyn Exporter<VectorModel>) {
    run(exporter, &fixtures::vector_model());
}

/// Runs every check on a mesh exporter (3MF).
pub fn run_mesh(exporter: &dyn Exporter<MeshModel>) {
    run(exporter, &fixtures::mesh_model());
}

fn run<M>(exporter: &dyn Exporter<M>, model: &M) {
    let id = exporter.id();
    assert!(!id.is_empty(), "exporter id is empty");
    let ext = exporter.file_extension();
    assert!(
        !ext.is_empty() && !ext.contains('.') && ext == ext.to_ascii_lowercase(),
        "{id}: file extension `{ext}` must be lowercase with no dot"
    );
    let mime = exporter.mime_type();
    assert!(mime.contains('/'), "{id}: `{mime}` is not a media type");

    let plain = ExportOptions {
        name: "FlatArt".into(),
        thumbnail: None,
    };
    let with_thumbnail = ExportOptions {
        name: "FlatArt".into(),
        thumbnail: Some(fixtures::two_colour_image()),
    };
    for (case, options) in [("plain", plain), ("with thumbnail", with_thumbnail)] {
        let bytes = ok(exporter.export(model, &options), case);
        assert!(!bytes.is_empty(), "{id} ({case}): no bytes");
        let again = ok(exporter.export(model, &options), case);
        assert!(bytes == again, "{id} ({case}): not deterministic");
    }
}
