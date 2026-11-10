//! Interfaces for the in-house stages that aren't built yet.
//!
//! **Provisional:** these let the pipeline call in-house code without
//! depending on it. `MeshBuilder`'s signature and contract are finished, with
//! a conformance suite, once task 10 implements it.

use crate::Triangulator;
use flatart_domain::settings::MeshSettings;
use flatart_domain::{MeshModel, Result, VectorModel};

/// Builds the watertight mesh from the vector model (step 8).
pub trait MeshBuilder {
    fn build(
        &self,
        model: &VectorModel,
        settings: &MeshSettings,
        triangulator: &dyn Triangulator,
    ) -> Result<MeshModel>;
}
