use wasm_bindgen::prelude::*;
use eframe::wasm_bindgen::{self, JsValue};

// Import modules
mod visual;
mod data_structures;
mod ui;
mod shape_editor;
mod geometry;
mod ast;
mod lua_parser;
mod project_generator;
mod translations;

use shape_editor::ShapeEditor;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(all(feature = "wee_alloc", target_arch = "wasm32"))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is the entry point for the web app
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    // Initialize logging for wasm
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Debug).expect("Failed to initialize logging");
    }

    let app = ShapeEditor::new();
    
    eframe::start_web(
        canvas_id,
        Box::new(|_cc| Box::new(app)),
    )
} 