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
use crate::lua_parser::parse_shapes_content;

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
        }
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
        let mut content = "{\n".to_string();
        
        for shape in &self.shapes {
            content.push_str(&shape.to_lua());
            content.push_str(",\n");
        }
        
        content.push_str("}\n");
        
        // Replace forward slashes with the OS-specific path separator
        let path = Path::new(&self.export_path);
        fs::write(path, content)
    }
    
    // Import shapes from Lua file
    pub fn import_shapes(&mut self) -> Result<(), io::Error> {
        self.save_state();
        
        let content = fs::read_to_string(&self.import_path)?;
        let shapes = self.parse_lua_shapes(&content)?;
        
        if !shapes.is_empty() {
            self.shapes = shapes;
            self.current_shape_idx = 0;
        }
        
        Ok(())
    }
    
    // Convert from data_structures::Shape to ast::Shape
    pub fn convert_to_ast_shape(&self, app_shape: &AppShape) -> crate::ast::Shape {
        let mut scales = Vec::new();
        
        // Create a single scale containing all vertices and ports
        let mut scale = crate::ast::Scale {
            verts: Vec::new(),
            ports: Vec::new(),
        };
        
        // Convert vertices
        for vertex in &app_shape.vertices {
            scale.verts.push(crate::ast::Vertex {
                x: vertex.x,
                y: vertex.y,
            });
        }
        
        // Convert ports
        for port in &app_shape.ports {
            scale.ports.push(crate::ast::Port {
                edge: port.edge,
                position: port.position,
                port_type: match port.port_type {
                    PortType::Default => Some(crate::ast::PortType::Default),
                    PortType::ThrusterIn => Some(crate::ast::PortType::ThrusterIn),
                    PortType::ThrusterOut => Some(crate::ast::PortType::ThrusterOut),
                    PortType::Missile => Some(crate::ast::PortType::Missile),
                    PortType::Launcher => Some(crate::ast::PortType::Launcher),
                    PortType::WeaponIn => Some(crate::ast::PortType::WeaponIn),
                    PortType::WeaponOut => Some(crate::ast::PortType::WeaponOut),
                    PortType::Root => Some(crate::ast::PortType::Root),
                    PortType::None => Some(crate::ast::PortType::None),
                },
            });
        }
        
        scales.push(scale);
        
        crate::ast::Shape {
            id: app_shape.id,
            name: Some(app_shape.name.clone()),
            scales,
            launcher_radial: if app_shape.launcher_radial { Some(true) } else { None },
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
                for ast_shape in &shapes_file.shapes {
                    app_shapes.push(self.convert_from_ast_shape(ast_shape));
                }
                Ok(app_shapes)
            }
            Err(_) => {
                // Fallback to the old parser if the new one fails
                self.parse_lua_shapes_legacy(content)
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
}

// Implementing eframe::App trait
impl eframe::App for ShapeEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Set the app visuals to match our style
        configure_visuals(ctx);
        
        // Render main navigation bar
        render_nav_bar(ctx, self);
        
        // Render the top panel with controls
        if self.active_tab != 1 { // Don't show in settings tab
            render_top_panel(ctx, self);
        }
        
        // Process keyboard shortcuts
        self.process_keyboard_shortcuts(ctx);
        
        // Render the appropriate panel based on active tab
        match self.active_tab {
            0 => {
                // In shapes tab, render side panel and central panel for editing
                render_side_panel(ctx, self);
                render_central_panel(ctx, self);
            },
            1 => render_settings_panel(ctx, self),
            _ => {
                // Default case - should not happen with only 2 tabs
                render_side_panel(ctx, self);
                render_central_panel(ctx, self);
            }
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