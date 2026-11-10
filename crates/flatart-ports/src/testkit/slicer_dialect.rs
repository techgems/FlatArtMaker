//! Conformance suite for [`SlicerDialect`].
//!
//! It checks what doesn't depend on the format. Each dialect's own tests
//! check its part names and extruders (task 12).

use super::{fixtures, ok};
use crate::{MODEL_PATH, SlicerDialect};
use std::collections::HashSet;

/// Files the 3MF exporter writes itself, which a dialect must not.
pub const EXPORTER_FILES: [&str; 3] = [
    "[Content_Types].xml",
    "_rels/.rels",
    "Metadata/thumbnail.png",
];

/// Runs every check on `dialect`.
pub fn run(dialect: &dyn SlicerDialect) {
    let id = dialect.id();
    assert!(
        !id.is_empty()
            && id
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
        "dialect id `{id}` must be lowercase letters and digits"
    );
    let model = fixtures::mesh_model();
    let files = ok(dialect.parts(&model), id);

    let mut paths = HashSet::new();
    for file in &files {
        let path = file.path.as_str();
        assert!(
            !path.is_empty() && !path.starts_with('/') && !path.contains('\\'),
            "{id}: path `{path}` must be relative with `/` separators"
        );
        assert!(paths.insert(path), "{id}: path `{path}` is repeated");
        assert!(
            !EXPORTER_FILES.contains(&path),
            "{id}: `{path}` belongs to the 3MF exporter"
        );
        assert!(!file.bytes.is_empty(), "{id}: `{path}` is empty");
    }
    assert!(paths.contains(MODEL_PATH), "{id}: no `{MODEL_PATH}`");

    let again = ok(dialect.parts(&model), id);
    assert!(files == again, "{id}: not deterministic");
}
