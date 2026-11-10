//! Collects the adapters a wiring crate hands in.

use crate::Pipeline;
use flatart_domain::{Error, MeshModel, Result, VectorModel};
use flatart_ports::{
    BorderTracer, ColorQuantizer, ColourMapper, CurveFitter, Exporter, MeshBuilder, RasterStages,
    Triangulator,
};
use std::collections::HashSet;

/// The adapters, one per port. A missing one makes its stage return
/// `Error::NotImplemented`.
#[derive(Default)]
pub(crate) struct Ports {
    pub raster: Option<Box<dyn RasterStages>>,
    pub quantizer: Option<Box<dyn ColorQuantizer>>,
    pub mapper: Option<Box<dyn ColourMapper>>,
    pub tracer: Option<Box<dyn BorderTracer>>,
    pub fitter: Option<Box<dyn CurveFitter>>,
    pub mesher: Option<Box<dyn MeshBuilder>>,
    pub triangulator: Option<Box<dyn Triangulator>>,
    pub vector_exporters: Vec<Box<dyn Exporter<VectorModel>>>,
    pub mesh_exporters: Vec<Box<dyn Exporter<MeshModel>>>,
}

/// Builds a [`Pipeline`]. Created by [`Pipeline::builder`].
#[derive(Default)]
pub struct PipelineBuilder {
    ports: Ports,
}

impl PipelineBuilder {
    /// Stages 1, 4, 5 and 6.
    pub fn raster_stages(mut self, adapter: Box<dyn RasterStages>) -> Self {
        self.ports.raster = Some(adapter);
        self
    }

    /// Stage 2.
    pub fn color_quantizer(mut self, adapter: Box<dyn ColorQuantizer>) -> Self {
        self.ports.quantizer = Some(adapter);
        self
    }

    /// Stage 3.
    pub fn colour_mapper(mut self, adapter: Box<dyn ColourMapper>) -> Self {
        self.ports.mapper = Some(adapter);
        self
    }

    /// Stage 7, with [`PipelineBuilder::curve_fitter`].
    pub fn border_tracer(mut self, adapter: Box<dyn BorderTracer>) -> Self {
        self.ports.tracer = Some(adapter);
        self
    }

    pub fn curve_fitter(mut self, adapter: Box<dyn CurveFitter>) -> Self {
        self.ports.fitter = Some(adapter);
        self
    }

    /// Stage 8, with [`PipelineBuilder::triangulator`].
    pub fn mesh_builder(mut self, adapter: Box<dyn MeshBuilder>) -> Self {
        self.ports.mesher = Some(adapter);
        self
    }

    pub fn triangulator(mut self, adapter: Box<dyn Triangulator>) -> Self {
        self.ports.triangulator = Some(adapter);
        self
    }

    /// Stage 9, for SVG and STEP. Can be called more than once.
    pub fn vector_exporter(mut self, adapter: Box<dyn Exporter<VectorModel>>) -> Self {
        self.ports.vector_exporters.push(adapter);
        self
    }

    /// Stage 9, for 3MF. Can be called more than once, e.g. once per slicer dialect.
    pub fn mesh_exporter(mut self, adapter: Box<dyn Exporter<MeshModel>>) -> Self {
        self.ports.mesh_exporters.push(adapter);
        self
    }

    /// Fails when two exporters share an id.
    pub fn build(self) -> Result<Pipeline> {
        let mut ids = HashSet::new();
        let vector = self.ports.vector_exporters.iter().map(|e| e.id());
        let mesh = self.ports.mesh_exporters.iter().map(|e| e.id());
        for id in vector.chain(mesh) {
            if !ids.insert(id) {
                return Err(Error::InvalidInput(format!(
                    "two exporters have the id `{id}`"
                )));
            }
        }
        Ok(Pipeline::new(self.ports))
    }
}
