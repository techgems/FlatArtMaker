//! Browser entry point, loaded by `web/src/modules/engine/engine.worker.ts`.

use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// Engine version string, used by the UI to confirm the module loaded.
#[wasm_bindgen(js_name = engineVersion)]
pub fn engine_version() -> String {
    flatart_pipeline::engine_info().version.to_owned()
}
