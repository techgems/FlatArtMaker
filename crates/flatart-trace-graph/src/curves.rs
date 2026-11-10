//! Fitting every border's curve, without crossings.
//!
//! Each border is first fitted smoothly. Where curves cross, the borders
//! involved are refitted one level tighter, and so on until nothing crosses.
//! The last level is the pixel edges themselves (every turn a corner), and
//! pixel edges of different borders only meet at junctions.

use crate::chains::Chain;
use crate::crossings;
use crate::simplify::{self, Point};
use flatart_domain::settings::{CornerThreshold, FitSettings, SmoothingTolerance};
use flatart_domain::{CubicBezier, Degrees, Error, Px, PxPoint, Result, Stage};
use flatart_ports::CurveFitter;

/// How closely a border follows its pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Level {
    /// The smooth points, at the user's settings.
    Smooth,
    /// The smooth points, at the tightest tolerance.
    Tight,
    /// The pixel edges, every turn a corner.
    Edges,
}

impl Level {
    fn next(self) -> Option<Level> {
        match self {
            Level::Smooth => Some(Level::Tight),
            Level::Tight => Some(Level::Edges),
            Level::Edges => None,
        }
    }
}

/// Fits each chain's curve.
pub struct Fitting<'a> {
    pub chains: &'a [Chain],
    /// Cells per source pixel.
    pub upscale: f64,
    /// The shortest run, in cells, that counts as a side of a real corner.
    pub corner_run: i64,
    pub settings: FitSettings,
    pub fitter: &'a dyn CurveFitter,
}

impl Fitting<'_> {
    fn points(&self, chain: &Chain, level: Level) -> Vec<PxPoint> {
        let cells: Vec<Point> = match level {
            Level::Edges => simplify::edge_points(&chain.vertices),
            _ => simplify::smooth_points(&chain.vertices, chain.ends.is_none(), self.corner_run),
        };
        cells
            .into_iter()
            .map(|(x, y)| PxPoint::new(x / self.upscale, y / self.upscale))
            .collect()
    }

    fn settings(&self, level: Level) -> Result<FitSettings> {
        let tightest = SmoothingTolerance::new(Px(SmoothingTolerance::MIN))?;
        Ok(match level {
            Level::Smooth => self.settings,
            Level::Tight => FitSettings {
                tolerance: tightest,
                ..self.settings
            },
            Level::Edges => FitSettings {
                tolerance: tightest,
                corner: CornerThreshold::new(Degrees(0.0))?,
            },
        })
    }

    fn fit(&self, chain: &Chain, level: Level) -> Result<Vec<CubicBezier>> {
        let curves = self
            .fitter
            .fit(&self.points(chain, level), &self.settings(level)?)?;
        if curves.is_empty() {
            return Err(Error::StageFailed {
                stage: Stage::Trace,
                message: "the curve fitter returned no curves".into(),
            });
        }
        Ok(curves)
    }

    /// Every chain's curves, in order, with no two crossing.
    pub fn run(&self) -> Result<Vec<Vec<CubicBezier>>> {
        let mut levels = vec![Level::Smooth; self.chains.len()];
        let mut curves = self
            .chains
            .iter()
            .map(|chain| self.fit(chain, Level::Smooth))
            .collect::<Result<Vec<_>>>()?;
        loop {
            let crossing = crossings::crossing(&curves);
            if crossing.is_empty() {
                return Ok(curves);
            }
            let mut refitted = false;
            for i in crossing {
                if let Some(next) = levels[i].next() {
                    levels[i] = next;
                    curves[i] = self.fit(&self.chains[i], next)?;
                    refitted = true;
                }
            }
            if !refitted {
                return Err(Error::StageFailed {
                    stage: Stage::Trace,
                    message: "borders still cross along the pixel edges; \
                              the curve fitter didn't keep corners straight"
                        .into(),
                });
            }
        }
    }
}
