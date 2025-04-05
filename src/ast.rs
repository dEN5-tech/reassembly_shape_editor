// Abstract Syntax Tree for parsing Lua shape definitions

/// Represents a complete shapes definition file
#[derive(Debug, Clone)]
pub struct ShapesFile {
    pub shapes: Vec<Shape>,
}

/// Represents a single shape definition
#[derive(Debug, Clone)]
pub struct Shape {
    pub id: usize,
    pub name: Option<String>,
    pub scales: Vec<Scale>,
    pub launcher_radial: Option<bool>,
}

/// Represents a scale variant of a shape
#[derive(Debug, Clone)]
pub struct Scale {
    pub verts: Vec<Vertex>,
    pub ports: Vec<Port>,
}

/// Represents a vertex with X, Y coordinates
#[derive(Debug, Clone)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
}

/// Represents a port with edge, position and optional type
#[derive(Debug, Clone)]
pub struct Port {
    pub edge: usize,
    pub position: f32,
    pub port_type: Option<PortType>,
}

/// Port types supported in Reassembly
#[derive(Debug, Clone, PartialEq)]
pub enum PortType {
    Default,
    ThrusterIn,
    ThrusterOut,
    WeaponIn,
    WeaponOut,
    Missile,
    Launcher,
    Root,
    None,
}

impl PortType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "THRUSTER_IN" => PortType::ThrusterIn,
            "THRUSTER_OUT" => PortType::ThrusterOut,
            "WEAPON_IN" => PortType::WeaponIn,
            "WEAPON_OUT" => PortType::WeaponOut,
            "MISSILE" => PortType::Missile,
            "LAUNCHER" => PortType::Launcher,
            "ROOT" => PortType::Root,
            "NONE" => PortType::None,
            _ => PortType::Default,
        }
    }
    
    pub fn to_str(&self) -> &'static str {
        match self {
            PortType::Default => "DEFAULT",
            PortType::ThrusterIn => "THRUSTER_IN",
            PortType::ThrusterOut => "THRUSTER_OUT",
            PortType::WeaponIn => "WEAPON_IN",
            PortType::WeaponOut => "WEAPON_OUT",
            PortType::Missile => "MISSILE",
            PortType::Launcher => "LAUNCHER",
            PortType::Root => "ROOT",
            PortType::None => "NONE",
        }
    }
} 