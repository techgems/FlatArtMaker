use crate::export;
use flatart_domain::VectorModel;
use flatart_ports::testkit::fixtures;

/// The model survives the trip from the main engine (serialised) to this
/// module (deserialised), and exports the same STEP file.
#[test]
fn a_model_read_back_exports_the_same_file() {
    let model = fixtures::vector_model();
    let json = serde_json::to_string(&model).unwrap();
    let back: VectorModel = serde_json::from_str(&json).unwrap();
    assert_eq!(back, model);
    assert_eq!(
        export(&back, "FlatArt").unwrap(),
        export(&model, "FlatArt").unwrap()
    );
}

#[test]
fn an_out_of_range_height_is_refused() {
    let mut json = serde_json::to_value(fixtures::vector_model()).unwrap();
    json["extrusion"]["colour"] = 0.0.into();
    assert!(serde_json::from_value::<VectorModel>(json).is_err());
}
