use std::io;
use std::path::Path;
use std::fs;
use full_moon::{
    ast, parse,
    visitors::Visitor,
    node::Node,
};
use full_moon::tokenizer::Symbol::Minus;

use crate::ast::{ShapesFile, Shape, Scale, Vertex, Port, PortType, ShroudComponent, CannonProperties, ThrusterProperties, FragmentProperties};

/// Error type for parsing operations
#[derive(Debug)]
pub enum ParserErrorKind {
    IoError(io::Error),
    ParseError(String),
}

/// Wrapper for parser errors
#[derive(Debug)]
pub struct ParseError {
    pub kind: ParserErrorKind,
}

impl From<io::Error> for ParseError {
    fn from(error: io::Error) -> Self {
        ParseError {
            kind: ParserErrorKind::IoError(error),
        }
    }
}

impl From<String> for ParseError {
    fn from(error: String) -> Self {
        ParseError {
            kind: ParserErrorKind::ParseError(error),
        }
    }
}

/// Parse a Lua shapes file from a file path
pub fn parse_shapes_file(path: &Path) -> Result<ShapesFile, ParseError> {
    let content = fs::read_to_string(path)?;
    parse_shapes_content(&content).map_err(|e| e.into())
}

/// Parse a Lua shapes file into our AST representation
pub fn parse_shapes_content(lua_content: &str) -> Result<ShapesFile, String> {
    // Attempt to fix common syntax issues
    let processed_content = fix_lua_syntax(lua_content);

    let valid_lua = format!("return {}", processed_content);
    let ast = match parse(&valid_lua) {
        Ok(ast) => ast,
        Err(_) => {
            // Try fallback legacy parser
            return legacy_parse_shapes(lua_content);
        }
    };
    
    // Find the table constructor which should contain the shapes table
    // First try to find a return statement
    let mut shapes_table = None;
    
    if let Some(last_stmt) = ast.nodes().last_stmt() {
        if let ast::LastStmt::Return(ret) = last_stmt {
            if let Some(expr) = ret.returns().first() {
                if let ast::Expression::TableConstructor(table) = expr.value() {
                    shapes_table = Some(table);
                }
            }
        }
    }
    
    // If no return statement, look for a top-level table
    if shapes_table.is_none() {
        for stmt in ast.nodes().stmts() {
            if let ast::Stmt::Assignment(assign) = stmt {
                if let Some(expr) = assign.expressions().iter().next() {
                    if let ast::Expression::TableConstructor(table) = expr {
                        shapes_table = Some(&table);
                        break;
                    }
                }
            } else if let ast::Stmt::LocalAssignment(assign) = stmt {
                if let Some(expr) = assign.expressions().iter().next() {
                    if let ast::Expression::TableConstructor(table) = expr {
                        shapes_table = Some(&table);
                        break;
                    }
                }
            }
        }
    }
    
    // If still no table found, check for a standalone table
    if shapes_table.is_none() {
        for stmt in ast.nodes().stmts() {
            // Note: full_moon doesn't have ExprStmt variant, we need to check what's available
            // in the actual Stmt enum for the version of full_moon being used
            if let ast::Stmt::LocalAssignment(assign) = stmt {
                if let Some(expr) = assign.expressions().iter().next() {
                    if let ast::Expression::TableConstructor(table) = expr {
                        shapes_table = Some(&table);
                        break;
                    }
                }
            }
        }
    }
    
    if let Some(table) = shapes_table {
        let mut shapes_file = ShapesFile { shapes: Vec::new() };
        
        // Process each field in the table as a shape
        for field in table.fields() {
            if let ast::Field::NoKey(expr) = field {
                if let ast::Expression::TableConstructor(shape_table) = expr {
                    if let Some(shape) = extract_shape(shape_table) {
                        shapes_file.shapes.push(shape);
                    }
                }
            }
        }
        
        if shapes_file.shapes.is_empty() {
            return legacy_parse_shapes(lua_content);
        }
        
        return Ok(shapes_file);
    }
    
    legacy_parse_shapes(lua_content)
}

// Function to fix common Lua syntax issues
fn fix_lua_syntax(content: &str) -> String {
    let mut fixed = content.to_string();
    
    // Add missing commas between table entries
    fixed = fixed.replace("}\n\t{", "},\n\t{");
    fixed = fixed.replace("}\n{", "},\n{");
    
    // Fix launcher_radial property formatting
    fixed = fixed.replace("launcher_radial=", "launcher_radial = ");
    fixed = fixed.replace("launcher_radial", "launcher_radial = true");
    
    fixed
}

// A simpler, more direct approach to parse shapes from Lua files
fn legacy_parse_shapes(content: &str) -> Result<ShapesFile, String> {
    let mut shapes = Vec::new();
    let lines = content.lines().collect::<Vec<_>>();
    
    // Find shapes by tracking opening and closing braces
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with("--") {
            i += 1;
            continue;
        }
        
        // Look for shape definitions that start with { and a number
        if line.starts_with("{") && line.contains(",") {
            let parts = line.trim_matches(|c| c == '{' || c == '}' || c == ',').split(',').collect::<Vec<_>>();
            if !parts.is_empty() {
                if let Ok(id) = parts[0].trim().parse::<usize>() {
                    // Found a shape with ID
                    let (shape, new_index) = parse_shape(id, &lines, i);
                    shapes.push(shape);
                    i = new_index;
                    continue;
                }
            }
        }
        
        i += 1;
    }
    
    Ok(ShapesFile { shapes })
}

// Parse a single shape from the lines starting at the given index
fn parse_shape(id: usize, lines: &[&str], start_index: usize) -> (Shape, usize) {
    let mut scales = Vec::new();
    let mut launcher_radial = None;
    let mut i = start_index + 1; // Skip the ID line
    let mut brace_level = 1; // We're already inside one level of braces
    
    while i < lines.len() && brace_level > 0 {
        let line = lines[i].trim();
        
        // Track brace levels
        brace_level += line.matches('{').count();
        brace_level -= line.matches('}').count();
        
        // Check for launcher_radial property
        if line.contains("launcher_radial") {
            launcher_radial = Some(true);
        }
        
        // Looking for scale definitions
        if line.contains("verts") && line.contains("{") {
            let (scale, new_index) = parse_scale(&lines, i);
            if !scale.verts.is_empty() {
                scales.push(scale);
            }
            i = new_index;
            continue;
        }
        
        i += 1;
    }
    
    let shape = Shape {
        id,
        name: None, // Could extract from comments if needed
        scales,
        launcher_radial,
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
    };
    
    (shape, i)
}

// Parse a scale definition from the lines starting at the given index
fn parse_scale(lines: &[&str], start_index: usize) -> (Scale, usize) {
    let mut verts = Vec::new();
    let mut ports = Vec::new();
    let mut i = start_index;
    let mut in_verts = false;
    let mut in_ports = false;
    
    while i < lines.len() {
        let line = lines[i].trim();
        
        // Check what section we're in
        if line.contains("verts") {
            in_verts = true;
            in_ports = false;
        } else if line.contains("ports") {
            in_verts = false;
            in_ports = true;
        }
        
        // Parse vertices
        if in_verts && line.contains("{") && line.contains(",") {
            let coords = line.trim_matches(|c| c == '{' || c == '}' || c == ',').split(',').collect::<Vec<_>>();
            if coords.len() >= 2 {
                if let (Ok(x), Ok(y)) = (coords[0].trim().parse::<f32>(), coords[1].trim().parse::<f32>()) {
                    verts.push(Vertex { x, y });
                }
            }
        }
        
        // Parse ports
        if in_ports && line.contains("{") && line.contains(",") {
            let parts = line.trim_matches(|c| c == '{' || c == '}' || c == ',').split(',').collect::<Vec<_>>();
            if parts.len() >= 2 {
                if let (Ok(edge), Ok(position)) = (parts[0].trim().parse::<usize>(), parts[1].trim().parse::<f32>()) {
                    let port_type = if parts.len() >= 3 {
                        let type_str = parts[2].trim();
                        Some(PortType::from_str(type_str))
                    } else {
                        None
                    };
                    
                    ports.push(Port {
                        edge,
                        position,
                        port_type,
                    });
                }
            }
        }
        
        // End of scale definition
        if line == "}" || line == "}," {
            break;
        }
        
        i += 1;
    }
    
    (Scale { verts, ports }, i)
}

/// Extract a shape from a Lua table constructor
fn extract_shape(table: &ast::TableConstructor) -> Option<Shape> {
    let mut id = None;
    let name = None;
    let mut scales = Vec::new();
    let mut launcher_radial = None;
    
    // Process each field in the shape table
    for (i, field) in table.fields().into_iter().enumerate() {
        match field {
            ast::Field::NoKey(expr) => {
                // First field should be the ID
                if i == 0 {
                    if let ast::Expression::Number(num) = expr {
                        if let Ok(id_val) = num.token().to_string().parse::<usize>() {
                            id = Some(id_val);
                        }
                    }
                }
                // Second field should be the scales table
                else if i == 1 {
                    if let ast::Expression::TableConstructor(scales_table) = expr {
                        // Each entry in the scales table is a scale
                        for scale_field in scales_table.fields().into_iter() {
                            if let ast::Field::NoKey(expr) = scale_field {
                                if let ast::Expression::TableConstructor(scale_table) = expr {
                                    let mut verts = Vec::new();
                                    let mut ports = Vec::new();
                                    
                                    // Iterate through fields in the scale table
                                    for def_field in scale_table.fields().into_iter() {
                                        if let ast::Field::NameKey { key, value, .. } = def_field {
                                            let key_str = key.token().to_string();
                                            
                                            // Parse vertices
                                            if key_str == "verts" {
                                                if let ast::Expression::TableConstructor(verts_table) = value {
                                                    for vert_field in verts_table.fields().into_iter() {
                                                        if let ast::Field::NoKey(expr) = vert_field {
                                                            if let ast::Expression::TableConstructor(vert_table) = expr {
                                                                let mut x = None;
                                                                let mut y = None;
                                                                
                                                                for (m, coord) in vert_table.fields().into_iter().enumerate() {
                                                                    if let ast::Field::NoKey(expr) = coord {
                                                                        match expr {
                                                                            // Handle regular numbers
                                                                            ast::Expression::Number(num) => {
                                                                                let val = num.token().to_string().parse::<f32>().ok();
                                                                                if m == 0 {
                                                                                    x = val;
                                                                                } else if m == 1 {
                                                                                    y = val;
                                                                                }
                                                                            },
                                                                            // Handle negative numbers (represented as UnaryOperator with Minus)
                                                                            ast::Expression::UnaryOperator { unop, expression } => {
                                                                                if *unop.token().token_type() == (full_moon::tokenizer::TokenType::Symbol { symbol: Minus }) {
                                                                                    if let ast::Expression::Number(num) = &**expression {
                                                                                        let val_str = num.token().to_string();
                                                                                        if let Ok(val) = val_str.parse::<f32>() {
                                                                                            let neg_val = -val;
                                                                                            if m == 0 {
                                                                                                x = Some(neg_val);
                                                                                            } else if m == 1 {
                                                                                                y = Some(neg_val);
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            },
                                                                            _ => {}
                                                                        }
                                                                    }
                                                                }
                                                                
                                                                if let (Some(x), Some(y)) = (x, y) {
                                                                    verts.push(Vertex { x, y });
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            } 
                                            // Parse ports
                                            else if key_str == "ports" {
                                                if let ast::Expression::TableConstructor(ports_table) = value {
                                                    for port_field in ports_table.fields().into_iter() {
                                                        if let ast::Field::NoKey(expr) = port_field {
                                                            if let ast::Expression::TableConstructor(port_table) = expr {
                                                                let mut edge = None;
                                                                let mut position = None;
                                                                let mut port_type = None;
                                                                
                                                                for (m, field) in port_table.fields().into_iter().enumerate() {
                                                                    if let ast::Field::NoKey(expr) = field {
                                                                        if m == 0 {
                                                                            if let ast::Expression::Number(num) = expr {
                                                                                edge = num.token().to_string().parse::<usize>().ok();
                                                                            }
                                                                        } else if m == 1 {
                                                                            if let ast::Expression::Number(num) = expr {
                                                                                position = num.token().to_string().parse::<f32>().ok();
                                                                            }
                                                                        } else if m == 2 {
                                                                            // Handle port type - don't try to use the String variant which may not exist
                                                                            
                                                                            // Use inspect pattern to capture the token value without assuming variant
                                                                            let type_str = match expr {
                                                                                ast::Expression::Symbol(token) => token.token().to_string(),
                                                                                // Use string representation for any other variant
                                                                                _ => format!("{:?}", expr),
                                                                            };
                                                                            port_type = Some(PortType::from_str(&type_str));
                                                                        }
                                                                    }
                                                                }
                                                                
                                                                if let (Some(edge), Some(position)) = (edge, position) {
                                                                    ports.push(Port {
                                                                        edge,
                                                                        position,
                                                                        port_type,
                                                                    });
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    
                                    scales.push(Scale { verts, ports });
                                }
                            }
                        }
                    }
                }
            },
            // Handle named properties at the shape level like "launcher_radial"
            ast::Field::NameKey { key, value, .. } => {
                let key_str = key.token().to_string();
                
                if key_str == "launcher_radial" {
                    // Default to true if the property exists
                    launcher_radial = Some(true);
                    
                    // Try to extract more specific value if available
                    if let ast::Expression::Symbol(symbol) = value {
                        let val_str = symbol.token().to_string();
                        if val_str == "false" {
                            launcher_radial = Some(false);
                        }
                    }
                    // Any other cases simply use the default true value
                }
                // Add more property handlers here as needed
            },
            // Handle any other field types we don't explicitly handle
            _ => {}
        }
    }
    
    if let Some(id) = id {
        Some(Shape {
            id,
            name,
            scales,
            launcher_radial,
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
        })
    } else {
        None
    }
}