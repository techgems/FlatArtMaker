use crate::{BambuDialect, PrusaDialect, ThreeMfExporter};
use flatart_domain::{ExportOptions, MeshModel, PackageFile};
use flatart_ports::testkit::{exporter, fixtures, slicer_dialect};
use flatart_ports::{Exporter, MODEL_PATH, SlicerDialect};
use std::io::{Cursor, Read};

#[test]
fn dialects_conform() {
    slicer_dialect::run(&PrusaDialect);
    slicer_dialect::run(&BambuDialect);
}

#[test]
fn exporters_conform() {
    exporter::run_mesh(&ThreeMfExporter::new(Box::new(PrusaDialect)));
    exporter::run_mesh(&ThreeMfExporter::new(Box::new(BambuDialect)));
}

fn file(files: &[PackageFile], path: &str) -> String {
    let file = files.iter().find(|f| f.path == path).unwrap();
    String::from_utf8(file.bytes.clone()).unwrap()
}

/// The fixture: `Base` (Black, slot 1) and `White` (slot 2), 12 triangles each.
fn model() -> MeshModel {
    fixtures::mesh_model()
}

#[test]
fn prusa_names_each_part_and_puts_slot_n_on_extruder_n() {
    let files = PrusaDialect.parts(&model()).unwrap();
    let config = file(&files, "Metadata/Slic3r_PE_model.config");
    let volume = |first, last, name, extruder| {
        format!(
            "<volume firstid=\"{first}\" lastid=\"{last}\">\n   \
             <metadata type=\"volume\" key=\"name\" value=\"{name}\"/>\n   \
             <metadata type=\"volume\" key=\"extruder\" value=\"{extruder}\"/>"
        )
    };
    assert!(config.contains(&volume(0, 11, "Base", 1)), "{config}");
    assert!(config.contains(&volume(12, 23, "White", 2)), "{config}");
    let model = file(&files, MODEL_PATH);
    assert_eq!(model.matches("<object ").count(), 1, "one object, one mesh");
    assert!(model.contains(r#"<metadata name="slic3rpe:Version3mf">1</metadata>"#));
    assert!(model.contains(r##"<m:color color="#141414FF"/>"##));
    assert!(model.contains(r##"<m:color color="#F0F0F0FF"/>"##));
}

#[test]
fn bambu_names_each_part_and_puts_slot_n_on_extruder_n() {
    let files = BambuDialect.parts(&model()).unwrap();
    let config = file(&files, "Metadata/model_settings.config");
    let part = |id, name, extruder| {
        format!(
            "<part id=\"{id}\" subtype=\"normal_part\">\n   \
             <metadata key=\"name\" value=\"{name}\"/>\n   \
             <metadata key=\"extruder\" value=\"{extruder}\"/>"
        )
    };
    assert!(config.contains(&part(1, "Base", 1)), "{config}");
    assert!(config.contains(&part(2, "White", 2)), "{config}");
    let model = file(&files, MODEL_PATH);
    assert_eq!(
        model.matches("<object ").count(),
        3,
        "two parts and their parent"
    );
    assert_eq!(model.matches("<component ").count(), 2);
    assert!(!model.contains("Application"), "no Application tag");
    assert!(model.contains(r#"<m:colorgroup id="4">"#));
}

fn unzip(bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    (0..archive.len())
        .map(|i| {
            let mut entry = archive.by_index(i).unwrap();
            let mut data = Vec::new();
            entry.read_to_end(&mut data).unwrap();
            (entry.name().unwrap().into_owned(), data)
        })
        .collect()
}

#[test]
fn the_package_holds_the_dialect_files_and_the_thumbnail() {
    let exporter = ThreeMfExporter::new(Box::new(PrusaDialect));
    assert_eq!(exporter.id(), "3mf-prusa");
    let options = ExportOptions {
        name: "Test".into(),
        thumbnail: Some(fixtures::two_colour_image()),
    };
    let entries = unzip(&exporter.export(&model(), &options).unwrap());
    let names: Vec<_> = entries.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        [
            "[Content_Types].xml",
            "_rels/.rels",
            "3D/3dmodel.model",
            "Metadata/Slic3r_PE_model.config",
            "Metadata/thumbnail.png",
        ]
    );
    let png = &entries[4].1;
    assert_eq!(&png[1..4], b"PNG");
    let rels = String::from_utf8(entries[1].1.clone()).unwrap();
    assert!(
        rels.contains("Target=\"/Metadata/thumbnail.png\""),
        "{rels}"
    );
}

#[test]
fn entries_say_unix_on_every_platform() {
    // The zip crate records the host system; pinning it keeps the bytes the
    // same from the Windows CLI, Linux and the browser.
    let exporter = ThreeMfExporter::new(Box::new(BambuDialect));
    let bytes = exporter
        .export(&model(), &ExportOptions::default())
        .unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    for i in 0..archive.len() {
        let entry = archive.by_index(i).unwrap();
        assert_eq!(entry.system(), zip::System::Unix, "{:?}", entry.name());
    }
}
