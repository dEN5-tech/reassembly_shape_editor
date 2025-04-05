// Data structures module

// Структура точки (вершины)
#[derive(Clone, Debug, PartialEq)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
}

// Структура порта
#[derive(Clone, Debug, PartialEq)]
pub struct Port {
    pub edge: usize,
    pub position: f32,
    pub port_type: PortType,
}

// Перечисление типов портов
#[derive(Clone, Debug, PartialEq)]
pub enum PortType {
    Default,
    ThrusterIn,
    ThrusterOut,
    Missile,
    Launcher,
    WeaponIn,
    WeaponOut,
    Root,
    None,
}

// Получение строкового представления типа порта
impl PortType {
    pub fn to_string(&self) -> String {
        match self {
            PortType::Default => "DEFAULT".to_string(),
            PortType::ThrusterIn => "THRUSTER_IN".to_string(),
            PortType::ThrusterOut => "THRUSTER_OUT".to_string(),
            PortType::Missile => "MISSILE".to_string(),
            PortType::Launcher => "LAUNCHER".to_string(),
            PortType::WeaponIn => "WEAPON_IN".to_string(),
            PortType::WeaponOut => "WEAPON_OUT".to_string(),
            PortType::Root => "ROOT".to_string(),
            PortType::None => "NONE".to_string(),
        }
    }
    
    // Parse port type from string
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "DEFAULT" => Some(PortType::Default),
            "THRUSTER_IN" => Some(PortType::ThrusterIn),
            "THRUSTER_OUT" => Some(PortType::ThrusterOut),
            "MISSILE" => Some(PortType::Missile),
            "LAUNCHER" => Some(PortType::Launcher),
            "WEAPON_IN" => Some(PortType::WeaponIn),
            "WEAPON_OUT" => Some(PortType::WeaponOut),
            "ROOT" => Some(PortType::Root),
            "NONE" => Some(PortType::None),
            _ => None,
        }
    }
}

// Структура формы
#[derive(Clone, Debug)]
pub struct Shape {
    pub id: usize,
    pub name: String,
    pub vertices: Vec<Vertex>,
    pub ports: Vec<Port>,
    pub selected_vertex: Option<usize>,
    pub selected_port: Option<usize>,
    pub launcher_radial: bool,
}

// Implement PartialEq to compare shapes for undo/redo functionality
impl PartialEq for Shape {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id &&
        self.name == other.name &&
        self.vertices == other.vertices &&
        self.ports == other.ports &&
        self.launcher_radial == other.launcher_radial
        // Note: We deliberately exclude selected_vertex and selected_port from comparison
        // since those are UI state rather than actual data we want to track for undo/redo
    }
}

impl Shape {
    pub fn new(id: usize) -> Self {
        Shape {
            id,
            name: format!("Shape_{}", id),
            vertices: vec![],
            ports: vec![],
            selected_vertex: None,
            selected_port: None,
            launcher_radial: false,
        }
    }

    // Генерация Lua кода для формы
    pub fn to_lua(&self) -> String {
        let mut lua = format!("    {{{}  --{}\n        {{\n            {{\n", self.id, self.name);
        
        // Добавление вершин
        lua.push_str("                verts={\n");
        for v in &self.vertices {
            lua.push_str(&format!("                    {{{}, {}}},\n", v.x, v.y));
        }
        lua.push_str("                },\n");
        
        // Добавление портов
        lua.push_str("                ports={\n");
        for p in &self.ports {
            if p.port_type == PortType::Default {
                lua.push_str(&format!("                    {{{}, {}}},\n", p.edge, p.position));
            } else {
                lua.push_str(&format!("                    {{{}, {}, {}}},\n", 
                    p.edge, p.position, p.port_type.to_string()));
            }
        }
        lua.push_str("                }\n");
        lua.push_str("            }\n        }\n    }");
        
        lua
    }
} 