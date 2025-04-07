#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{File, FileReader, FileList, Event, HtmlInputElement};
#[cfg(target_arch = "wasm32")]
use js_sys::Reflect;

// Import modules
mod visual;
mod data_structures;
mod ui;
mod shape_editor;
mod geometry;
mod ast;
mod project_generator;
mod translations;
mod parser;
mod serializer;

// Re-export public items
pub use parser::{parse_shapes_content, parse_shapes_file, ParseError, ParserErrorKind};
pub use serializer::serialize_shapes_file;
pub use shape_editor::ShapeEditor;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(all(feature = "wee_alloc", target_arch = "wasm32"))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Store a global reference to the shape editor for file input callbacks
#[cfg(target_arch = "wasm32")]
static mut SHAPE_EDITOR_INSTANCE: Option<*mut ShapeEditor> = None;

// This is the entry point for the web app
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    // Initialize logging for wasm
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Debug).expect("Failed to initialize logging");

    let app = ShapeEditor::new();
    
    // Store a reference to the shape editor for file input callbacks
    unsafe {
        SHAPE_EDITOR_INSTANCE = Some(Box::into_raw(Box::new(app)));
    }
    
    // Set up the file input handler
    setup_file_input_handler()?;
    
    // Get the app instance from our global reference
    let app_instance = unsafe {
        if let Some(ptr) = SHAPE_EDITOR_INSTANCE {
            Box::from_raw(ptr)
        } else {
            return Err(JsValue::from_str("Failed to initialize app"));
        }
    };
    
    // Create an owned version of canvas_id that can be moved into the closure
    let canvas_id_owned = canvas_id.to_owned();
    
    // Use eframe for web startup
    wasm_bindgen_futures::spawn_local(async move {
        eframe::start_web(
            &canvas_id_owned,
            Box::new(|_cc| app_instance),
        )
        .expect("Failed to start eframe");
    });
    
    Ok(())
}

// Set up the file input handler
#[cfg(target_arch = "wasm32")]
fn setup_file_input_handler() -> Result<(), JsValue> {
    use wasm_bindgen::closure::Closure;
    
    // Create the input element if it doesn't exist
    ShapeEditor::create_file_input_element();
    
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    
    if let Some(input_element) = document.get_element_by_id("file-input") {
        let input = input_element.dyn_into::<HtmlInputElement>()?;
        
        // Create a closure for the onchange event
        let onchange_callback = Closure::wrap(Box::new(move |event: Event| {
            let target = event.target().unwrap();
            let input: HtmlInputElement = target.dyn_into().unwrap();
            
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    let filename = file.name();
                    
                    // Create a FileReader to read the file
                    let reader = FileReader::new().unwrap();
                    let reader_clone = reader.clone();
                    
                    // Create a closure for the onload event
                    let onload_callback = Closure::wrap(Box::new(move |_: Event| {
                        let result = reader_clone.result().unwrap();
                        let text = result.as_string().unwrap();
                        
                        // Call the shape editor's handle_file_content method
                        unsafe {
                            if let Some(editor_ptr) = SHAPE_EDITOR_INSTANCE {
                                let editor = &mut *editor_ptr;
                                editor.handle_file_content(text, filename.clone());
                            }
                        }
                    }) as Box<dyn FnMut(Event)>);
                    
                    // Set the onload handler
                    reader.set_onload(Some(onload_callback.as_ref().unchecked_ref()));
                    
                    // Start reading the file as text
                    reader.read_as_text(&file).unwrap();
                    
                    // Leak the closure to keep it alive
                    onload_callback.forget();
                }
            }
        }) as Box<dyn FnMut(Event)>);
        
        // Set the onchange handler
        input.set_onchange(Some(onchange_callback.as_ref().unchecked_ref()));
        
        // Leak the closure to keep it alive
        onchange_callback.forget();
    }
    
    Ok(())
}
