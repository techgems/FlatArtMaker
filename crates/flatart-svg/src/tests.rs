use crate::SvgExporter;
use flatart_domain::ExportOptions;
use flatart_ports::Exporter;
use flatart_ports::testkit::{exporter, fixtures};

#[test]
fn conforms() {
    exporter::run_vector(&SvgExporter);
}

fn export() -> String {
    let options = ExportOptions {
        name: "Nested & <test>".into(),
        thumbnail: None,
    };
    let bytes = SvgExporter
        .export(&fixtures::vector_model(), &options)
        .unwrap();
    String::from_utf8(bytes).unwrap()
}

#[test]
fn is_sized_in_mm_to_the_art() {
    // The 4 × 4 px art at 10 mm per pixel, its transparent margin dropped.
    let svg = export();
    assert!(
        svg.contains(r#"width="40mm" height="40mm" viewBox="0 0 40 40""#),
        "{svg}"
    );
}

#[test]
fn has_a_layer_per_colour_with_a_path_per_face() {
    let svg = export();
    for (name, fill) in [("Black", "#141414"), ("White", "#f0f0f0")] {
        let layer = format!(
            r#"<g id="{name}" inkscape:groupmode="layer" inkscape:label="{name}" fill="{fill}" fill-rule="evenodd">"#
        );
        assert!(svg.contains(&layer), "no layer for {name}:\n{svg}");
    }
    assert_eq!(svg.matches("<path ").count(), 2);
    // Black's ring: its outer square, then the island cut out of it.
    assert!(
        svg.contains(r#"<path d="M0 0C13.3333 0 26.6667 0 40 0"#),
        "{svg}"
    );
    assert_eq!(
        svg.matches('Z').count(),
        3,
        "two loops for the ring, one for the island"
    );
}

#[test]
fn escapes_the_title() {
    assert!(export().contains("<title>Nested &amp; &lt;test&gt;</title>"));
}
