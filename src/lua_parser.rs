// Parser for Lua shape files using nom
use std::fs;
use std::io;

use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::{char, digit1, multispace1, space0},
    combinator::{map, map_res, opt, recognize, value},
    multi::{many0, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple},
};

use crate::ast::{ShapesFile, Shape, Scale, Vertex, Port, PortType};

/// Error type for parser operations
#[derive(Debug)]
pub enum ParseError {
    IoError(io::Error),
    ParseError(String),
}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> Self {
        ParseError::IoError(err)
    }
}

/// Parses a shapes.lua file and returns a ShapesFile
pub fn parse_shapes_file(file_path: &str) -> Result<ShapesFile, ParseError> {
    let content = fs::read_to_string(file_path)?;
    parse_shapes_content(&content)
}

/// Parses a string containing shapes.lua content
pub fn parse_shapes_content(content: &str) -> Result<ShapesFile, ParseError> {
    match shapes_file(content) {
        Ok((_, shapes_file)) => Ok(shapes_file),
        Err(e) => Err(ParseError::ParseError(format!("Failed to parse: {}", e))),
    }
}

// Parses whitespace and comments
fn ws(input: &str) -> IResult<&str, ()> {
    map(
        many0(alt((
            map(multispace1, |_| ()),
            map(preceded(tag("--"), take_until("\n")), |_| ()),
        ))),
        |_| (),
    )(input)
}

// Parse a number (integer or float)
fn number(input: &str) -> IResult<&str, f32> {
    map_res(
        recognize(pair(
            opt(char('-')),
            alt((
                // Case one: .42
                recognize(tuple((
                    char('.'),
                    digit1,
                ))),
                // Case two: 42 or 42. or 42.42
                recognize(tuple((
                    digit1,
                    opt(tuple((
                        char('.'),
                        opt(digit1),
                    ))),
                ))),
            )),
        )),
        |s: &str| s.parse::<f32>(),
    )(input)
}

// Parse a vertex like "{x, y}"
fn vertex(input: &str) -> IResult<&str, Vertex> {
    map(
        delimited(
            char('{'),
            tuple((
                terminated(number, tuple((space0, char(','), space0))),
                number,
            )),
            preceded(space0, char('}')),
        ),
        |(x, y)| Vertex { x, y },
    )(input)
}

// Parse port type string
fn port_type(input: &str) -> IResult<&str, PortType> {
    map(
        take_while1(|c: char| c.is_alphabetic() || c == '_'),
        PortType::from_str,
    )(input)
}

// Parse a port like "{edge, position}" or "{edge, position, TYPE}"
fn port(input: &str) -> IResult<&str, Port> {
    map(
        delimited(
            char('{'),
            tuple((
                terminated(map_res(digit1, |s: &str| s.parse::<usize>()), tuple((space0, char(','), space0))),
                terminated(number, tuple((space0, opt(tuple((char(','), space0)))))),
                opt(port_type),
            )),
            preceded(space0, char('}')),
        ),
        |(edge, position, port_type)| Port { edge, position, port_type },
    )(input)
}

// Parse vertices section
fn vertices_section(input: &str) -> IResult<&str, Vec<Vertex>> {
    delimited(
        tuple((tag("verts"), space0, char('='), space0, char('{'))),
        separated_list0(
            tuple((space0, char(','), space0)),
            vertex,
        ),
        tuple((space0, opt(char(',')), space0, char('}')))
    )(input)
}

// Parse ports section
fn ports_section(input: &str) -> IResult<&str, Vec<Port>> {
    delimited(
        tuple((tag("ports"), space0, char('='), space0, char('{'))),
        separated_list0(
            tuple((space0, char(','), space0)),
            port,
        ),
        tuple((space0, opt(char(',')), space0, char('}')))
    )(input)
}

// Parse a scale (contains vertices and ports)
fn scale(input: &str) -> IResult<&str, Scale> {
    map(
        delimited(
            tuple((char('{'), ws)),
            tuple((
                terminated(vertices_section, ws),
                ports_section,
            )),
            tuple((ws, char('}')))
        ),
        |(verts, ports)| Scale { verts, ports },
    )(input)
}

// Parse launcher_radial property
fn launcher_radial(input: &str) -> IResult<&str, bool> {
    alt((
        value(true, tag("launcher_radial=true")),
        value(false, tag("launcher_radial=false")),
        value(true, tag("launcher_radial")),
    ))(input)
}

// Parse a shape name comment
fn shape_name(input: &str) -> IResult<&str, String> {
    map(
        preceded(tag("--"), take_while(|c: char| c != '\n' && c != '\r')),
        |s: &str| s.trim().to_string(),
    )(input)
}

// Parse a shape definition
fn shape(input: &str) -> IResult<&str, Shape> {
    map(
        tuple((
            // Shape ID with optional name
            delimited(
                char('{'),
                tuple((
                    map_res(digit1, |s: &str| s.parse::<usize>()),
                    opt(preceded(space0, shape_name)),
                )),
                tuple((ws, char('{'), ws))
            ),
            // Scales
            terminated(
                separated_list1(
                    tuple((ws, char(','), ws)),
                    scale,
                ),
                tuple((ws, char('}')))
            ),
            // Optional launcher_radial property
            opt(preceded(ws, launcher_radial)),
        )),
        |((id, name), scales, launcher_radial)| Shape {
            id,
            name,
            scales,
            launcher_radial,
        },
    )(input)
}

// Parse the entire shapes file
fn shapes_file(input: &str) -> IResult<&str, ShapesFile> {
    map(
        delimited(
            tuple((ws, char('{'), ws)),
            separated_list0(
                tuple((ws, char(','), ws)),
                shape,
            ),
            tuple((ws, char('}'), ws))
        ),
        |shapes| ShapesFile { shapes },
    )(input)
}

/// Serializes a ShapesFile back to a Lua string
pub fn serialize_shapes_file(shapes_file: &ShapesFile) -> String {
    let mut result = String::from("{\n");
    
    for (i, shape) in shapes_file.shapes.iter().enumerate() {
        result.push_str(&format!("  {{{}", shape.id));
        
        if let Some(name) = &shape.name {
            result.push_str(&format!("  --{}", name));
        }
        
        result.push_str("\n    {\n");
        
        for (j, scale) in shape.scales.iter().enumerate() {
            result.push_str("      {\n");
            
            // Vertices
            result.push_str("        verts={\n");
            for vert in &scale.verts {
                result.push_str(&format!("          {{{}, {}}},\n", vert.x, vert.y));
            }
            result.push_str("        },\n");
            
            // Ports
            result.push_str("        ports={\n");
            for port in &scale.ports {
                if let Some(port_type) = &port.port_type {
                    result.push_str(&format!("          {{{}, {}, {}}},\n", 
                                            port.edge, port.position, port_type.to_str()));
                } else {
                    result.push_str(&format!("          {{{}, {}}},\n", port.edge, port.position));
                }
            }
            result.push_str("        }\n");
            
            // End of scale
            if j < shape.scales.len() - 1 {
                result.push_str("      },\n");
            } else {
                result.push_str("      }\n");
            }
        }
        
        // Add launcher_radial property if present
        if let Some(launcher_radial) = shape.launcher_radial {
            if launcher_radial {
                result.push_str("    }\n    launcher_radial=true\n");
            }
        } else {
            result.push_str("    }\n");
        }
        
        // End of shape
        if i < shapes_file.shapes.len() - 1 {
            result.push_str("  },\n");
        } else {
            result.push_str("  }\n");
        }
    }
    
    result.push_str("}\n");
    result
} 