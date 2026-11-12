use crate::StepExporter;
use flatart_ports::testkit::exporter;

#[test]
fn conforms() {
    exporter::run_vector(&StepExporter);
}

#[test]
fn the_fixture_reads_back_as_closed_named_solids() {
    use flatart_domain::ExportOptions;
    use flatart_ports::Exporter;
    use flatart_ports::testkit::fixtures;
    let options = ExportOptions {
        name: "FlatArt".into(),
        thumbnail: None,
    };
    let bytes = StepExporter
        .export(&fixtures::vector_model(), &options)
        .unwrap();
    let step = String::from_utf8(bytes).unwrap();
    let summary = crate::check::check(&step).unwrap();
    eprintln!("{summary:#?}");
    assert_eq!(
        summary.schema,
        "AUTOMOTIVE_DESIGN { 1 0 10303 214 1 1 1 1 }"
    );
    assert_eq!(summary.products[0], "FlatArt");
    assert!(!summary.bodies.is_empty());
}
