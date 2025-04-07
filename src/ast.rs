// Abstract Syntax Tree for parsing Lua shape definitions
use serde::Serialize;

/// Represents a complete shapes definition file
/// 
/// A shapes file contains a list of shape definitions enclosed in braces.
/// Each shape must follow these rules:
/// - Must be enclosed in outer braces {}
/// - Each shape must be separated by commas
/// - File must end with a newline
/// 
/// Example:
/// ```lua
/// {
///   {1001, --shape_name  -- ID and optional name
///     {                  -- Properties block
///       {               -- Scale block
///         verts={       -- Vertices section
///           {20, 10},   -- Each vertex as {x, y}
///           {30, 10},
///           {30, 20}
///         },
///         ports={       -- Ports section
///           {0, 0.5},   -- {edge_index, position}
///           {1, 0.5, THRUSTER_OUT}  -- {edge_index, position, type}
///         }
///       }              -- End scale
///     }                -- End properties
///   }                  -- End shape
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct ShapesFile {
    pub shapes: Vec<Shape>,
}

/// Represents a single shape definition
/// 
/// Each shape must have:
/// - A unique ID between 100-10000
/// - At least one scale definition
/// - Optional name comment after ID
/// - Optional properties like launcher_radial, colors, etc.
/// 
/// Shape ID Rules:
/// - Must be unique across all shapes
/// - Must be between 100-10000
/// - Lower numbers (100-999) recommended for basic shapes
/// - Higher numbers (1000+) for complex shapes
/// 
/// Example:
/// ```lua
/// {1001, --shape_name
///   {
///     {
///       verts={{0,0}, {1,0}, {0,1}},
///       ports={{0, 0.5}}
///     }
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct Shape {
    pub id: usize,
    pub name: Option<String>,
    pub scales: Vec<Scale>,
    pub launcher_radial: Option<bool>,
    pub mirror_of: Option<usize>,
    pub group: Option<usize>,
    pub features: Option<Vec<String>>,
    pub fill_color: Option<u32>,
    pub fill_color1: Option<u32>,
    pub line_color: Option<u32>,
    pub durability: Option<f32>,
    pub density: Option<f32>,
    pub grow_rate: Option<f32>,
    pub shroud: Option<Vec<ShroudComponent>>,
    pub cannon: Option<CannonProperties>,
    pub thruster: Option<ThrusterProperties>,
}

/// Represents a scale variant of a shape
/// 
/// Each scale must have:
/// - At least 3 vertices forming a convex polygon
/// - Optional ports on any edges
/// - Vertices must be defined in clockwise or counter-clockwise order
/// - No duplicate vertices allowed
/// 
/// Scale Rules:
/// - Multiple scales allowed for LOD (Level of Detail)
/// - Each scale should maintain same general shape
/// - Port positions should be consistent between scales
/// 
/// Example:
/// ```lua
/// {
///   verts={          -- Vertices section
///     {0,0},         -- First vertex
///     {1,0},         -- Second vertex
///     {0,1}          -- Third vertex
///   },
///   ports={          -- Ports section
///     {0, 0.5},      -- Port on first edge at 50%
///     {2, 0.25}      -- Port on third edge at 25%
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct Scale {
    pub verts: Vec<Vertex>,
    pub ports: Vec<Port>,
}

/// Represents a vertex with X, Y coordinates
/// 
/// Vertices must:
/// - Be unique (no duplicates)
/// - Form a convex polygon when connected in sequence
/// - Be defined in consistent order (clockwise or counter-clockwise)
/// - Use reasonable coordinate values (typically -100 to 100)
/// 
/// Example: {x, y}
/// ```lua
/// {20, 10}   -- x=20, y=10
/// {-5, 3.5}  -- Fractional coordinates allowed
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
}

/// Represents a port with edge, position and optional type
/// 
/// Port Rules:
/// - edge: Valid vertex index (0-based)
/// - position: Fraction along edge (0.0-1.0)
/// - type (optional): Valid port type
/// 
/// Port Position Formula:
/// For n equally spaced ports:
/// position = (1/n)/2 + k*(1/n) where k=0..n-1
/// 
/// Example: For 4 equally spaced ports:
/// ```lua
/// {0, 0.125}  -- First port  (k=0)
/// {0, 0.375}  -- Second port (k=1)
/// {0, 0.625}  -- Third port  (k=2)
/// {0, 0.875}  -- Fourth port (k=3)
/// ```
/// 
/// Port Types:
/// ```lua
/// {0, 0.5}                 -- Default connection port
/// {0, 0.5, THRUSTER_OUT}   -- Thruster output
/// {1, 0.5, THRUSTER_IN}    -- Thruster input
/// {2, 0.5, WEAPON_OUT}     -- Weapon output
/// {3, 0.5, WEAPON_IN}      -- Weapon input
/// {0, 0.5, LAUNCHER}       -- Object generator
/// {1, 0.5, MISSILE}        -- Missile attachment
/// {2, 0.5, ROOT}          -- Environment attachment
/// {3, 0.5, NONE}          -- No connection
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct Port {
    pub edge: usize,
    pub position: f32,
    pub port_type: Option<PortType>,
}

/// Port types supported in Reassembly
/// 
/// Port Type Rules:
/// - THRUSTER_OUT: Where thrust is generated (only one per thruster)
/// - THRUSTER_IN: Where thrusters can connect (multiple allowed)
/// - WEAPON_OUT: Where projectiles are generated (weapon source)
/// - WEAPON_IN: Where weapons can connect (weapon target)
/// - LAUNCHER: Where launched objects are generated
/// - MISSILE: Where missiles attach and generate thrust
/// - ROOT: For attaching to environment blocks
/// - NONE: No special behavior
/// - Default: Standard connection point
#[derive(Debug, Clone, PartialEq, Serialize)]
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

/// Represents a shroud decoration component
#[derive(Debug, Clone, Serialize)]
pub struct ShroudComponent {
    pub size: (f32, f32),
    pub offset: (f32, f32, f32),
    pub taper: f32,
    pub count: usize,
    pub angle: f32,
    pub tri_color_id: usize,
    pub tri_color1_id: usize,
    pub line_color_id: usize,
    pub shape: String,
}

/// Properties for cannon weapons
#[derive(Debug, Clone, Serialize)]
pub struct CannonProperties {
    pub damage: f32,
    pub power: f32,
    pub rounds_per_sec: f32,
    pub muzzle_vel: f32,
    pub range: f32,
    pub spread: f32,
    pub rounds_per_burst: Option<usize>,
    pub burstyness: Option<f32>,
    pub color: Option<u32>,
    pub explosive: Option<String>,
    pub fragment: Option<FragmentProperties>,
}

/// Properties for thruster components
#[derive(Debug, Clone, Serialize)]
pub struct ThrusterProperties {
    pub force: f32,
    pub power: f32,
    pub color: Option<u32>,
}

/// Properties for explosive fragments
#[derive(Debug, Clone, Serialize)]
pub struct FragmentProperties {
    pub rounds_per_burst: usize,
    pub muzzle_vel: f32,
    pub spread: f32,
    pub pattern: Option<String>,
    pub damage: f32,
    pub range: f32,
    pub color: Option<u32>,
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

impl std::fmt::Display for PortType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
} 