use crate::Triangulator;
use flatart_domain::settings::MeshSettings;
use flatart_domain::{MeshModel, Result, VectorModel};

/// Builds the printable parts from the vector model (Pipeline step 8).
///
/// B is `model.extrusion.base` (`None` for no base) and C is
/// `model.extrusion.colour`.
///
/// # Contract
/// - **Flattened once:** each border becomes one polyline, within
///   `settings.curve_tolerance` of its curve, ending exactly at the border's
///   ends. Every part uses those same points, so parts that meet along a
///   border share identical edge points.
/// - **Placement:** millimetres, x right and y up (the image's y flipped).
///   The outline's lowest x and y are 0, so the footprint is the art's
///   extent in source pixels times `model.scale`.
/// - **With a base:** the first part is the `Base`, in the palette's base
///   colour. It covers every face from z = 0 to B, and the base colour's own
///   faces rise inside it to B + C. With `fill_base_holes`, outside areas
///   enclosed by the art are filled from 0 to B too.
/// - **Colour parts:** one per output colour that has faces, in palette
///   order, leaving out the base colour when there's a base. Each runs from
///   B to B + C, or from 0 to C with no base.
/// - **Watertight:** every part's mesh is closed and faces outward, as
///   `testkit::mesh::assert_watertight` checks.
/// - **Triangulation** goes only through `triangulator`.
/// - **Deterministic.**
pub trait MeshBuilder {
    fn build(
        &self,
        model: &VectorModel,
        settings: &MeshSettings,
        triangulator: &dyn Triangulator,
    ) -> Result<MeshModel>;
}
