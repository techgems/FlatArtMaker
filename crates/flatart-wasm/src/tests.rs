use crate::project::Project;
use crate::types::Slicer;
use flatart_cli::{Job, read_png};
use flatart_domain::{ExportOptions, RgbaImage};
use flatart_pipeline::THUMBNAIL_SIDE;

fn jolteon() -> RgbaImage {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/jolteon/jolteon.png"
    );
    read_png(&std::fs::read(path).unwrap()).unwrap()
}

fn rgba_bytes(image: &RgbaImage) -> Vec<u8> {
    image
        .pixels()
        .iter()
        .flat_map(|p| [p.r, p.g, p.b, p.a])
        .collect()
}

fn loaded(image: &RgbaImage) -> Project {
    let mut project = Project::new().unwrap();
    project
        .load_image(&rgba_bytes(image), image.width(), image.height())
        .unwrap();
    project
}

/// A 3 × 2 image: a red and a blue pixel on the top row, the rest clear.
fn tiny() -> (Vec<u8>, u32, u32) {
    let mut bytes = vec![0; 3 * 2 * 4];
    bytes[..4].copy_from_slice(&[255, 0, 0, 255]);
    bytes[4..8].copy_from_slice(&[0, 0, 255, 255]);
    (bytes, 3, 2)
}

#[test]
fn jolteon_3mf_matches_the_cli() {
    let image = jolteon();
    let mut project = loaded(&image);

    let mut cli = Job::new(image).pipeline().unwrap();
    for (slicer, id) in [(Slicer::Prusa, "3mf-prusa"), (Slicer::Bambu, "3mf-bambu")] {
        let options = ExportOptions {
            name: "jolteon".into(),
            thumbnail: Some(cli.thumbnail(THUMBNAIL_SIDE).unwrap()),
        };
        let expected = cli.export(id, &options).unwrap();
        let got = project.export_3mf(slicer, "jolteon").unwrap();
        assert!(
            got == expected,
            "{id}: {} vs {} bytes",
            got.len(),
            expected.len()
        );
    }
}

#[test]
fn loading_gives_bounds_and_defaults_sized_to_100_mm() {
    let (bytes, w, h) = tiny();
    let mut project = Project::new().unwrap();
    let info = project.load_image(&bytes, w, h).unwrap();
    assert_eq!((info.bounds.width, info.bounds.height), (2, 1));
    let (x, _) = info.settings.scale.size(2, 1);
    assert!((x.0 - 100.0).abs() < 1e-9, "{x:?}");
}

#[test]
fn bad_pixels_are_an_error() {
    let mut project = Project::new().unwrap();
    assert!(
        project.load_image(&[0; 7], 1, 2).is_err(),
        "not whole pixels"
    );
    assert!(project.load_image(&[0; 8], 3, 1).is_err(), "wrong size");
}

#[test]
fn nothing_runs_before_an_image() {
    let mut project = Project::new().unwrap();
    assert!(project.mapped().is_err());
    assert!(project.mesh_parts().is_err());
}

#[test]
fn the_label_map_covers_the_image() {
    let (bytes, w, h) = tiny();
    let mut project = Project::new().unwrap();
    project.load_image(&bytes, w, h).unwrap();
    let info = project.mapped().unwrap();
    let labels = project.mapped_labels().unwrap();
    assert_eq!((info.width, info.height), (w, h));
    assert_eq!(labels.len(), (w * h) as usize);
    assert!(labels[2..].iter().all(|&l| l == info.outside));
    let ids: Vec<u16> = info.palette.colours().iter().map(|c| c.id.0).collect();
    assert!(labels[..2].iter().all(|l| ids.contains(l)), "{labels:?}");
}

#[test]
fn mesh_buffers_match_their_parts() {
    let (bytes, w, h) = tiny();
    let mut project = Project::new().unwrap();
    project.load_image(&bytes, w, h).unwrap();
    let parts = project.mesh_parts().unwrap();
    assert!(!parts.is_empty());
    for i in 0..parts.len() {
        let positions = project.mesh_positions(i).unwrap();
        let indices = project.mesh_indices(i).unwrap();
        assert_eq!(positions.len() % 3, 0);
        assert_eq!(indices.len() % 3, 0);
        let vertices = (positions.len() / 3) as u32;
        assert!(indices.iter().all(|&v| v < vertices));
    }
    assert!(project.mesh_positions(parts.len()).is_err());
}

#[test]
fn the_svg_exports() {
    let (bytes, w, h) = tiny();
    let mut project = Project::new().unwrap();
    project.load_image(&bytes, w, h).unwrap();
    let svg = String::from_utf8(project.export_svg("tiny").unwrap()).unwrap();
    assert!(svg.contains("<title>tiny</title>"), "{svg}");
}
