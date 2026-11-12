//! Browser entry point, loaded by `web/src/modules/engine/engine.worker.ts`.
//!
//! - `wiring`: the adapters the browser ships (the CLI's set).
//! - `project`: one project's pipeline, in plain Rust.
//! - `types`: results that aren't domain types, with generated TypeScript.
//!
//! [`Engine`] is the JavaScript face of a [`project::Project`]: values go
//! through serde (their TypeScript types come from tsify), big buffers as
//! typed arrays, and every error becomes a thrown `Error`, never a panic.

mod project;
mod types;
mod wiring;

use project::Project;
use serde::Serialize;
use serde::de::DeserializeOwned;
use types::Slicer;
use wasm_bindgen::prelude::{JsError, JsValue, wasm_bindgen};

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// Engine version string, used by the UI to confirm the module loaded.
#[wasm_bindgen(js_name = engineVersion)]
pub fn engine_version() -> String {
    flatart_pipeline::engine_info().version.to_owned()
}

type JsResult<T> = Result<T, JsError>;

fn fail(error: flatart_domain::Error) -> JsError {
    JsError::new(&error.to_string())
}

fn to_js<T: Serialize>(value: &T) -> JsResult<JsValue> {
    value
        .serialize(&serde_wasm_bindgen::Serializer::new())
        .map_err(|e| JsError::new(&e.to_string()))
}

fn from_js<T: DeserializeOwned>(value: JsValue, what: &str) -> JsResult<T> {
    serde_wasm_bindgen::from_value(value).map_err(|e| JsError::new(&format!("invalid {what}: {e}")))
}

/// One open project. Keep one per project: its pipeline caches every stage.
#[wasm_bindgen]
pub struct Engine {
    project: Project,
}

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> JsResult<Engine> {
        Ok(Engine {
            project: Project::new().map_err(fail)?,
        })
    }

    /// Starts over on a new image (RGBA, 4 bytes per pixel, straight
    /// alpha): no edits, default settings sized to 100 mm on the longest side.
    #[wasm_bindgen(js_name = loadImage, unchecked_return_type = "LoadedImageInfo")]
    pub fn load_image(&mut self, rgba: &[u8], width: u32, height: u32) -> JsResult<JsValue> {
        to_js(&self.project.load_image(rgba, width, height).map_err(fail)?)
    }

    #[wasm_bindgen(js_name = setSettings)]
    pub fn set_settings(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "Settings")] settings: JsValue,
    ) -> JsResult<()> {
        self.project.set_settings(from_js(settings, "settings")?);
        Ok(())
    }

    #[wasm_bindgen(js_name = applyEdits)]
    pub fn apply_edits(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "EditLog")] log: JsValue,
    ) -> JsResult<()> {
        self.project.apply_edits(from_js(log, "edit log")?);
        Ok(())
    }

    /// The colour-mapping result, without its label map.
    #[wasm_bindgen(unchecked_return_type = "MappedInfo")]
    pub fn mapped(&mut self) -> JsResult<JsValue> {
        to_js(&self.project.mapped().map_err(fail)?)
    }

    /// The output label map, one cell per source pixel.
    #[wasm_bindgen(js_name = mappedLabels)]
    pub fn mapped_labels(&mut self) -> JsResult<Vec<u16>> {
        self.project.mapped_labels().map_err(fail)
    }

    #[wasm_bindgen(js_name = vectorModel, unchecked_return_type = "VectorModel")]
    pub fn vector_model(&mut self) -> JsResult<JsValue> {
        to_js(&self.project.vector_model().map_err(fail)?)
    }

    #[wasm_bindgen(unchecked_return_type = "Report")]
    pub fn report(&mut self) -> JsResult<JsValue> {
        to_js(&self.project.report().map_err(fail)?)
    }

    #[wasm_bindgen(js_name = meshParts, unchecked_return_type = "MeshPartInfo[]")]
    pub fn mesh_parts(&mut self) -> JsResult<JsValue> {
        to_js(&self.project.mesh_parts().map_err(fail)?)
    }

    /// Part `part`'s vertices: x, y, z in mm, z up.
    #[wasm_bindgen(js_name = meshPositions)]
    pub fn mesh_positions(&mut self, part: usize) -> JsResult<Vec<f32>> {
        self.project.mesh_positions(part).map_err(fail)
    }

    /// Part `part`'s triangles: vertex-index triples.
    #[wasm_bindgen(js_name = meshIndices)]
    pub fn mesh_indices(&mut self, part: usize) -> JsResult<Vec<u32>> {
        self.project.mesh_indices(part).map_err(fail)
    }

    #[wasm_bindgen(js_name = export3mf)]
    pub fn export_3mf(
        &mut self,
        #[wasm_bindgen(unchecked_param_type = "Slicer")] slicer: JsValue,
        name: &str,
    ) -> JsResult<Vec<u8>> {
        let slicer: Slicer = from_js(slicer, "slicer")?;
        self.project.export_3mf(slicer, name).map_err(fail)
    }

    #[wasm_bindgen(js_name = exportSvg)]
    pub fn export_svg(&mut self, name: &str) -> JsResult<Vec<u8>> {
        self.project.export_svg(name).map_err(fail)
    }
}

#[cfg(test)]
mod tests;
