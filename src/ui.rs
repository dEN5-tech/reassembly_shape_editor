// UI components module
use eframe::egui;
use egui::*;

use crate::data_structures::{Vertex, Port, PortType};
use crate::shape_editor::ShapeEditor;
use crate::translations::t;
use crate::{ visual::*};
use crate::geometry::{area_for_poly, Vec2};

// Render game-style navigation bar
pub fn render_nav_bar(ctx: &egui::Context, app: &mut ShapeEditor) {
    egui::TopBottomPanel::top("nav_bar")
        .frame(Frame::none().fill(Color32::from_rgba_unmultiplied(20, 20, 20, 220)))
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                // Use the game-style tab buttons for main navigation
                if game_tab_button(ui, &t("shapes"), app.active_tab == 0).clicked() {
                    app.active_tab = 0;
                }
                if game_tab_button(ui, &t("settings"), app.active_tab == 1).clicked() {
                    app.active_tab = 1;
                }
            });
        });
    
    // Show the section title
    egui::TopBottomPanel::top("section_title")
        .frame(Frame::none().fill(Color32::TRANSPARENT))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(5.0);
                let title = match app.active_tab {
                    1 => t("settings"),
                    _ => t("current_construction")
                };
                ui.heading(&title);
                ui.add_space(5.0);
            });
        });
}

// Render top panel with controls for zoom, grid, and export
pub fn render_top_panel(ctx: &egui::Context, app: &mut ShapeEditor) {
    let top_panel_frame = ui_panel_frame();
    
    egui::TopBottomPanel::top("top_panel")
        .frame(top_panel_frame)
        .show(ctx, |ui| {
        // First row: basic controls
        ui.horizontal(|ui| {
            if styled_button(ui, &t("new_shape")).clicked() {
                app.add_shape();
            }
            
            ui.add_space(20.0);
            
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(&t("zoom"));
                    ui.add(egui::Slider::new(&mut app.zoom, 0.1..=5.0).fixed_decimals(2));
                });
            });
            
            ui.add_space(20.0);
            
            ui.group(|ui| {
                ui.vertical(|ui| {
                    styled_checkbox(ui, &mut app.show_grid, &t("show_grid"));
                    styled_checkbox(ui, &mut app.snap_to_grid, &t("snap_to_grid"));
                });
            });
            
            ui.add_space(20.0);
            
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(&t("grid_size"));
                    ui.add(egui::Slider::new(&mut app.grid_size, 1.0..=50.0).step_by(1.0));
                });
            });
        });
        
        // Second row: export and import controls
        ui.horizontal(|ui| {
            // Export controls
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(&t("export_file"));
                    ui.add(egui::TextEdit::singleline(&mut app.export_path).desired_width(200.0));
                    
                    // Add file selection button
                    if styled_button(ui, &t("browse")).clicked() {
                        app.select_export_file();
                    }
                    
                    if styled_button(ui, &t("export")).clicked() {
                        if let Err(e) = app.export_shapes() {
                            app.show_error(&t("error_export"), &e.to_string());
                        } else {
                            app.status_message = Some(format!("{} {}", t("shapes_exported"), app.export_path));
                            app.status_time = 3.0;
                        }
                    }
                });
            });
            
            ui.add_space(10.0);
            
            if styled_button(ui, &t("export_lua")).clicked() {
                // Temporarily save the original path
                let original_path = app.export_path.clone();
                
                // Set path to shapes.lua for the export
                app.export_path = "shapes.lua".to_string();
                
                // Export shapes
                if let Err(e) = app.export_shapes() {
                    app.show_error(&t("error_export"), &e.to_string());
                } else {
                    app.status_message = Some(format!("{} shapes.lua", t("shapes_exported")));
                    app.status_time = 3.0;
                }
                
                // Restore the original path
                app.export_path = original_path;
            }
            
            ui.add_space(20.0);
            
            // Import controls
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(&t("import_file"));
                    ui.add(egui::TextEdit::singleline(&mut app.import_path).desired_width(200.0));
                    
                    // Add file selection button
                    if styled_button(ui, &t("browse")).clicked() {
                        app.select_import_file();
                    }
                    
                    if styled_button(ui, &t("import")).clicked() {
                        if let Err(_e) = app.import_shapes() {
                            // Error handling is now done in import_shapes()
                            // Show errors via the dialog
                        } else {
                            app.status_message = Some(format!("{} {}", t("shapes_imported"), app.import_path));
                            app.status_time = 3.0;
                        }
                    }
                });
            });
            
            ui.add_space(10.0);
            
            if styled_button(ui, &t("import_lua")).clicked() {
                // Temporarily save the original path
                let original_path = app.import_path.clone();
                
                // Set path to shapes.lua for the import
                app.import_path = "shapes.lua".to_string();
                
                // Import shapes
                if let Err(_e) = app.import_shapes() {
                    // Error handling is now done in import_shapes()
                    // Show errors via the dialog
                } else {
                    app.status_message = Some(format!("{} shapes.lua", t("shapes_imported")));
                    app.status_time = 3.0;
                }
                
                // Restore the original path
                app.import_path = original_path;
            }
        });
    });
}

// Render side panel with shape, vertex, and port controls
pub fn render_side_panel(ctx: &egui::Context, app: &mut ShapeEditor) {
    let side_panel_frame = ui_panel_frame();
    
    // Collection of edits to apply after the UI is rendered
    enum ShapeEdit {
        UpdateName(String),
        UpdateVertex(usize, Vertex),
        RemoveVertex(usize),
        AddPort(Port),
        UpdatePort(usize, Port),
        RemovePort(usize),
        SelectVertex(Option<usize>),
        SelectPort(Option<usize>),
        ToggleLauncherRadial(bool),
    }
    
    let mut edits = Vec::new();
    
    egui::SidePanel::left("side_panel")
        .frame(side_panel_frame)
        .default_width(220.0)
        .show(ctx, |ui| {
        // Apply heading style
        ui.heading(&t("shapes"));
        
        ui.push_id("shapes_list", |ui| {
            // Frame for the shapes list
            egui::Frame::none()
                .fill(Color32::from_rgba_unmultiplied(16, 16, 16, 230))
                .inner_margin(6.0)
                .rounding(4.0)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (i, shape) in app.shapes.iter().enumerate() {
                            let selected = i == app.current_shape_idx;
                            // Custom styling for selected labels
                            let selectable = ui.selectable_label(selected, &shape.name);
                            if selectable.clicked() {
                                app.current_shape_idx = i;
                            }
                        }
                    });
                });
        });
        
        ui.add_space(10.0);
        
        if !app.shapes.is_empty() {
            let current_shape_idx = app.current_shape_idx;
            let shape = &app.shapes[current_shape_idx];
            
            ui.heading(&t("shape_properties"));
            
            // Shape properties frame
            egui::Frame::none()
                .fill(Color32::from_rgba_unmultiplied(16, 16, 16, 230))
                .inner_margin(6.0)
                .rounding(4.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.strong("ID:");
                        ui.label(shape.id.to_string());
                    });
                    
                    ui.add_space(4.0);
                    
                    ui.horizontal(|ui| {
                        ui.strong(&format!("{}:", t("shape_name")));
                        let mut name = shape.name.clone();
                        if ui.add(egui::TextEdit::singleline(&mut name).desired_width(140.0)).changed() {
                            edits.push(ShapeEdit::UpdateName(name));
                        }
                    });
                    
                    ui.add_space(4.0);
                    
                    ui.horizontal(|ui| {
                        ui.strong(&format!("{}:", t("radial_launcher")));
                        let mut launcher_radial = shape.launcher_radial;
                        if ui.checkbox(&mut launcher_radial, "").changed() {
                            edits.push(ShapeEdit::ToggleLauncherRadial(launcher_radial));
                        }
                    });
                });
            
            ui.add_space(10.0);
            
            ui.heading(&t("vertices"));
            ui.push_id("vertices_list", |ui| {
                // Custom frame for vertex list
                egui::Frame::none()
                    .fill(Color32::from_rgba_unmultiplied(16, 16, 16, 230))
                    .inner_margin(6.0)
                    .rounding(4.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(150.0)
                            .show(ui, |ui| {
                                let vertices = &shape.vertices;
                                
                                for (i, vertex) in vertices.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        let selected = shape.selected_vertex == Some(i);
                                        if ui.selectable_label(selected, format!("V{}", i)).clicked() {
                                            edits.push(ShapeEdit::SelectVertex(Some(i)));
                                        }
                                        
                                        ui.add_space(5.0);
                                        
                                        ui.label("X:");
                                        let mut x = vertex.x;
                                        let changed_x = ui.add(egui::DragValue::new(&mut x).speed(0.1).fixed_decimals(1)).changed();
                                        
                                        ui.add_space(5.0);
                                        
                                        ui.label("Y:");
                                        let mut y = vertex.y;
                                        let changed_y = ui.add(egui::DragValue::new(&mut y).speed(0.1).fixed_decimals(1)).changed();
                                        
                                        if changed_x || changed_y {
                                            edits.push(ShapeEdit::UpdateVertex(i, Vertex { x, y }));
                                        }
                                        
                                        ui.with_layout(egui::Layout::right_to_left(), |ui| {
                                            // Delete button styling
                                            if styled_button(ui, "X").clicked() {
                                                edits.push(ShapeEdit::RemoveVertex(i));
                                            }
                                        });
                                    });
                                }
                            });
                    });
            });
            
            ui.add_space(10.0);
            
            ui.heading(&t("ports"));
            ui.push_id("ports_list", |ui| {
                // Custom frame for ports list
                egui::Frame::none()
                    .fill(Color32::from_rgba_unmultiplied(16, 16, 16, 230))
                    .inner_margin(6.0)
                    .rounding(4.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(180.0)
                            .show(ui, |ui| {
                                let ports = &shape.ports;
                                
                                for (i, port) in ports.iter().enumerate() {
                                    ui.push_id(i, |ui| {
                                        // Port frame for each port
                                        egui::Frame::none()
                                            .inner_margin(4.0)
                                            .fill(if shape.selected_port == Some(i) {
                                                Color32::from_rgba_unmultiplied(40, 40, 50, 230)
                                            } else {
                                                Color32::TRANSPARENT
                                            })
                                            .show(ui, |ui| {
                                                let mut port_updated = false;
                                                let mut new_port = port.clone();
                                                
                                                ui.horizontal(|ui| {
                                                    let selected = shape.selected_port == Some(i);
                                                    if ui.selectable_label(selected, format!("P{}", i)).clicked() {
                                                        edits.push(ShapeEdit::SelectPort(Some(i)));
                                                    }
                                                    
                                                    ui.add_space(5.0);
                                                    
                                                    ui.label(&format!("{}:", t("edge")));
                                                    if ui.add(egui::DragValue::new(&mut new_port.edge).speed(0.1)).changed() {
                                                        port_updated = true;
                                                    }
                                                    
                                                    ui.add_space(5.0);
                                                    
                                                    ui.label(&format!("{}:", t("position")));
                                                    if ui.add(egui::DragValue::new(&mut new_port.position).speed(0.01)
                                                        .clamp_range(0.0..=1.0).fixed_decimals(2)).changed() {
                                                        port_updated = true;
                                                    }
                                                });
                                                
                                                ui.horizontal(|ui| {
                                                    ui.label(&format!("{}:", t("type")));
                                                    ui.add_space(5.0);
                                                    
                                                    if egui::ComboBox::from_id_source(format!("port_type_{}", i))
                                                        .selected_text(new_port.port_type.to_string())
                                                        .width(120.0)
                                                        .show_ui(ui, |ui| {
                                                            ui.selectable_value(&mut new_port.port_type, PortType::Default, "DEFAULT");
                                                            ui.selectable_value(&mut new_port.port_type, PortType::ThrusterIn, "THRUSTER_IN");
                                                            ui.selectable_value(&mut new_port.port_type, PortType::ThrusterOut, "THRUSTER_OUT");
                                                            ui.selectable_value(&mut new_port.port_type, PortType::Missile, "MISSILE");
                                                            ui.selectable_value(&mut new_port.port_type, PortType::Launcher, "LAUNCHER");
                                                            ui.selectable_value(&mut new_port.port_type, PortType::WeaponIn, "WEAPON_IN");
                                                            ui.selectable_value(&mut new_port.port_type, PortType::WeaponOut, "WEAPON_OUT");
                                                            ui.selectable_value(&mut new_port.port_type, PortType::Root, "ROOT");
                                                            ui.selectable_value(&mut new_port.port_type, PortType::None, "NONE");
                                                        })
                                                        .response
                                                        .changed()
                                                    {
                                                        port_updated = true;
                                                    }
                                                    
                                                    ui.with_layout(egui::Layout::right_to_left(), |ui| {
                                                        // Delete button styling
                                                        if styled_button(ui, "X").clicked() {
                                                            edits.push(ShapeEdit::RemovePort(i));
                                                        }
                                                    });
                                                });
                                                
                                                if port_updated {
                                                    edits.push(ShapeEdit::UpdatePort(i, new_port.clone()));
                                                }
                                            });
                                        
                                        ui.add_space(2.0);
                                    });
                                }
                                
                                ui.add_space(5.0);
                                
                                // Style add button using our custom button
                                if styled_button(ui, &t("add_port")).clicked() && !shape.vertices.is_empty() {
                                    edits.push(ShapeEdit::AddPort(Port {
                                        edge: 0,
                                        position: 0.5,
                                        port_type: PortType::Default,
                                    }));
                                }
                            });
                    });
            });
        }
    });
    
    // Apply all collected edits
    if !edits.is_empty() {
        let current_shape_idx = app.current_shape_idx;
        
        for edit in edits {
            match edit {
                ShapeEdit::UpdateName(name) => {
                    app.save_state();
                    app.shapes[current_shape_idx].name = name;
                },
                ShapeEdit::UpdateVertex(idx, vertex) => {
                    app.save_state();
                    if idx < app.shapes[current_shape_idx].vertices.len() {
                        app.shapes[current_shape_idx].vertices[idx] = vertex;
                    }
                },
                ShapeEdit::RemoveVertex(idx) => {
                    app.remove_vertex(current_shape_idx, idx);
                },
                ShapeEdit::AddPort(port) => {
                    app.add_port(current_shape_idx, port);
                },
                ShapeEdit::UpdatePort(idx, port) => {
                    app.save_state();
                    if idx < app.shapes[current_shape_idx].ports.len() {
                        app.shapes[current_shape_idx].ports[idx] = port;
                    }
                },
                ShapeEdit::RemovePort(idx) => {
                    app.remove_port(current_shape_idx, idx);
                },
                ShapeEdit::SelectVertex(idx) => {
                    app.shapes[current_shape_idx].selected_vertex = idx;
                    app.shapes[current_shape_idx].selected_port = None;
                },
                ShapeEdit::SelectPort(idx) => {
                    app.shapes[current_shape_idx].selected_port = idx;
                    app.shapes[current_shape_idx].selected_vertex = None;
                },
                ShapeEdit::ToggleLauncherRadial(launcher_radial) => {
                    app.save_state();
                    app.shapes[current_shape_idx].launcher_radial = launcher_radial;
                },
            }
        }
    }
}

// Render central panel with the canvas for shape editing
pub fn render_central_panel(ctx: &egui::Context, app: &mut ShapeEditor) {
    // Central panel with custom styling - dark background
    let central_panel_frame = Frame::none()
        .fill(Color32::from_rgb(0, 0, 0)) // Pure black background
        .inner_margin(0.0);
    
    egui::CentralPanel::default()
        .frame(central_panel_frame)
        .show(ctx, |ui| {
        let response = ui.allocate_response(ui.available_size(), egui::Sense::click_and_drag());
        let rect = response.rect;
        
        // Handle mouse wheel for zooming
        if let Some(pos) = ui.ctx().pointer_interact_pos() {
            let scroll_delta = ui.ctx().input().scroll_delta.y;
            if scroll_delta != 0.0 && rect.contains(pos) {
                app.zoom_at(pos, rect, scroll_delta * 0.01);
            }
        }
        
        // Check if middle mouse is pressed or released
        let middle_pressed = ui.ctx().input().pointer.button_down(egui::PointerButton::Middle);
        let was_middle_down = app.middle_drag_ongoing;
        let middle_released = !middle_pressed && was_middle_down;
        
        // Handle middle mouse button for panning
        if middle_pressed && !was_middle_down {
            app.middle_drag_ongoing = true;
            app.zoom_center = ui.ctx().pointer_interact_pos().unwrap_or(rect.center());
        } else if middle_released {
            app.middle_drag_ongoing = false;
        }
        
        // Perform the panning with middle mouse
        if app.middle_drag_ongoing {
            if let Some(current_pos) = ui.ctx().pointer_interact_pos() {
                let delta = current_pos - app.zoom_center;
                app.pan.x += delta.x / app.zoom;
                app.pan.y += delta.y / app.zoom;
                app.zoom_center = current_pos;
            }
        }
        
        // Обработка перетаскивания холста правой кнопкой мыши (legacy support)
        if response.dragged_by(egui::PointerButton::Secondary) {
            let delta = response.drag_delta();
            app.pan.x += delta.x / app.zoom;
            app.pan.y += delta.y / app.zoom;
        }
        
        if !app.shapes.is_empty() {
            let shape_idx = app.current_shape_idx;
            
            // Отрисовка сетки
            if app.show_grid {
                render_grid(&ui.painter(), app, rect);
            }
            
            // Рисуем форму, если есть хотя бы две вершины
            if app.shapes[shape_idx].vertices.len() > 1 {
                render_shape(&ui.painter(), ctx, app, shape_idx, rect);
            }
            
            // Отрисовка вершин
            render_vertices(&ui.painter(), app, shape_idx, rect);
            
            // Отображение информации о форме
            let info_text = format!(
                "Форма: {} (ID: {})\nВершин: {}\nПортов: {}", 
                app.shapes[shape_idx].name,
                app.shapes[shape_idx].id,
                app.shapes[shape_idx].vertices.len(),
                app.shapes[shape_idx].ports.len()
            );
            
            ui.painter().text(
                rect.min + vec2(10.0, 10.0),
                Align2::LEFT_TOP,
                info_text,
                FontId::proportional(14.0),
                Color32::WHITE,
            );
            
            // Display keybind help in the bottom right
            let keybind_text = "Ctrl+Z: Отменить | Ctrl+Y: Повторить | Alt+Клик: Добавить порт | Ctrl+Клик: Добавить вершину на грани | Esc: Отменить выделение | Delete: Удалить выделенное";
            ui.painter().text(
                rect.right_bottom() - vec2(10.0, 10.0),
                Align2::RIGHT_BOTTOM,
                keybind_text,
                FontId::proportional(12.0),
                Color32::from_rgba_unmultiplied(200, 200, 200, 180),
            );
            
            // Обработка клика на холсте для добавления или выбора вершины
            handle_canvas_clicks(app, response, rect, shape_idx);
        }
    });
}

// Helper function to render the grid
fn render_grid(painter: &Painter, app: &ShapeEditor, rect: Rect) {
    let grid_color = Color32::from_rgba_premultiplied(100, 100, 100, 100);
    
    let min_x = ((rect.min.x - rect.center().x) / app.zoom - app.pan.x) / app.grid_size;
    let max_x = ((rect.max.x - rect.center().x) / app.zoom - app.pan.x) / app.grid_size;
    let min_y = ((rect.min.y - rect.center().y) / app.zoom - app.pan.y) / app.grid_size;
    let max_y = ((rect.max.y - rect.center().y) / app.zoom - app.pan.y) / app.grid_size;
    
    let min_x = min_x.floor() as i32;
    let max_x = max_x.ceil() as i32;
    let min_y = min_y.floor() as i32;
    let max_y = max_y.ceil() as i32;
    
    // Draw vertical grid lines
    for x in min_x..=max_x {
        let x_pos = x as f32 * app.grid_size;
        let start = app.shape_to_screen_coords(&Vertex { x: x_pos, y: min_y as f32 * app.grid_size }, rect);
        let end = app.shape_to_screen_coords(&Vertex { x: x_pos, y: max_y as f32 * app.grid_size }, rect);
        painter.line_segment([start, end], Stroke::new(1.0, grid_color));
    }
    
    // Draw horizontal grid lines
    for y in min_y..=max_y {
        let y_pos = y as f32 * app.grid_size;
        let start = app.shape_to_screen_coords(&Vertex { x: min_x as f32 * app.grid_size, y: y_pos }, rect);
        let end = app.shape_to_screen_coords(&Vertex { x: max_x as f32 * app.grid_size, y: y_pos }, rect);
        painter.line_segment([start, end], Stroke::new(1.0, grid_color));
    }
    
    // Draw coordinate axes
    let origin = app.shape_to_screen_coords(&Vertex { x: 0.0, y: 0.0 }, rect);
    let x_axis = app.shape_to_screen_coords(&Vertex { x: max_x as f32 * app.grid_size, y: 0.0 }, rect);
    let y_axis = app.shape_to_screen_coords(&Vertex { x: 0.0, y: max_y as f32 * app.grid_size }, rect);
    
    painter.line_segment([origin, x_axis], Stroke::new(2.0, Color32::RED));
    painter.line_segment([origin, y_axis], Stroke::new(2.0, Color32::GREEN));
}

// Helper function to render the shape
fn render_shape(painter: &Painter, ctx: &egui::Context, app: &ShapeEditor, shape_idx: usize, rect: Rect) {
    // Convert vertices to screen coordinates
    let mut points = Vec::new();
    for vertex in &app.shapes[shape_idx].vertices {
        points.push(app.shape_to_screen_coords(vertex, rect));
    }
    
    let fill_color = Color32::from_rgba_premultiplied(30, 40, 80, 160);
    let stroke = Stroke::new(1.0, Color32::WHITE);

    // Draw the shape as triangles from center
    if points.len() > 2 {
        // Calculate center point
        let center = points.iter().fold(Pos2::new(0.0, 0.0), |acc, pos| {
            Pos2::new(acc.x + pos.x, acc.y + pos.y)
        });
        let center = Pos2::new(center.x / points.len() as f32, center.y / points.len() as f32);
        
        // Draw triangles from center to each edge
        for i in 0..points.len() {
            let p1 = points[i];
            let p2 = points[(i + 1) % points.len()];
            
            let triangle = vec![center, p1, p2];
            
            // Fill triangle with transparent stroke
            painter.add(egui::Shape::convex_polygon(
                triangle,
                fill_color,
                Stroke::new(0.0, Color32::TRANSPARENT),
            ));
        }
        
        // Draw shape outline
        for i in 0..points.len() {
            let start = points[i];
            let end = points[(i + 1) % points.len()];
            painter.line_segment([start, end], stroke);
        }
        
        // Calculate and display shape area
        let vertices: Vec<Vec2> = app.shapes[shape_idx].vertices.iter()
            .map(|v| Vec2::new(v.x, v.y))
            .collect();
            
        if vertices.len() >= 3 {
            let area = area_for_poly(&vertices);
            let area_text = format!("Area: {:.1}", area);
            
            painter.text(
                points[0] + vec2(-10.0, -20.0),
                Align2::RIGHT_CENTER,
                area_text,
                FontId::monospace(12.0),
                Color32::LIGHT_BLUE,
            );
        }
    }

    // Draw shape outline with ports
    for i in 0..app.shapes[shape_idx].vertices.len() {
        let start = points[i];
        let end = points[(i + 1) % points.len()];
        
        // Draw edge
        painter.line_segment([start, end], Stroke::new(2.0, Color32::WHITE));
        
        // Draw ports on this edge
        for (port_idx, port) in app.shapes[shape_idx].ports.iter().enumerate() {
            if port.edge == i {
                let t = port.position;
                let port_pos = Pos2 {
                    x: start.x + (end.x - start.x) * t,
                    y: start.y + (end.y - start.y) * t,
                };
                
                // Check if this port is selected
                let is_selected = app.shapes[shape_idx].selected_port == Some(port_idx);
                
                // Get port color based on type
                let port_color = match port.port_type {
                    PortType::Default => Color32::YELLOW,
                    PortType::ThrusterIn | PortType::ThrusterOut => Color32::BLUE,
                    PortType::Missile | PortType::Launcher => Color32::RED,
                    PortType::WeaponIn | PortType::WeaponOut => Color32::LIGHT_BLUE,
                    PortType::Root => Color32::GREEN,
                    PortType::None => Color32::GRAY,
                };
                
                // Draw port with glow animation
                let time = ctx.input().time as f32;
                let pulse = (time * 2.0).sin() * 0.5 + 0.5;
                let size = 5.0 + pulse * 2.0;
                
                // Port glow - make it brighter if selected
                let glow_color = if is_selected {
                    port_color.linear_multiply(0.5)
                } else {
                    port_color.linear_multiply(0.3)
                };
                
                painter.circle_filled(port_pos, size + 2.0, glow_color);
                painter.circle_filled(port_pos, size, port_color);
                
                // Port label
                let port_text = match port.port_type {
                    PortType::Default => "",
                    PortType::ThrusterIn => "TI",
                    PortType::ThrusterOut => "TO",
                    PortType::Missile => "M",
                    PortType::Launcher => "L",
                    PortType::WeaponIn => "WI",
                    PortType::WeaponOut => "WO",
                    PortType::Root => "R",
                    PortType::None => "N",
                };
                
                if port_text != "" {
                    painter.text(
                        port_pos + vec2(8.0, 0.0),
                        Align2::LEFT_CENTER,
                        port_text,
                        FontId::monospace(10.0),
                        port_color,
                    );
                }
            }
        }
    }
    
    // Draw shape folding visualization
    if app.shapes[shape_idx].vertices.len() > 2 {
        let first_vertex = app.shape_to_screen_coords(&app.shapes[shape_idx].vertices[0], rect);
        for i in 2..app.shapes[shape_idx].vertices.len() {
            let vertex = app.shape_to_screen_coords(&app.shapes[shape_idx].vertices[i], rect);
            painter.line_segment(
                [first_vertex, vertex], 
                Stroke::new(1.0, Color32::from_rgba_premultiplied(150, 150, 150, 100))
            );
        }
    }
}

// Helper function to render all vertices
fn render_vertices(painter: &Painter, app: &ShapeEditor, shape_idx: usize, rect: Rect) {
    for (i, v) in app.shapes[shape_idx].vertices.iter().enumerate() {
        let pos = app.shape_to_screen_coords(v, rect);
        let is_selected = app.shapes[shape_idx].selected_vertex == Some(i);
        let is_first = i == 0;
        
        // Special highlighting for first vertex
        let (fill_color, stroke_color, size) = if is_first {
            if is_selected {
                (Color32::YELLOW, Color32::WHITE, 7.0)
            } else {
                (Color32::GOLD, Color32::WHITE, 6.0)
            }
        } else if is_selected {
            (Color32::LIGHT_BLUE, Color32::WHITE, 6.0)
        } else {
            (Color32::DARK_BLUE, Color32::WHITE, 5.0)
        };
        
        painter.circle_filled(pos, size, fill_color);
        painter.circle_stroke(pos, size, Stroke::new(1.0, stroke_color));
        
        // Display vertex number
        painter.text(
            pos + vec2(10.0, 0.0),
            Align2::LEFT_CENTER,
            format!("{}", i),
            FontId::monospace(14.0),
            if is_selected { Color32::YELLOW } else { Color32::WHITE },
        );
    }
}

// Handle canvas clicks for adding/selecting vertices and ports
fn handle_canvas_clicks(app: &mut ShapeEditor, response: Response, rect: Rect, shape_idx: usize) {
    let input = response.ctx.input();
    
    // Handle Escape key to clear selection
    if input.key_pressed(egui::Key::Escape) {
        app.shapes[shape_idx].selected_vertex = None;
        app.shapes[shape_idx].selected_port = None;
    }
    
    // Handle Delete key to remove selected elements
    if input.key_pressed(egui::Key::Delete) || input.key_pressed(egui::Key::Backspace) {
        if let Some(vertex_idx) = app.shapes[shape_idx].selected_vertex {
            app.remove_vertex(shape_idx, vertex_idx);
        } else if let Some(port_idx) = app.shapes[shape_idx].selected_port {
            app.remove_port(shape_idx, port_idx);
        }
    }
    
    // Add or select vertex/port on click
    if response.clicked() {
        if let Some(mouse_pos) = response.interact_pointer_pos() {
            // Check if Alt is pressed for port creation mode
            let alt_pressed = input.modifiers.alt;
            
            // First check for clicking on ports
            let mut clicked_port_idx = None;
            
            for (i, port) in app.shapes[shape_idx].ports.iter().enumerate() {
                let edge_idx = port.edge;
                if edge_idx < app.shapes[shape_idx].vertices.len() {
                    let v1 = &app.shapes[shape_idx].vertices[edge_idx];
                    let v2 = &app.shapes[shape_idx].vertices[(edge_idx + 1) % app.shapes[shape_idx].vertices.len()];
                    
                    let start = app.shape_to_screen_coords(v1, rect);
                    let end = app.shape_to_screen_coords(v2, rect);
                    
                    let t = port.position;
                    let port_pos = Pos2 {
                        x: start.x + (end.x - start.x) * t,
                        y: start.y + (end.y - start.y) * t,
                    };
                    
                    if (mouse_pos - port_pos).length() < 10.0 {
                        clicked_port_idx = Some(i);
                        break;
                    }
                }
            }
            
            // Then check for clicking on vertices
            let mut clicked_vertex_idx = None;
            
            if clicked_port_idx.is_none() {
                for (i, v) in app.shapes[shape_idx].vertices.iter().enumerate() {
                    let pos = app.shape_to_screen_coords(v, rect);
                    if (mouse_pos - pos).length() < 10.0 {
                        clicked_vertex_idx = Some(i);
                        break;
                    }
                }
            }
            
            // Check for clicking on an edge to add a port (when Alt is pressed or no vertex is clicked)
            let mut clicked_edge = None;
            let mut edge_position = 0.5; // Default position on edge
            
            if (clicked_vertex_idx.is_none() && clicked_port_idx.is_none()) || alt_pressed {
                for i in 0..app.shapes[shape_idx].vertices.len() {
                    let v1 = &app.shapes[shape_idx].vertices[i];
                    let v2 = &app.shapes[shape_idx].vertices[(i + 1) % app.shapes[shape_idx].vertices.len()];
                    
                    let start = app.shape_to_screen_coords(v1, rect);
                    let end = app.shape_to_screen_coords(v2, rect);
                    
                    // Check distance from point to line segment
                    let closest = closest_point_on_line_segment(mouse_pos, start, end);
                    let distance = (mouse_pos - closest).length();
                    
                    if distance < 10.0 {
                        clicked_edge = Some(i);
                        
                        // Calculate normalized position along the edge
                        let total_length = (end - start).length();
                        if total_length > 0.0 {
                            edge_position = (closest - start).length() / total_length;
                        }
                        break;
                    }
                }
            }
            
            // Handle selections and creations
            if let Some(port_idx) = clicked_port_idx {
                // Select port
                app.shapes[shape_idx].selected_port = Some(port_idx);
                app.shapes[shape_idx].selected_vertex = None;
            } else if let Some(vertex_idx) = clicked_vertex_idx {
                // Select vertex
                app.shapes[shape_idx].selected_vertex = Some(vertex_idx);
                app.shapes[shape_idx].selected_port = None;
            } else if alt_pressed && clicked_edge.is_some() {
                // Add a new port on edge when Alt is pressed
                let edge_idx = clicked_edge.unwrap();
                app.add_port(shape_idx, Port {
                    edge: edge_idx,
                    position: edge_position,
                    port_type: PortType::Default,
                });
                // Select the new port
                app.shapes[shape_idx].selected_port = Some(app.shapes[shape_idx].ports.len() - 1);
                app.shapes[shape_idx].selected_vertex = None;
            } else if clicked_edge.is_some() && app.shapes[shape_idx].vertices.len() > 2 {
                // Clicking on an edge can select it or add a vertex in the middle
                if input.modifiers.ctrl {
                    // Ctrl+Click on edge to add a vertex in the middle
                    let edge_idx = clicked_edge.unwrap();
                    let v1 = &app.shapes[shape_idx].vertices[edge_idx];
                    let v2 = &app.shapes[shape_idx].vertices[(edge_idx + 1) % app.shapes[shape_idx].vertices.len()];
                    
                    // Create new vertex at the midpoint (or clicked position)
                    let new_vertex = Vertex {
                        x: v1.x + (v2.x - v1.x) * edge_position,
                        y: v1.y + (v2.y - v1.y) * edge_position,
                    };
                    
                    // Insert new vertex after edge_idx
                    app.save_state();
                    app.shapes[shape_idx].vertices.insert(edge_idx + 1, new_vertex);
                    app.shapes[shape_idx].selected_vertex = Some(edge_idx + 1);
                    app.shapes[shape_idx].selected_port = None;
                    
                    // Adjust ports on this edge
                    for port in &mut app.shapes[shape_idx].ports {
                        if port.edge == edge_idx {
                            if port.position > edge_position {
                                // Port is after the new vertex, move it to new edge
                                port.edge = edge_idx + 1;
                                // Adjust position to new edge scale
                                port.position = (port.position - edge_position) / (1.0 - edge_position);
                            } else {
                                // Port is before new vertex, keep it on same edge but rescale
                                port.position = port.position / edge_position;
                            }
                        } else if port.edge > edge_idx {
                            // Increment edge index for all ports after this edge
                            port.edge += 1;
                        }
                    }
                } else {
                    // Just clear selection when clicking empty space
                    app.shapes[shape_idx].selected_vertex = None;
                    app.shapes[shape_idx].selected_port = None;
                }
            } else {
                // Add new vertex when clicking on empty space
                let shape_coords = app.screen_to_shape_coords(mouse_pos, rect);
                app.add_or_update_vertex(shape_idx, shape_coords, None);
            }
        }
    }
    
    // Handle drag for moving vertices
    let drag_ongoing = response.dragged_by(egui::PointerButton::Primary);
    let drag_started = response.drag_started();
    
    if let Some(idx) = app.shapes[shape_idx].selected_vertex {
        if drag_ongoing {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                let shape_coords = app.screen_to_shape_coords(mouse_pos, rect);
                
                if drag_started {
                    // Save state only when drag starts
                    app.save_state();
                }
                
                // Update vertex position
                app.shapes[shape_idx].vertices[idx] = shape_coords;
            }
        }
    } else if let Some(idx) = app.shapes[shape_idx].selected_port {
        if drag_ongoing {
            if let Some(mouse_pos) = response.interact_pointer_pos() {
                if drag_started {
                    app.save_state();
                }
                
                // Get the edge for this port
                let port = &app.shapes[shape_idx].ports[idx];
                let edge_idx = port.edge;
                
                if edge_idx < app.shapes[shape_idx].vertices.len() {
                    let v1 = &app.shapes[shape_idx].vertices[edge_idx];
                    let v2 = &app.shapes[shape_idx].vertices[(edge_idx + 1) % app.shapes[shape_idx].vertices.len()];
                    
                    let start = app.shape_to_screen_coords(v1, rect);
                    let end = app.shape_to_screen_coords(v2, rect);
                    
                    // Calculate new position on the edge
                    let closest = closest_point_on_line_segment(mouse_pos, start, end);
                    let total_length = (end - start).length();
                    if total_length > 0.0 {
                        let new_position = (closest - start).length() / total_length;
                        app.shapes[shape_idx].ports[idx].position = new_position.clamp(0.0, 1.0);
                    }
                }
            }
        }
    }
}

// Helper function to find the closest point on a line segment
fn closest_point_on_line_segment(p: Pos2, a: Pos2, b: Pos2) -> Pos2 {
    let ap = Vec2::new(p.x - a.x, p.y - a.y);
    let ab = Vec2::new(b.x - a.x, b.y - a.y);
    
    let ab_squared = ab.x * ab.x + ab.y * ab.y;
    if ab_squared < 0.0001 {
        return a; // Points A and B are the same
    }
    
    let t = (ap.x * ab.x + ap.y * ab.y) / ab_squared;
    if t < 0.0 {
        return a; // Beyond point A on the line
    } else if t > 1.0 {
        return b; // Beyond point B on the line
    }
    
    Pos2::new(
        a.x + ab.x * t,
        a.y + ab.y * t
    )
}

// Render settings panel with language selection
pub fn render_settings_panel(ctx: &egui::Context, app: &mut ShapeEditor) {
    if app.active_tab != 1 {
        return;
    }
    
    egui::CentralPanel::default()
        .show(ctx, |ui| {
            ui.add_space(20.0);
            
            // Create a frame for settings
            let settings_frame = Frame::none()
                .fill(Color32::from_rgba_unmultiplied(32, 32, 32, 217))
                .inner_margin(16.0)
                .outer_margin(8.0)
                .rounding(Rounding::same(4.0));
            
            settings_frame.show(ui, |ui| {
                // Create a centered layout with max width
                let max_width = 400.0;
                let available_width = ui.available_width();
                let indent = ((available_width - max_width) / 2.0).max(0.0);
                
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.add_space(indent);
                    ui.vertical(|ui| {
                        ui.set_max_width(max_width);
                        
                        // Language settings
                        ui.heading(&t("language"));
                        ui.add_space(10.0);
                        
                        let languages = crate::translations::available_languages();
                        let mut current_lang = crate::translations::get_current_language();
                        
                        egui::ComboBox::from_id_source("language_selector")
                            .selected_text(match current_lang.as_str() {
                                "en" => t("language_en"),
                                "ru" => t("language_ru"),
                                _ => current_lang.clone()
                            })
                            .width(200.0)
                            .show_ui(ui, |ui| {
                                for lang in languages {
                                    let display_name = match lang.as_str() {
                                        "en" => t("language_en"),
                                        "ru" => t("language_ru"),
                                        _ => lang.clone()
                                    };
                                    
                                    if ui.selectable_value(&mut current_lang, lang.clone(), display_name).clicked() {
                                        crate::translations::set_language(&lang);
                                    }
                                }
                            });
                        
                        ui.add_space(20.0);
                        
                        // Add Apply button
                        if action_button(ui, &t("apply")).clicked() {
                            // Show confirmation message
                            app.status_message = Some(t("settings_saved"));
                            app.status_time = 3.0; // Show for 3 seconds
                        }
                    });
                });
                
                ui.add_space(10.0);
            });
            
            // Show status message if exists
            if let Some(msg) = &app.status_message {
                if app.status_time > 0.0 {
                    // Create a toast-like notification
                    let job = egui::text::LayoutJob::simple_singleline(
                        msg.clone(), 
                        TextStyle::Body.resolve(ui.style()),
                        Color32::WHITE
                    );
                    let galley = ui.painter().layout(
                        job.text.clone(),
                        job.sections.first().map(|s| s.format.font_id.clone()).unwrap_or_else(|| TextStyle::Body.resolve(ui.style())),
                        Color32::WHITE,
                        f32::INFINITY
                    );
                    let padding = 10.0;
                    let width = galley.rect.width() + padding * 2.0;
                    let height = galley.rect.height() + padding * 2.0;
                    
                    let screen_width = ui.available_width();
                    let toast_rect = Rect::from_center_size(
                        Pos2::new(screen_width / 2.0, 60.0),
                        egui::Vec2::new(width, height)
                    );
                    
                    ui.painter().rect_filled(
                        toast_rect,
                        Rounding::same(4.0),
                        Color32::from_rgba_unmultiplied(40, 40, 40, 230)
                    );
                    
                    ui.painter().rect_stroke(
                        toast_rect,
                        Rounding::same(4.0),
                        Stroke::new(1.0, Color32::from_rgb(100, 200, 100))
                    );
                    
                    ui.painter().text(
                        toast_rect.center(),
                        Align2::CENTER_CENTER,
                        msg,
                        TextStyle::Body.resolve(ui.style()),
                        Color32::from_rgb(100, 200, 100)
                    );
                    
                    // Update the timer when drawing the frame
                    let ctx = ui.ctx();
                    app.status_time -= ctx.input().predicted_dt;
                    ctx.request_repaint(); // Ensure we keep rendering
                }
            }
        });
} 