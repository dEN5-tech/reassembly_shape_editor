// Shape editor module
use eframe::egui;
use egui::*;
use std::io;
use std::fs;
use std::path::Path;
use std::str::FromStr;

use crate::data_structures::{Shape as AppShape, Vertex, Port, PortType};
use crate::geometry::round_to;
use crate::ui::*;
use crate::visual::*;
use crate::parser::{parse_shapes_content, ParseError};
use crate::serializer::serialize_shapes_file;

#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

// Maximum size for undo history
const MAX_UNDO_HISTORY: usize = 100;

// Главная структура приложения
pub struct ShapeEditor {
    pub shapes: Vec<AppShape>,
    pub current_shape_idx: usize,
    pub grid_size: f32,
    pub show_grid: bool,
    pub snap_to_grid: bool,
    pub zoom: f32,
    pub pan: Vec2,
    pub dragging: bool,
    pub last_mouse_pos: Pos2,
    pub export_path: String,
    pub import_path: String,
    // Undo/redo history
    undo_history: Vec<Vec<AppShape>>,
    redo_history: Vec<Vec<AppShape>>,
    // Store state for middle-mouse zoom
    pub middle_drag_ongoing: bool,
    pub zoom_center: Pos2,
    // Game UI state
    pub active_tab: usize,
    pub resources: i32,
    pub points: i32,
    // Settings and UI state
    pub status_message: Option<String>,
    pub status_time: f32,
    // Error dialog state
    pub show_error_dialog: bool,
    pub error_title: String,
    pub error_message: String,
}

impl ShapeEditor {
    pub fn new() -> Self {
        let mut shapes = Vec::new();
        shapes.push(AppShape::new(1));
        
        Self {
            shapes: shapes.clone(),
            current_shape_idx: 0,
            grid_size: 10.0,
            show_grid: true,
            snap_to_grid: true,
            zoom: 1.0,
            pan: Vec2::new(0.0, 0.0),
            dragging: false,
            last_mouse_pos: Pos2::new(0.0, 0.0),
            export_path: "shapes.lua".to_string(),
            import_path: "shapes.lua".to_string(),
            undo_history: vec![shapes],
            redo_history: Vec::new(),
            middle_drag_ongoing: false,
            zoom_center: Pos2::ZERO,
            active_tab: 0,  // Default to Shapes tab
            resources: 500,
            points: 200,
            status_message: None,
            status_time: 0.0,
            // Initialize error dialog state
            show_error_dialog: false,
            error_title: String::new(),
            error_message: String::new(),
        }
    }
    
    // Show an error dialog with the given title and message
    pub fn show_error(&mut self, title: &str, message: &str) {
        self.error_title = title.to_string();
        self.error_message = message.to_string();
        self.show_error_dialog = true;
    }
    
    // Save current state to undo history
    pub fn save_state(&mut self) {
        self.redo_history.clear(); // Clear redo history when new action is performed
        
        // Only save if there's a difference from the last state
        if let Some(last_state) = self.undo_history.last() {
            if last_state == &self.shapes {
                return; // No change, no need to save
            }
        }
        
        self.undo_history.push(self.shapes.clone());
        
        // Limit history size
        if self.undo_history.len() > MAX_UNDO_HISTORY {
            self.undo_history.remove(0);
        }
    }
    
    // Undo last action
    pub fn undo(&mut self) {
        if self.undo_history.len() > 1 { // Keep at least one state in undo history
            // Save current state to redo
            self.redo_history.push(self.shapes.clone());
            
            // Pop the current state from undo (it's the one we're at)
            self.undo_history.pop();
            
            // Use the last state from undo
            if let Some(previous_state) = self.undo_history.last() {
                self.shapes = previous_state.clone();
                
                // Make sure current_shape_idx is valid
                if self.current_shape_idx >= self.shapes.len() && !self.shapes.is_empty() {
                    self.current_shape_idx = self.shapes.len() - 1;
                }
            }
        }
    }
    
    // Redo previously undone action
    pub fn redo(&mut self) {
        if let Some(next_state) = self.redo_history.pop() {
            // Save current state to undo
            self.undo_history.push(self.shapes.clone());
            
            // Apply the redo state
            self.shapes = next_state;
            
            // Make sure current_shape_idx is valid
            if self.current_shape_idx >= self.shapes.len() && !self.shapes.is_empty() {
                self.current_shape_idx = self.shapes.len() - 1;
            }
        }
    }
    
    // Преобразование координаты экрана в координату формы
    pub fn screen_to_shape_coords(&self, screen_pos: Pos2, rect: Rect) -> Vertex {
        let center = rect.center();
        let x = (screen_pos.x - center.x) / self.zoom - self.pan.x;
        let y = (screen_pos.y - center.y) / self.zoom - self.pan.y;
        
        if self.snap_to_grid {
            Vertex {
                x: round_to(x, self.grid_size),
                y: round_to(y, self.grid_size),
            }
        } else {
            Vertex { x, y }
        }
    }
    
    // Преобразование координаты формы в координату экрана
    pub fn shape_to_screen_coords(&self, shape_pos: &Vertex, rect: Rect) -> Pos2 {
        let center = rect.center();
        Pos2 {
            x: center.x + (shape_pos.x + self.pan.x) * self.zoom,
            y: center.y + (shape_pos.y + self.pan.y) * self.zoom,
        }
    }
    
    // Добавление новой формы
    pub fn add_shape(&mut self) {
        self.save_state();
        
        let id = self.shapes.len() + 1;
        self.shapes.push(AppShape::new(id));
        self.current_shape_idx = self.shapes.len() - 1;
    }
    
    // Add or update a vertex
    pub fn add_or_update_vertex(&mut self, shape_idx: usize, vertex: Vertex, vertex_idx: Option<usize>) {
        self.save_state();
        
        if let Some(idx) = vertex_idx {
            if idx < self.shapes[shape_idx].vertices.len() {
                self.shapes[shape_idx].vertices[idx] = vertex;
            }
        } else {
            self.shapes[shape_idx].vertices.push(vertex);
            self.shapes[shape_idx].selected_vertex = Some(self.shapes[shape_idx].vertices.len() - 1);
        }
    }
    
    // Remove a vertex
    pub fn remove_vertex(&mut self, shape_idx: usize, vertex_idx: usize) {
        if vertex_idx < self.shapes[shape_idx].vertices.len() {
            self.save_state();
            
            self.shapes[shape_idx].vertices.remove(vertex_idx);
            
            // Update selected vertex
            if let Some(selected) = self.shapes[shape_idx].selected_vertex {
                if selected >= vertex_idx {
                    self.shapes[shape_idx].selected_vertex = if selected > 0 { Some(selected - 1) } else { None };
                }
            }
            
            // Update ports affected by vertex removal
            let mut i = 0;
            while i < self.shapes[shape_idx].ports.len() {
                let port = &mut self.shapes[shape_idx].ports[i];
                
                // If port is on the removed edge or after, adjust or remove it
                if port.edge >= vertex_idx {
                    if port.edge == vertex_idx {
                        // Remove port on the deleted edge
                        self.shapes[shape_idx].ports.remove(i);
                        continue;
                    } else {
                        // Adjust edge index for ports after the deleted vertex
                        port.edge -= 1;
                    }
                }
                
                i += 1;
            }
        }
    }
    
    // Add a port
    pub fn add_port(&mut self, shape_idx: usize, port: Port) {
        self.save_state();
        self.shapes[shape_idx].ports.push(port);
    }
    
    // Remove a port
    pub fn remove_port(&mut self, shape_idx: usize, port_idx: usize) {
        if port_idx < self.shapes[shape_idx].ports.len() {
            self.save_state();
            
            self.shapes[shape_idx].ports.remove(port_idx);
            
            // Update selected port
            if let Some(selected) = self.shapes[shape_idx].selected_port {
                if selected >= port_idx {
                    self.shapes[shape_idx].selected_port = if selected > 0 { Some(selected - 1) } else { None };
                }
            }
        }
    }
    
    // Handle zoom at specific position
    pub fn zoom_at(&mut self, screen_pos: Pos2, rect: Rect, delta: f32) {
        let old_zoom = self.zoom;
        
        // Adjust zoom
        self.zoom = (self.zoom * (1.0 + delta * 0.1)).clamp(0.1, 10.0);
        
        // Calculate world position before zoom
        let center = rect.center();
        let before_x = (screen_pos.x - center.x) / old_zoom;
        let before_y = (screen_pos.y - center.y) / old_zoom;
        
        // Calculate world position after zoom
        let after_x = (screen_pos.x - center.x) / self.zoom;
        let after_y = (screen_pos.y - center.y) / self.zoom;
        
        // Adjust panning to keep the world position constant under cursor
        self.pan.x += after_x - before_x;
        self.pan.y += after_y - before_y;
    }
    
    // Экспорт всех форм в файл shapes.lua
    pub fn export_shapes(&self) -> Result<(), std::io::Error> {
        // Convert shapes to AST shapes for export
        let mut ast_shapes = Vec::new();
        for app_shape in &self.shapes {
            ast_shapes.push(self.convert_to_ast_shape(app_shape));
        }
        
        // Create shapes file
        let shapes_file = crate::ast::ShapesFile { shapes: ast_shapes };
        
        // Serialize to Lua format
        let lua_content = serialize_shapes_file(&shapes_file);
        
        // Write to file
        #[cfg(not(target_arch = "wasm32"))]
        {
            match fs::write(&self.export_path, lua_content) {
                Ok(_) => Ok(()),
                Err(e) => {
                    // This error will be displayed in the UI via the error dialog
                    Err(e)
                }
            }
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            self.download_file(&lua_content);
            Ok(())
        }
    }
    
    // Download file in browser (WebAssembly target)
    #[cfg(target_arch = "wasm32")]
    fn download_file(&self, content: &str) {
        use wasm_bindgen::JsCast;
        use js_sys::Reflect;
        use wasm_bindgen::JsValue;
        
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        
        // Create a Blob with the content
        let blob_parts = js_sys::Array::new();
        blob_parts.push(&js_sys::JsString::from(content));
        
        let blob = web_sys::Blob::new_with_str_sequence(&blob_parts).unwrap();
        
        // Create object URL by calling the browser's createObjectURL function
        // Using js_sys::Reflect to call the function
        let global = js_sys::global();
        let url_obj = global.unchecked_ref::<web_sys::Window>();
        
        // Create an object URL for the blob
        let url_create_fn = Reflect::get(&url_obj, &JsValue::from_str("URL")).unwrap();
        let create_obj_url = Reflect::get(
            &url_create_fn, 
            &JsValue::from_str("createObjectURL")
        ).unwrap();
        
        let url = Reflect::apply(
            &create_obj_url.dyn_ref().unwrap(),
            &url_create_fn,
            &js_sys::Array::of1(&blob)
        ).unwrap().as_string().unwrap();
        
        // Create a temporary anchor element for downloading
        let a = document.create_element("a").unwrap();
        let a_element = a.dyn_into::<web_sys::HtmlElement>().unwrap();
        
        // Set up the anchor to trigger download
        a_element.set_attribute("href", &url).unwrap();
        a_element.set_attribute("download", &self.export_path).unwrap();
        a_element.style().set_property("display", "none").unwrap();
        
        // Add to document, click, and remove
        document.body().unwrap().append_child(&a_element).unwrap();
        a_element.click();
        document.body().unwrap().remove_child(&a_element).unwrap();
        
        // Clean up the URL by calling revokeObjectURL
        let revoke_obj_url = Reflect::get(
            &url_create_fn, 
            &JsValue::from_str("revokeObjectURL")
        ).unwrap();
        
        Reflect::apply(
            &revoke_obj_url.dyn_ref().unwrap(),
            &url_create_fn,
            &js_sys::Array::of1(&JsValue::from_str(&url))
        ).unwrap();
    }
    
    // Import shapes from Lua file
    pub fn import_shapes(&mut self) -> Result<(), io::Error> {
        self.save_state();
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            let content = match fs::read_to_string(&self.import_path) {
                Ok(content) => content,
                Err(e) => {
                    self.show_error("Import Error", &format!("Failed to read file: {}", e));
                    return Err(e);
                }
            };
            
            match self.parse_lua_shapes(&content) {
                Ok(shapes) => {
                    if !shapes.is_empty() {
                        self.shapes = shapes;
                        self.current_shape_idx = 0;
                    }
                    Ok(())
                },
                Err(e) => {
                    self.show_error("Import Error", &format!("Failed to parse shapes: {}", e));
                    Err(io::Error::new(io::ErrorKind::InvalidData, e))
                }
            }
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            // For WebAssembly, file reading is handled through the file input element
            // The actual reading happens in handle_file_content
            // Here we just return success
            Ok(())
        }
    }
    
    // Convert from data_structures::Shape to ast::Shape
    pub fn convert_to_ast_shape(&self, app_shape: &AppShape) -> crate::ast::Shape {
        let mut scales = Vec::new();
        let scale = crate::ast::Scale {
            verts: app_shape.vertices.iter().map(|v| crate::ast::Vertex { x: v.x, y: v.y }).collect(),
            ports: app_shape.ports.iter().map(|p| crate::ast::Port { 
                edge: p.edge, 
                position: p.position, 
                port_type: Some(crate::ast::PortType::from_str(&p.port_type.to_string()))
            }).collect(),
        };
        
        scales.push(scale);
        
        crate::ast::Shape {
            id: app_shape.id,
            name: Some(app_shape.name.clone()),
            scales,
            launcher_radial: if app_shape.launcher_radial { Some(true) } else { None },
            mirror_of: None,
            group: None,
            features: None,
            fill_color: None,
            fill_color1: None,
            line_color: None,
            durability: None,
            density: None,
            grow_rate: None,
            shroud: None,
            cannon: None,
            thruster: None,
        }
    }
    
    // Convert from ast::Shape to data_structures::Shape
    pub fn convert_from_ast_shape(&self, ast_shape: &crate::ast::Shape) -> AppShape {
        let mut app_shape = AppShape::new(ast_shape.id);
        
        if let Some(name) = &ast_shape.name {
            app_shape.name = name.clone();
        }
        
        // Use the first scale for vertices and ports
        if !ast_shape.scales.is_empty() {
            let scale = &ast_shape.scales[0];
            
            // Convert vertices
            for vert in &scale.verts {
                app_shape.vertices.push(Vertex {
                    x: vert.x,
                    y: vert.y,
                });
            }
            
            // Convert ports
            for port in &scale.ports {
                app_shape.ports.push(Port {
                    edge: port.edge,
                    position: port.position,
                    port_type: if let Some(pt) = &port.port_type {
                        match pt {
                            crate::ast::PortType::Default => PortType::Default,
                            crate::ast::PortType::ThrusterIn => PortType::ThrusterIn,
                            crate::ast::PortType::ThrusterOut => PortType::ThrusterOut,
                            crate::ast::PortType::Missile => PortType::Missile,
                            crate::ast::PortType::Launcher => PortType::Launcher,
                            crate::ast::PortType::WeaponIn => PortType::WeaponIn,
                            crate::ast::PortType::WeaponOut => PortType::WeaponOut,
                            crate::ast::PortType::Root => PortType::Root,
                            crate::ast::PortType::None => PortType::None,
                        }
                    } else {
                        PortType::Default
                    },
                });
            }
        }
        
        // Set launcher_radial property
        if let Some(launcher_radial) = ast_shape.launcher_radial {
            app_shape.launcher_radial = launcher_radial;
        }
        
        app_shape
    }
    
    // Parse shapes from Lua string using the ast module
    fn parse_lua_shapes(&self, content: &str) -> Result<Vec<AppShape>, io::Error> {
        match parse_shapes_content(content) {
            Ok(shapes_file) => {
                let mut app_shapes = Vec::new();
                println!("Successfully parsed {} shapes", shapes_file.shapes.len());
                
                for ast_shape in &shapes_file.shapes {
                    let app_shape = self.convert_from_ast_shape(ast_shape);
                    println!("Converted shape ID: {}, Name: {}, Vertices: {}, Ports: {}, launcher_radial: {}", 
                             app_shape.id, 
                             app_shape.name, 
                             app_shape.vertices.len(), 
                             app_shape.ports.len(),
                             app_shape.launcher_radial);
                    app_shapes.push(app_shape);
                }
                
                Ok(app_shapes)
            }
            Err(e) => {
                println!("Failed to parse shapes: {}", e);
                // Convert parse error to IO error with the message
                Err(io::Error::new(io::ErrorKind::InvalidData, e))
            }
        }
    }
    
    // Original legacy parser
    fn parse_lua_shapes_legacy(&self, content: &str) -> Result<Vec<AppShape>, io::Error> {
        let mut shapes = Vec::new();
        let mut lines = content.lines().enumerate();
        
        // Skip initial line that might be a comment or opening brace
        while let Some((_, line)) = lines.next() {
            let trimmed = line.trim();
            if trimmed == "{" || trimmed.starts_with("{") {
                break;
            }
        }
        
        let mut current_shape: Option<AppShape> = None;
        let mut in_verts = false;
        let mut in_ports = false;
        
        while let Some((_line_num, line)) = lines.next() {
            let trimmed = line.trim();
            
            // Check for shape start
            if trimmed.starts_with("{") && !in_verts && !in_ports {
                // Extract shape ID and possibly name
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    // Try to parse ID
                    if let Ok(id) = parts[0].trim_matches(|c| c == '{' || c == '}' || c == ',').parse::<usize>() {
                        let mut name = format!("Shape_{}", id);
                        
                        // Check for name comment
                        if trimmed.contains("--") {
                            if let Some(name_part) = trimmed.split("--").nth(1) {
                                name = name_part.trim().to_string();
                            }
                        }
                        
                        current_shape = Some(AppShape {
                            id,
                            name,
                            vertices: Vec::new(),
                            ports: Vec::new(),
                            selected_vertex: None,
                            selected_port: None,
                            launcher_radial: false,
                        });
                    }
                }
            }
            
            // Check for verts section
            if trimmed.contains("verts=") {
                in_verts = true;
                continue;
            }
            
            // Check for ports section
            if trimmed.contains("ports=") {
                in_verts = false;
                in_ports = true;
                continue;
            }
            
            // Parse vertex
            if in_verts && trimmed.starts_with("{") && current_shape.is_some() {
                let parts: Vec<&str> = trimmed
                    .trim_matches(|c| c == '{' || c == '}' || c == ',')
                    .split(',')
                    .collect();
                
                if parts.len() >= 2 {
                    if let (Ok(x), Ok(y)) = (
                        f32::from_str(parts[0].trim()),
                        f32::from_str(parts[1].trim())
                    ) {
                        if let Some(shape) = &mut current_shape {
                            shape.vertices.push(Vertex { x, y });
                        }
                    }
                }
            }
            
            // Parse port
            if in_ports && trimmed.starts_with("{") && current_shape.is_some() {
                let parts: Vec<&str> = trimmed
                    .trim_matches(|c| c == '{' || c == '}' || c == ',')
                    .split(',')
                    .collect();
                
                if parts.len() >= 2 {
                    if let (Ok(edge), Ok(position)) = (
                        usize::from_str(parts[0].trim()),
                        f32::from_str(parts[1].trim())
                    ) {
                        if let Some(shape) = &mut current_shape {
                            let mut port_type = PortType::Default;
                            
                            // Check if port type is specified
                            if parts.len() >= 3 {
                                let type_str = parts[2].trim();
                                port_type = match type_str {
                                    "THRUSTER_IN" => PortType::ThrusterIn,
                                    "THRUSTER_OUT" => PortType::ThrusterOut,
                                    "MISSILE" => PortType::Missile,
                                    "LAUNCHER" => PortType::Launcher,
                                    "WEAPON_IN" => PortType::WeaponIn,
                                    "WEAPON_OUT" => PortType::WeaponOut,
                                    "ROOT" => PortType::Root,
                                    "NONE" => PortType::None,
                                    _ => PortType::Default,
                                };
                            }
                            
                            shape.ports.push(Port {
                                edge,
                                position,
                                port_type,
                            });
                        }
                    }
                }
            }
            
            // End of shape
            if trimmed == "}" && !in_verts && !in_ports {
                if let Some(shape) = current_shape.take() {
                    shapes.push(shape);
                }
            }
            
            // End of verts or ports section
            if trimmed == "}," || trimmed == "}" {
                in_verts = false;
                in_ports = false;
            }
        }
        
        Ok(shapes)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn select_import_file(&mut self) -> bool {
        if let Some(path) = FileDialog::new()
            .add_filter("Lua files", &["lua"])
            .set_directory("/")
            .pick_file() {
                if let Some(path_str) = path.to_str() {
                    self.import_path = path_str.to_string();
                    return true;
                }
            }
        false
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    pub fn select_export_file(&mut self) -> bool {
        if let Some(path) = FileDialog::new()
            .add_filter("Lua files", &["lua"])
            .set_directory("/")
            .save_file() {
                if let Some(path_str) = path.to_str() {
                    self.export_path = path_str.to_string();
                    return true;
                }
            }
        false
    }
    
    #[cfg(target_arch = "wasm32")]
    pub fn has_file_input_element() -> bool {
        use wasm_bindgen::JsCast;
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        document.get_element_by_id("file-input").is_some()
    }
    
    #[cfg(target_arch = "wasm32")]
    pub fn create_file_input_element() {
        use wasm_bindgen::JsCast;
        use wasm_bindgen::closure::Closure;
        
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        
        // Create file input element if it doesn't exist
        if !Self::has_file_input_element() {
            let input = document.create_element("input").unwrap();
            let input_element = input.dyn_into::<web_sys::HtmlInputElement>().unwrap();
            
            input_element.set_id("file-input");
            input_element.set_type("file");
            input_element.style().set_property("display", "none").unwrap();
            input_element.set_accept(".lua");
            
            let body = document.body().unwrap();
            body.append_child(&input_element).unwrap();
        }
    }
    
    #[cfg(target_arch = "wasm32")]
    pub fn select_import_file(&mut self) -> bool {
        use wasm_bindgen::JsCast;
        use wasm_bindgen::closure::Closure;
        
        Self::create_file_input_element();
        
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        
        if let Some(input_element) = document.get_element_by_id("file-input") {
            let input = input_element.dyn_into::<web_sys::HtmlInputElement>().unwrap();
            input.click();
            
            // File selection is handled asynchronously through JavaScript events
            // We'll read the file in the onchange event handler defined in the UI layer
            return true;
        }
        
        false
    }
    
    #[cfg(target_arch = "wasm32")]
    pub fn select_export_file(&mut self) -> bool {
        // In WebAssembly, we can't directly save files, so we'll just use the text input
        // The export function will handle saving differently for WASM
        true
    }
    
    // Handle file content from Web input
    #[cfg(target_arch = "wasm32")]
    pub fn handle_file_content(&mut self, content: String, filename: String) {
        self.import_path = filename;
        
        match self.parse_lua_shapes(&content) {
            Ok(shapes) => {
                if !shapes.is_empty() {
                    self.save_state();
                    self.shapes = shapes;
                    self.current_shape_idx = 0;
                    self.status_message = Some(format!("{} {}", crate::translations::t("shapes_imported"), self.import_path));
                    self.status_time = 3.0;
                }
            },
            Err(e) => {
                self.show_error("Import Error", &format!("Failed to parse shapes: {}", e));
            }
        }
    }
}

// Implementing eframe::App trait
impl eframe::App for ShapeEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply dark theme
        configure_visuals(ctx);
        
        // Process keyboard shortcuts
        self.process_keyboard_shortcuts(ctx);
        
        // Render UI components based on the active tab
        render_nav_bar(ctx, self);
        
        if self.active_tab == 0 {
            // Shapes tab
            render_top_panel(ctx, self);
            render_side_panel(ctx, self);
            render_central_panel(ctx, self);
        } else if self.active_tab == 1 {
            // Settings tab
            render_settings_panel(ctx, self);
        }
        
        // Show error dialog if needed
        if self.show_error_dialog {
            if show_error_dialog(
                ctx, 
                self.error_title.clone(), 
                self.error_message.clone(), 
                &mut self.show_error_dialog
            ) {
                // Dialog was closed
                self.show_error_dialog = false;
            }
        }
        
        // Request continuous redraw while status message is showing
        if self.status_time > 0.0 {
            ctx.request_repaint();
        }
    }
}

// Add the process_keyboard_shortcuts method to the main ShapeEditor impl
impl ShapeEditor {
    // Process keyboard shortcuts for undo/redo and other functions
    fn process_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        // Undo/Redo shortcuts
        if ctx.input().key_pressed(egui::Key::Z) && ctx.input().modifiers.ctrl {
            if ctx.input().modifiers.shift {
                self.redo();
            } else {
                self.undo();
            }
        } else if ctx.input().key_pressed(egui::Key::Y) && ctx.input().modifiers.ctrl {
            self.redo();
        }
    }
} 