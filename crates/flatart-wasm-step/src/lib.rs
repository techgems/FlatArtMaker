//! Browser entry point for STEP export, loaded on demand by
//! `web/src/modules/engine/engine.worker.ts` the first time a STEP file is
//! asked for. It's a module of its own so the main engine never carries
//! truck (SPEC "Bindings").
//!
//! The worker hands it the main engine's `VectorModel` as a JavaScript
//! value; every error becomes a thrown `Error`, never a panic.

use flatart_domain::{ExportOptions, Result, VectorModel};
use flatart_ports::Exporter;
use flatart_step_truck::StepExporter;
use wasm_bindgen::prelude::{JsError, JsValue, wasm_bindgen};

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// The STEP file for `model`, its root part named `name`.
pub fn export(model: &VectorModel, name: &str) -> Result<Vec<u8>> {
    let options = ExportOptions {
        name: name.to_owned(),
        thumbnail: None,
    };
    StepExporter.export(model, &options)
}

/// `model` is the main engine's `vectorModel()`.
#[wasm_bindgen(js_name = exportStep)]
pub fn export_step(
    #[wasm_bindgen(unchecked_param_type = "VectorModel")] model: JsValue,
    name: &str,
) -> std::result::Result<Vec<u8>, JsError> {
    let model: VectorModel = serde_wasm_bindgen::from_value(model)
        .map_err(|e| JsError::new(&format!("invalid vector model: {e}")))?;
    export(&model, name).map_err(|e| JsError::new(&e.to_string()))
}

#[cfg(test)]
mod tests;
