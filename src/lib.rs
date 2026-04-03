#[cfg(not(target_arch = "wasm32"))]
pub mod api;
pub mod app;
pub mod components;
#[cfg(not(target_arch = "wasm32"))]
pub mod db;
pub mod models;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn hydrate() {
    leptos::mount::mount_to_body(app::App);
}
