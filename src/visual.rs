use eframe::egui;
use egui::*;
use crate::data_structures::PortType;
use crate::translations::t;

/// Draws a port marker at the specified position with the given port type
pub fn draw_port(painter: &Painter, pos: Pos2, port_type: &PortType, selected: bool) {
    let radius = 4.0;
    let color = port_color(port_type);
    
    // Draw the port circle
    if selected {
        // Draw selected port with highlight
        painter.circle_stroke(pos, radius + 2.0, Stroke::new(1.5, Color32::from_rgb(255, 255, 0)));
        painter.circle_filled(pos, radius, color);
    } else {
        painter.circle_filled(pos, radius, color);
        painter.circle_stroke(pos, radius, Stroke::new(1.0, Color32::from_rgb(140, 140, 140)));
    }
}

/// Returns the appropriate color for a port based on its type
fn port_color(port_type: &PortType) -> Color32 {
    match port_type {
        PortType::Default => Color32::from_rgb(200, 200, 200),
        PortType::ThrusterIn => Color32::from_rgb(0, 150, 255),
        PortType::ThrusterOut => Color32::from_rgb(0, 200, 255),
        PortType::Missile => Color32::from_rgb(255, 100, 0),
        PortType::Launcher => Color32::from_rgb(255, 150, 0),
        PortType::WeaponIn => Color32::from_rgb(255, 50, 50),
        PortType::WeaponOut => Color32::from_rgb(255, 0, 0),
        PortType::Root => Color32::from_rgb(0, 255, 0),
        PortType::None => Color32::from_rgb(100, 100, 100),
    }
}

/// Creates a styled button that matches the CSS design
pub fn styled_button(ui: &mut Ui, text: &str) -> Response {
    let button_padding = vec2(12.0, 6.0);
    let border_radius = 4.0;
    let button_stroke = Stroke::new(1.0, Color32::from_rgb(140, 140, 140));
    
    // Normal state
    let normal_fill = Color32::from_rgba_unmultiplied(32, 32, 32, 217);
    let normal_text = Color32::from_rgb(140, 140, 140);
    
    // Create button visuals - without rounding since it's not supported in this version
    let button = Button::new(RichText::new(text).color(normal_text))
        .fill(normal_fill)
        .stroke(button_stroke);
    
    // Set padding and rounding by wrapping in a Frame
    let frame = Frame::none()
        .inner_margin(button_padding)
        .fill(Color32::TRANSPARENT)
        .rounding(Rounding::same(border_radius));
    
    let response = frame.show(ui, |ui| {
        ui.add(button)
    }).inner;
    
    // Handle hover/active states similar to CSS classes
    if response.hovered() {
        ui.ctx().request_repaint(); // For smooth transitions
        
        // Apply hover highlighting - brighter fill and text
        let hover_fill = Color32::from_rgba_unmultiplied(50, 50, 50, 217);
        let hover_text = Color32::from_rgb(238, 238, 238);
        let hover_stroke = Stroke::new(1.0, Color32::from_rgb(200, 200, 200));
        
        // Draw the hover state manually
        let rect = response.rect;
        ui.painter().rect(
            rect, 
            Rounding::same(border_radius), 
            hover_fill, 
            hover_stroke
        );
        
        // Replace the text with hovered style
        ui.painter().text(
            rect.center(), 
            Align2::CENTER_CENTER, 
            text, 
            TextStyle::Button.resolve(ui.style()), 
            hover_text
        );
    }
    
    // Active/pressed state
    if response.is_pointer_button_down_on() {
        ui.ctx().request_repaint();
        
        // Apply active/pressed styling - darker fill and white text
        let active_fill = Color32::from_rgba_unmultiplied(25, 25, 25, 217);
        let active_text = Color32::from_rgb(255, 255, 255);
        let active_stroke = Stroke::new(1.0, Color32::from_rgb(255, 255, 255));
        
        // Draw the pressed state with slight scale transform effect
        let rect = response.rect;
        let scale = 0.96; // Scale down slightly when pressed
        let scaled_rect = Rect::from_center_size(
            rect.center(),
            rect.size() * scale
        );
        
        ui.painter().rect(
            scaled_rect, 
            Rounding::same(border_radius), 
            active_fill, 
            active_stroke
        );
        
        // Replace the text with active style
        ui.painter().text(
            scaled_rect.center(), 
            Align2::CENTER_CENTER, 
            text, 
            TextStyle::Button.resolve(ui.style()), 
            active_text
        );
    }
    
    response
}

/// Creates a styled checkbox that matches the modern UI style
pub fn styled_checkbox(ui: &mut Ui, checked: &mut bool, text: &str) -> Response {
    let checkbox_size = 16.0;
    let spacing = 8.0;
    let border_radius = 3.0;
    
    // Create a layout for the checkbox
    let total_width = ui.available_width();
    
    let (rect, mut response) = ui.allocate_exact_size(
        Vec2::new(total_width, checkbox_size + 4.0), 
        Sense::click()
    );
    
    // Handle interaction
    if response.clicked() {
        *checked = !*checked;
        response.mark_changed();
    }
    
    // Draw checkbox box
    let checkbox_rect = Rect::from_min_size(rect.min, Vec2::splat(checkbox_size));
    
    // Determine colors based on state
    let (fill_color, stroke) = if *checked {
        if response.hovered() {
            // Checked + Hovered
            (Color32::from_rgb(30, 130, 255), Stroke::new(1.0, Color32::from_rgb(238, 238, 238)))
        } else {
            // Checked
            (Color32::from_rgb(0, 150, 255), Stroke::new(1.0, Color32::from_rgb(140, 140, 140)))
        }
    } else {
        if response.hovered() {
            // Unchecked + Hovered
            (Color32::from_rgba_unmultiplied(45, 45, 45, 217), Stroke::new(1.0, Color32::from_rgb(238, 238, 238)))
        } else {
            // Unchecked
            (Color32::from_rgba_unmultiplied(32, 32, 32, 217), Stroke::new(1.0, Color32::from_rgb(140, 140, 140)))
        }
    };
    
    // Draw the checkbox with rounded corners
    ui.painter().rect(checkbox_rect, Rounding::same(border_radius), fill_color, stroke);
    
    // Draw check mark if checked
    if *checked {
        let check_color = Color32::from_rgb(255, 255, 255); // White check mark
        let points = vec![
            checkbox_rect.min + vec2(3.0, 8.0),
            checkbox_rect.min + vec2(7.0, 12.0),
            checkbox_rect.min + vec2(14.0, 4.0),
        ];
        ui.painter().line_segment(
            [points[0], points[1]],
            Stroke::new(2.0, check_color),
        );
        ui.painter().line_segment(
            [points[1], points[2]],
            Stroke::new(2.0, check_color),
        );
    }
    
    // Draw text with appropriate color
    let text_pos = checkbox_rect.right_center() + vec2(spacing, 0.0);
    let text_color = if response.hovered() {
        Color32::from_rgb(238, 238, 238)
    } else {
        Color32::from_rgb(200, 200, 200)
    };
    
    ui.painter().text(
        text_pos,
        Align2::LEFT_CENTER,
        text,
        TextStyle::Body.resolve(ui.style()),
        text_color,
    );
    
    response
}

/// Configures visuals to match the CSS style
pub fn configure_visuals(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();
    
    // Configure dark theme similar to the CSS
    visuals.extreme_bg_color = Color32::from_rgb(0, 0, 0); // #000000 background
    visuals.code_bg_color = Color32::from_rgba_unmultiplied(32, 32, 32, 217); // rgba(32,32,32,0.85)
    visuals.faint_bg_color = Color32::from_rgba_unmultiplied(100, 100, 100, 50); // rgba(100,100,100,0.2)
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgba_unmultiplied(32, 32, 32, 217); // rgba(32,32,32,0.85)
    visuals.widgets.inactive.bg_fill = Color32::from_rgba_unmultiplied(32, 32, 32, 217);
    visuals.widgets.hovered.bg_fill = Color32::from_rgba_unmultiplied(50, 50, 50, 217);
    visuals.widgets.active.bg_fill = Color32::from_rgba_unmultiplied(70, 70, 70, 217);
    
    // Text color
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(255, 255, 255)); // #FFFFFF
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(140, 140, 140)); // #8C8C8C
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::from_rgb(238, 238, 238)); // #EEEEEE
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::from_rgb(255, 255, 255)); // #FFFFFF
    
    // Border colors
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(140, 140, 140)); // rgba(140,140,140,1.0)
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(140, 140, 140));
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(238, 238, 238)); // #EEEEEE
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(255, 255, 255)); // #FFFFFF
    
    // Apply rounded corners to widgets
    visuals.widgets.noninteractive.rounding = Rounding::same(4.0);
    visuals.widgets.inactive.rounding = Rounding::same(4.0);
    visuals.widgets.hovered.rounding = Rounding::same(4.0);
    visuals.widgets.active.rounding = Rounding::same(4.0);
    
    // Selected item highlight color
    visuals.selection.bg_fill = Color32::from_rgb(255, 255, 0); // #FFFF00
    visuals.selection.stroke = Stroke::new(1.0, Color32::from_rgb(255, 255, 0));
    
    // Set window rounding to match CSS
    visuals.window_rounding = Rounding::same(4.0);
    
    ctx.set_visuals(visuals);
    
    // Configure fonts
    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(20.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(12.0, FontFamily::Monospace)),
        (TextStyle::Button, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(10.0, FontFamily::Proportional)),
    ].into();
    ctx.set_style(style);
}

/// Creates a custom frame style for UI elements
pub fn custom_frame_style() -> egui::Frame {
    egui::Frame {
        fill: Color32::from_rgba_unmultiplied(32, 32, 32, 217), // rgba(32,32,32,0.85)
        stroke: Stroke::new(1.0, Color32::from_rgb(140, 140, 140)), // border: 1px solid rgba(140,140,140,1.0)
        inner_margin: egui::style::Margin::same(4.0), // padding: 4px
        outer_margin: egui::style::Margin::same(3.0), // margin: 3px
        rounding: egui::Rounding::same(4.0), // rounded corners like in CSS
        shadow: eframe::epaint::Shadow::default(),
    }
}

/// Creates a panel styled similarly to the Reassembly UI div.ui elements
pub fn ui_panel_frame() -> egui::Frame {
    egui::Frame {
        fill: Color32::from_rgba_unmultiplied(100, 100, 100, 50), // rgba(100,100,100,0.2)
        stroke: Stroke::new(1.0, Color32::from_rgb(140, 140, 140)), // border: 1px solid rgba(140,140,140,1.0)
        inner_margin: egui::style::Margin::same(4.0), // padding: 4px
        outer_margin: egui::style::Margin::same(4.0), // margin: 4px
        rounding: egui::Rounding::same(4.0), // rounded corners like in CSS
        shadow: eframe::epaint::Shadow::default(),
    }
}

/// Creates a component box styled similarly to the Reassembly UI div.component elements
pub fn component_frame() -> egui::Frame {
    egui::Frame {
        fill: Color32::from_rgba_unmultiplied(32, 32, 32, 217), // rgba(32,32,32,0.85) 
        stroke: Stroke::new(1.0, Color32::from_rgb(140, 140, 140)),
        inner_margin: egui::style::Margin::same(0.0), // No padding
        outer_margin: egui::style::Margin::same(3.0), // margin: 0 3px 0 3px
        rounding: egui::Rounding::same(4.0), // rounded corners like in CSS
        shadow: eframe::epaint::Shadow::default(),
    }
}

/// Creates a popup frame styled like modern popups
pub fn popup_frame() -> egui::Frame {
    egui::Frame {
        fill: Color32::from_rgba_unmultiplied(32, 32, 32, 245), // Nearly opaque background
        stroke: Stroke::new(1.0, Color32::from_rgb(140, 140, 140)),
        inner_margin: egui::style::Margin::same(8.0), // More padding for popups
        outer_margin: egui::style::Margin::same(4.0),
        rounding: egui::Rounding::same(4.0), // Rounded corners
        shadow: eframe::epaint::Shadow::default(), // Use default shadow
    }
}

/// Create a focused/highlighted button style
pub fn action_button(ui: &mut Ui, text: &str) -> Response {
    // Action button with a bright blue background
    let button_padding = vec2(12.0, 6.0);
    let border_radius = 4.0;
    
    // Normal state
    let normal_fill = Color32::from_rgb(31, 105, 255); // Action blue color
    let normal_text = Color32::from_rgb(255, 255, 255); // White text
    let normal_stroke = Stroke::new(1.0, Color32::from_rgb(31, 105, 255));
    
    // Create button visuals - without rounding since it's not supported in this version
    let button = Button::new(RichText::new(text).color(normal_text))
        .fill(normal_fill)
        .stroke(normal_stroke);
    
    // Set padding and rounding by wrapping in a Frame
    let frame = Frame::none()
        .inner_margin(button_padding)
        .fill(Color32::TRANSPARENT)
        .rounding(Rounding::same(border_radius));
    
    let response = frame.show(ui, |ui| {
        ui.add(button)
    }).inner;
    
    // Handle hover state
    if response.hovered() {
        ui.ctx().request_repaint();
        
        // Apply hover highlighting - lighter blue
        let hover_fill = Color32::from_rgb(71, 133, 255);
        let hover_stroke = Stroke::new(1.0, Color32::from_rgb(71, 133, 255));
        
        // Draw the hover state
        let rect = response.rect;
        ui.painter().rect(
            rect, 
            Rounding::same(border_radius), 
            hover_fill, 
            hover_stroke
        );
        
        // Text remains white
        ui.painter().text(
            rect.center(), 
            Align2::CENTER_CENTER, 
            text, 
            TextStyle::Button.resolve(ui.style()), 
            normal_text
        );
    }
    
    // Active/pressed state
    if response.is_pointer_button_down_on() {
        ui.ctx().request_repaint();
        
        // Darker blue when pressed
        let active_fill = Color32::from_rgb(0, 90, 200);
        let active_stroke = Stroke::new(1.0, Color32::from_rgb(255, 255, 255));
        
        // Draw with scale effect
        let rect = response.rect;
        let scale = 0.96;
        let scaled_rect = Rect::from_center_size(
            rect.center(),
            rect.size() * scale
        );
        
        ui.painter().rect(
            scaled_rect, 
            Rounding::same(border_radius), 
            active_fill, 
            active_stroke
        );
        
        // Text remains white
        ui.painter().text(
            scaled_rect.center(), 
            Align2::CENTER_CENTER, 
            text, 
            TextStyle::Button.resolve(ui.style()), 
            normal_text
        );
    }
    
    response
}

/// Creates a tab-like button styled after the game UI tabs
pub fn game_tab_button(ui: &mut Ui, text: &str, selected: bool) -> Response {
    let button_padding = vec2(16.0, 8.0);
    let border_radius = 4.0;
    
    // Colors based on state
    let (fill_color, text_color, stroke) = if selected {
        // Selected tab
        (
            Color32::from_rgba_unmultiplied(64, 64, 64, 230), 
            Color32::from_rgb(255, 255, 255),
            Stroke::new(1.0, Color32::from_rgb(140, 140, 140))
        )
    } else {
        // Unselected tab
        (
            Color32::from_rgba_unmultiplied(32, 32, 32, 180),
            Color32::from_rgb(180, 180, 180),
            Stroke::new(1.0, Color32::from_rgb(100, 100, 100))
        )
    };
    
    // Create button
    let button = Button::new(RichText::new(text).color(text_color))
        .fill(fill_color)
        .stroke(stroke);
    
    // Frame for padding and rounding
    let frame = Frame::none()
        .inner_margin(button_padding)
        .fill(Color32::TRANSPARENT)
        .rounding(Rounding::same(border_radius));
    
    let response = frame.show(ui, |ui| {
        ui.add(button)
    }).inner;
    
    // Handle hover state
    if response.hovered() && !selected {
        ui.ctx().request_repaint();
        
        // Hover effect
        let hover_fill = Color32::from_rgba_unmultiplied(48, 48, 48, 200);
        let hover_text = Color32::from_rgb(220, 220, 220);
        
        let rect = response.rect;
        ui.painter().rect(
            rect, 
            Rounding::same(border_radius), 
            hover_fill, 
            Stroke::new(1.0, Color32::from_rgb(160, 160, 160))
        );
        
        ui.painter().text(
            rect.center(), 
            Align2::CENTER_CENTER, 
            text, 
            TextStyle::Button.resolve(ui.style()), 
            hover_text
        );
    }
    
    response
}

/// Creates a ship slot element similar to the game's ship selection UI
pub fn ship_slot_frame(selected: bool) -> egui::Frame {
    let (fill, stroke) = if selected {
        (
            Color32::from_rgba_unmultiplied(64, 64, 64, 230),
            Stroke::new(1.0, Color32::from_rgb(180, 180, 180))
        )
    } else {
        (
            Color32::from_rgba_unmultiplied(32, 32, 32, 200),
            Stroke::new(1.0, Color32::from_rgb(140, 140, 140))
        )
    };
    
    egui::Frame {
        fill,
        stroke,
        inner_margin: egui::style::Margin::same(4.0),
        outer_margin: egui::style::Margin::same(4.0),
        rounding: egui::Rounding::same(4.0),
        shadow: eframe::epaint::Shadow::default(),
    }
}

/// Creates a list item for ship selection similar to the game's UI
pub fn ship_list_item(ui: &mut Ui, name: &str, p_value: i32, selected: bool) -> Response {
    let frame = ship_slot_frame(selected);
    
    let response = frame.show(ui, |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                // Make the name text bold if selected
                if selected {
                    ui.strong(name);
                } else {
                    ui.label(name);
                }
                
                ui.with_layout(Layout::right_to_left(), |ui| {
                    // Display the P value with a colored background
                    let p_text = format!("{}P", p_value);
                    let (text, bg_color) = if p_value > 1000 {
                        (RichText::new(&p_text).color(Color32::WHITE), Color32::from_rgb(180, 0, 0))
                    } else if p_value > 500 {
                        (RichText::new(&p_text).color(Color32::WHITE), Color32::from_rgb(200, 100, 0))
                    } else if p_value > 200 {
                        (RichText::new(&p_text).color(Color32::BLACK), Color32::from_rgb(200, 200, 0))
                    } else {
                        (RichText::new(&p_text).color(Color32::BLACK), Color32::from_rgb(100, 200, 100))
                    };
                    
                    ui.label(text.background_color(bg_color));
                });
            });
        }).response
    }).inner;
    
    // Handle hover effects
    if response.hovered() && !selected {
        let hover_fill = Color32::from_rgba_unmultiplied(50, 50, 50, 220);
        ui.painter().rect(
            response.rect, 
            Rounding::same(4.0), 
            hover_fill, 
            Stroke::new(1.0, Color32::from_rgb(180, 180, 180))
        );
    }
    
    response
}

/// Creates a header with indicator values like the game's resource display
pub fn resource_indicator(ui: &mut egui::Ui, label: &str, current: i32, max: i32, color: Color32) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).color(Color32::from_rgb(200, 200, 200)));
        ui.with_layout(Layout::right_to_left(), |ui| {
            let text = format!("{}/{}", current, max);
            ui.label(RichText::new(text).color(color));
        });
    });
}

/// Creates a status bar similar to the game UI's resource bars
pub fn status_bar(ui: &mut egui::Ui, current: f32, max: f32, color: Color32) -> Response {
    let height = 12.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
    
    // Draw background
    ui.painter().rect_filled(
        rect,
        Rounding::same(2.0),
        Color32::from_rgba_unmultiplied(20, 20, 20, 180)
    );
    
    // Calculate fill width
    let fill_ratio = (current / max).clamp(0.0, 1.0);
    let fill_width = rect.width() * fill_ratio;
    
    // Draw fill
    if fill_width > 0.0 {
        let fill_rect = Rect::from_min_size(
            rect.min, 
            Vec2::new(fill_width, rect.height())
        );
        
        ui.painter().rect_filled(
            fill_rect,
            Rounding::same(2.0),
            color
        );
    }
    
    // Draw border
    ui.painter().rect_stroke(
        rect,
        Rounding::same(2.0),
        Stroke::new(1.0, Color32::from_rgb(80, 80, 80))
    );
    
    response
}

/// Renders a construction slot as seen in the game UI
pub fn construction_slot(ui: &mut Ui, slot_number: i32, empty: bool) -> Response {
    let slot_size = Vec2::new(120.0, 80.0);
    let (rect, response) = ui.allocate_exact_size(slot_size, Sense::click());
    
    // Draw slot background
    let bg_color = if empty {
        Color32::from_rgba_unmultiplied(30, 30, 30, 150)
    } else {
        Color32::from_rgba_unmultiplied(50, 50, 50, 180)
    };
    
    ui.painter().rect_filled(
        rect,
        Rounding::same(4.0),
        bg_color
    );
    
    // Draw border
    let border_color = if response.hovered() {
        Color32::from_rgb(200, 200, 200)
    } else {
        Color32::from_rgb(140, 140, 140)
    };
    
    ui.painter().rect_stroke(
        rect,
        Rounding::same(4.0),
        Stroke::new(1.0, border_color)
    );
    
    // Draw slot number in top-left corner
    ui.painter().text(
        rect.left_top() + vec2(8.0, 8.0),
        Align2::LEFT_TOP,
        slot_number.to_string(),
        TextStyle::Monospace.resolve(ui.style()),
        Color32::from_rgb(200, 200, 200)
    );
    
    // If empty, add placeholder text
    if empty {
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            &t("empty"),
            TextStyle::Body.resolve(ui.style()),
            Color32::from_rgb(150, 150, 150)
        );
    }
    
    response
}

/// Creates a separator line like those used in the game UI
pub fn ui_separator(ui: &mut Ui) {
    ui.add_space(4.0);
    let rect = ui.available_rect_before_wrap();
    let separator_height = 1.0;
    let separator_rect = Rect::from_min_size(
        rect.min, 
        Vec2::new(rect.width(), separator_height)
    );
    
    ui.painter().rect_filled(
        separator_rect,
        Rounding::same(0.0),
        Color32::from_rgb(100, 100, 100)
    );
    ui.add_space(4.0);
}

/// Creates a tooltip similar to the game's UI
pub fn show_tooltip(ui: &egui::Ui, response: &Response, text: &str) {
    if response.hovered() {
        egui::containers::show_tooltip_at(
            ui.ctx(),
            egui::Id::new("tooltip"),
            Some(response.rect.center()),
            |ui| {
                ui.label(text);
            }
        );
    }
}

/// Creates an error dialog frame
pub fn error_dialog_frame() -> egui::Frame {
    egui::Frame {
        fill: Color32::from_rgba_unmultiplied(40, 20, 20, 245), // Dark red background
        stroke: Stroke::new(1.0, Color32::from_rgb(200, 100, 100)), // Red border
        inner_margin: egui::style::Margin::same(12.0), // More padding for error dialogs
        outer_margin: egui::style::Margin::same(4.0),
        rounding: egui::Rounding::same(4.0), // Rounded corners
        shadow: eframe::epaint::Shadow::default(), // Use default shadow
    }
}

/// Shows a modal error dialog
/// 
/// # Arguments
/// * `ctx` - The egui context
/// * `title` - Dialog title (displayed in the window header)
/// * `message` - Message content as RichText or convertible to RichText
/// * `open` - Mutable reference to a boolean controlling dialog visibility
/// 
/// # Returns
/// `true` if the OK button was clicked, `false` otherwise
pub fn show_error_dialog<T: Into<egui::RichText>>(
    ctx: &egui::Context, 
    title: impl Into<egui::WidgetText>, 
    message: T, 
    open: &mut bool
) -> bool {
    let mut result = false;
    
    if *open {
        // Center the dialog
        let screen_rect = ctx.available_rect();
        let dialog_size = egui::vec2(500.0, 250.0); // Larger dialog for more detailed errors
        let dialog_pos = screen_rect.center() - dialog_size / 2.0;
        
        // Convert message to RichText
        let rich_message = message.into();
        
        // Create a modal background overlay
        let _overlay_frame = egui::Frame::none()
            .fill(Color32::from_rgba_unmultiplied(0, 0, 0, 200));
        
        egui::Area::new("error_dialog_overlay")
            .fixed_pos(screen_rect.min)
            .movable(false)
            .interactable(true)
            .show(ctx, |ui| {
                ui.painter().rect_filled(
                    screen_rect,
                    0.0,
                    Color32::from_rgba_unmultiplied(0, 0, 0, 150)
                );
            });
        
        // Create the dialog window
        egui::Window::new(title)
            .fixed_pos(dialog_pos)
            .fixed_size(dialog_size)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .frame(error_dialog_frame())
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.heading(&t("error_dialog_title"));
                    ui.add_space(10.0);
                    
                    // Create a scrolling area for long error messages
                    egui::ScrollArea::vertical()
                        .max_height(150.0)
                        .show(ui, |ui| {
                            // Show message text with word wrap
                            ui.label(rich_message.size(16.0));
                        });
                    
                    ui.add_space(20.0);
                    
                    // Ok button
                    let _button_response = ui.with_layout(
                        egui::Layout::bottom_up(egui::Align::Center),
                        |ui| {
                            ui.horizontal(|ui| {
                                if ui.button(&t("error_dialog_ok")).clicked() {
                                    *open = false;
                                    result = true;
                                }
                            });
                        }
                    );
                });
            });
        
        // Prevent interaction with the rest of the UI while dialog is open
        ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground, 
            egui::Id::new("error_dialog_blocker")
        )).add(egui::Shape::Noop);
    }
    
    result
}
