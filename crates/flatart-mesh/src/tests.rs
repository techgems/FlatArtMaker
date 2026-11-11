use crate::Extruder;
use flatart_domain::settings::{
    CurveTolerance, Extrusion, LayerHeight, MeshSettings, PrintScale, TraceUpscale,
};
use flatart_domain::{
    ColourName, Label, LabelMap, Mm, MmPerPx, OutputColour, OutputColourId, OutputPalette, Rgb,
    VectorModel,
};
use flatart_fit_kurbo::KurboFitter;
use flatart_ports::testkit::{fixtures, mesh, mesh_builder};
use flatart_ports::{BorderTracer, MeshBuilder};
use flatart_trace_graph::GraphTracer;
use proptest::prelude::*;

use flatart_tri_itriangle::ITriangulator;

#[test]
fn conforms() {
    mesh_builder::run(&Extruder, &ITriangulator);
}

fn palette(n: u16) -> OutputPalette {
    let names = ["Black", "White", "Red", "Blue"];
    let colours = (0..n)
        .map(|i| OutputColour {
            id: OutputColourId(i),
            name: ColourName::new(names[usize::from(i)]).unwrap(),
            display: Rgb { r: 0, g: 0, b: 0 },
        })
        .collect();
    OutputPalette::new(colours, Some(OutputColourId(0))).unwrap()
}

fn model(labels: &LabelMap, base: f64, fill: bool) -> VectorModel {
    let borders = GraphTracer
        .trace(labels, &fixtures::trace_settings(), &KurboFitter)
        .unwrap();
    VectorModel {
        borders,
        palette: palette(4),
        scale: PrintScale::new(MmPerPx(0.5)).unwrap(),
        extrusion: Extrusion {
            colour: LayerHeight::new(Mm(1.0)).unwrap(),
            base: LayerHeight::optional(Mm(base)).unwrap(),
            fill_base_holes: fill,
        },
    }
}

fn settings() -> MeshSettings {
    MeshSettings {
        curve_tolerance: CurveTolerance::new(Mm(0.02)).unwrap(),
    }
}

fn label_map() -> impl Strategy<Value = LabelMap> {
    (1u32..9, 1u32..9).prop_flat_map(|(w, h)| {
        let cell = prop_oneof![
            3 => (0u16..4).prop_map(Label),
            1 => Just(Label::OUTSIDE),
        ];
        proptest::collection::vec(cell, (w * h) as usize).prop_map(move |cells| {
            LabelMap::new(w, h, TraceUpscale::new(1).unwrap(), cells).unwrap()
        })
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn traced_maps_mesh_watertight(labels in label_map(), base in prop_oneof![Just(0.0), Just(2.0)], fill: bool) {
        let model = model(&labels, base, fill);
        let built = Extruder.build(&model, &settings(), &ITriangulator).unwrap();
        mesh::assert_model_watertight(&built);
        if base == 0.0 {
            prop_assert!(built.parts.iter().all(|p| p.kind == flatart_domain::PartKind::Colour));
        }
    }
}
