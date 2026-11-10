//! Ties the steps together into `BorderTracer`.

use crate::curves::Fitting;
use crate::grid::Grid;
use crate::{chains, faces};
use flatart_domain::settings::TraceSettings;
use flatart_domain::{
    Border, BorderEnds, BorderMap, Junction, JunctionId, LabelMap, Px, PxPoint, Result,
};
use flatart_ports::{BorderTracer, CurveFitter};

/// A turn is a real corner, not a staircase step, when the straight runs on
/// both sides are at least this many source pixels long.
const CORNER_RUN_PX: i64 = 2;

/// Traces the pixel-edge graph, simplifies each border's staircase, and fits
/// it through the injected `CurveFitter`.
#[derive(Debug, Clone, Copy, Default)]
pub struct GraphTracer;

impl BorderTracer for GraphTracer {
    fn trace(
        &self,
        labels: &LabelMap,
        settings: &TraceSettings,
        fitter: &dyn CurveFitter,
    ) -> Result<BorderMap> {
        let grid = Grid::new(labels);
        let upscale = i64::from(labels.upscale().get());
        let k = upscale as f64;
        let traced = chains::trace(&grid);
        let curves = Fitting {
            chains: &traced.chains,
            upscale: k,
            corner_run: CORNER_RUN_PX * upscale,
            settings: settings.fit,
            fitter,
        }
        .run()?;
        let faces = faces::assemble(&grid, &traced.chains)?;

        let junctions = traced
            .junctions
            .iter()
            .map(|&(x, y)| Junction {
                at: PxPoint::new(x as f64 / k, y as f64 / k),
            })
            .collect();
        let borders = traced
            .chains
            .iter()
            .zip(curves)
            .map(|(chain, curve)| Border {
                ends: match chain.ends {
                    Some((start, end)) => BorderEnds::Open {
                        start: JunctionId(start),
                        end: JunctionId(end),
                    },
                    None => BorderEnds::Closed,
                },
                curve,
                left: chain.left,
                right: chain.right,
            })
            .collect();
        Ok(BorderMap {
            width: Px(grid.width as f64 / k),
            height: Px(grid.height as f64 / k),
            junctions,
            borders,
            faces,
        })
    }
}
