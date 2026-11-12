//! The 3MF exporter: wraps a dialect's files in the 3MF package.

use crate::xml::Xml;
use flatart_domain::{Error, ExportOptions, MeshModel, PackageFile, Result, RgbaImage, Stage};
use flatart_ports::{Exporter, MODEL_PATH, SlicerDialect};
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, System, ZipWriter};

const THUMBNAIL_PATH: &str = "Metadata/thumbnail.png";
const RELS_NS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";

fn failed(what: &str, error: impl std::fmt::Display) -> Error {
    Error::StageFailed {
        stage: Stage::Export,
        message: format!("{what}: {error}"),
    }
}

/// Writes a 3MF in one slicer's dialect.
pub struct ThreeMfExporter {
    dialect: Box<dyn SlicerDialect>,
    id: String,
}

impl ThreeMfExporter {
    /// The exporter's id is `3mf-` plus the dialect's, e.g. `3mf-prusa`.
    pub fn new(dialect: Box<dyn SlicerDialect>) -> Self {
        let id = format!("3mf-{}", dialect.id());
        Self { dialect, id }
    }
}

fn content_types(thumbnail: bool) -> Result<Vec<u8>> {
    let mut xml = Xml::new()?;
    xml.open(
        "Types",
        &[(
            "xmlns",
            "http://schemas.openxmlformats.org/package/2006/content-types",
        )],
    )?;
    let mut defaults = vec![
        (
            "rels",
            "application/vnd.openxmlformats-package.relationships+xml",
        ),
        (
            "model",
            "application/vnd.ms-package.3dmanufacturing-3dmodel+xml",
        ),
    ];
    if thumbnail {
        defaults.push(("png", "image/png"));
    }
    for (extension, content_type) in defaults {
        xml.empty(
            "Default",
            &[("Extension", extension), ("ContentType", content_type)],
        )?;
    }
    xml.close("Types")?;
    Ok(xml.finish())
}

fn relationships(thumbnail: bool) -> Result<Vec<u8>> {
    let mut xml = Xml::new()?;
    xml.open("Relationships", &[("xmlns", RELS_NS)])?;
    let model_target = format!("/{MODEL_PATH}");
    xml.empty(
        "Relationship",
        &[
            ("Target", &model_target),
            ("Id", "rel-1"),
            (
                "Type",
                "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel",
            ),
        ],
    )?;
    if thumbnail {
        let target = format!("/{THUMBNAIL_PATH}");
        xml.empty(
            "Relationship",
            &[
                ("Target", &target),
                ("Id", "rel-2"),
                (
                    "Type",
                    "http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail",
                ),
            ],
        )?;
    }
    xml.close("Relationships")?;
    Ok(xml.finish())
}

fn png(image: &RgbaImage) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, image.width(), image.height());
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| failed("thumbnail", e))?;
    let data: Vec<u8> = image
        .pixels()
        .iter()
        .flat_map(|p| [p.r, p.g, p.b, p.a])
        .collect();
    writer
        .write_image_data(&data)
        .map_err(|e| failed("thumbnail", e))?;
    writer.finish().map_err(|e| failed("thumbnail", e))?;
    Ok(bytes)
}

fn zip(files: &[PackageFile]) -> Result<Vec<u8>> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    // A fixed time and host system, so the same model gives the same bytes
    // on every platform (the zip crate otherwise records DOS on Windows and
    // Unix elsewhere, WASM included).
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .last_modified_time(DateTime::default())
        .system(System::Unix);
    for file in files {
        zip.start_file(file.path.as_str(), options)
            .map_err(|e| failed("zip", e))?;
        zip.write_all(&file.bytes).map_err(|e| failed("zip", e))?;
    }
    let cursor = zip.finish().map_err(|e| failed("zip", e))?;
    Ok(cursor.into_inner())
}

impl Exporter<MeshModel> for ThreeMfExporter {
    fn id(&self) -> &str {
        &self.id
    }

    fn file_extension(&self) -> &str {
        "3mf"
    }

    fn mime_type(&self) -> &str {
        "model/3mf"
    }

    fn export(&self, model: &MeshModel, options: &ExportOptions) -> Result<Vec<u8>> {
        let thumbnail = options.thumbnail.as_ref();
        let mut files = vec![
            PackageFile {
                path: "[Content_Types].xml".into(),
                bytes: content_types(thumbnail.is_some())?,
            },
            PackageFile {
                path: "_rels/.rels".into(),
                bytes: relationships(thumbnail.is_some())?,
            },
        ];
        files.extend(self.dialect.parts(model)?);
        if let Some(image) = thumbnail {
            files.push(PackageFile {
                path: THUMBNAIL_PATH.into(),
                bytes: png(image)?,
            });
        }
        zip(&files)
    }
}
