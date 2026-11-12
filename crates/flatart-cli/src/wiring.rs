//! Every adapter, wired into a pipeline.

use crate::Fitter;
use flatart_3mf::{BambuDialect, PrusaDialect, ThreeMfExporter};
use flatart_domain::Result;
use flatart_fit_kurbo::KurboFitter;
use flatart_fit_visioncortex::VisionCortexFitter;
use flatart_mapping::MergeMapper;
use flatart_mesh::Extruder;
use flatart_pipeline::Pipeline;
use flatart_ports::CurveFitter;
use flatart_quant_kmeans::KmeansQuantizer;
use flatart_raster::PixelStages;
use flatart_step_truck::StepExporter;
use flatart_svg::SvgExporter;
use flatart_trace_graph::GraphTracer;
use flatart_tri_itriangle::ITriangulator;

/// A pipeline with every stage and exporter: `svg`, `step`, `3mf-prusa`
/// and `3mf-bambu`, fitting curves with `fitter`.
pub fn pipeline(fitter: Fitter) -> Result<Pipeline> {
    let curve_fitter: Box<dyn CurveFitter> = match fitter {
        Fitter::Kurbo => Box::new(KurboFitter),
        Fitter::VisionCortex => Box::new(VisionCortexFitter),
    };
    Pipeline::builder()
        .raster_stages(Box::new(PixelStages))
        .color_quantizer(Box::new(KmeansQuantizer))
        .colour_mapper(Box::new(MergeMapper))
        .border_tracer(Box::new(GraphTracer))
        .curve_fitter(curve_fitter)
        .mesh_builder(Box::new(Extruder))
        .triangulator(Box::new(ITriangulator))
        .vector_exporter(Box::new(SvgExporter))
        .vector_exporter(Box::new(StepExporter))
        .mesh_exporter(Box::new(ThreeMfExporter::new(Box::new(PrusaDialect))))
        .mesh_exporter(Box::new(ThreeMfExporter::new(Box::new(BambuDialect))))
        .build()
}
