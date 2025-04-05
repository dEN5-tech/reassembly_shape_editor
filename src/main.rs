// Main application entry point
mod visual;
mod data_structures;
mod ui;
mod shape_editor;
mod geometry;
mod ast;
mod lua_parser;
mod project_generator;
mod translations;

use eframe::{self, egui};
use shape_editor::ShapeEditor;
use std::env;

fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    // Check if this is a project generation request
    if args.len() > 1 && args[1] == "--generate-project" {
        let project_name = if args.len() > 2 { &args[2] } else { "reassembly_mod" };
        match project_generator::generate_project(project_name) {
            Ok(_) => println!("Project '{}' created successfully!", project_name),
            Err(err) => eprintln!("Error creating project: {}", err),
        }
        return;
    }
    
    // Normal application startup
    let app = ShapeEditor::new();
    let mut native_options = eframe::NativeOptions::default();
    
    // Set window size
    native_options.initial_window_size = Some(egui::Vec2::new(1200.0, 800.0));
    
    eframe::run_native(
        &translations::t("app_title"), 
        native_options, 
        Box::new(|_cc| Box::new(app))
    );
}