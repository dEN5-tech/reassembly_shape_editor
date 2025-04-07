use crate::ast::{ShapesFile, Shape, Scale, Vertex, Port, PortType, ShroudComponent, CannonProperties, ThrusterProperties, FragmentProperties};

/// Serializes a ShapesFile back to a Lua string
pub fn serialize_shapes_file(shapes_file: &ShapesFile) -> String {
    let mut result = String::from("{\n");
    
    for (i, shape) in shapes_file.shapes.iter().enumerate() {
        // Shape ID and optional name
        result.push_str(&format!("    {{{},", shape.id));
        
        if let Some(name) = &shape.name {
            result.push_str(&format!(" --{}", name));
        }
        
        result.push_str("\n");
        
        // Begin shape properties block
        result.push_str("        {\n");
        
        // Scales - special handling to match expected format
        for (j, scale) in shape.scales.iter().enumerate() {
            result.push_str("            {\n");
            
            // Vertices
            result.push_str("                verts = {");
            if scale.verts.is_empty() {
                result.push_str("}");
            } else {
                result.push_str("\n");
                for vert in &scale.verts {
                    result.push_str(&format!("                    {{{}, {}}},\n", vert.x, vert.y));
                }
                result.push_str("                }");
            }
            result.push_str(",\n");
            
            // Ports
            result.push_str("                ports = {");
            if scale.ports.is_empty() {
                result.push_str("}");
            } else {
                result.push_str("\n");
                for port in &scale.ports {
                    if let Some(port_type) = &port.port_type {
                        result.push_str(&format!("                    {{{}, {}, {}}},  -- Edge {}, position {}, type {}\n", 
                                                port.edge, port.position, port_type.to_str(), port.edge, port.position, port_type.to_str()));
                    } else {
                        result.push_str(&format!("                    {{{}, {}}},\n", port.edge, port.position));
                    }
                }
                result.push_str("                }");
            }
            
            // End of scale
            if j < shape.scales.len() - 1 {
                result.push_str(&format!("\n            }}, --scale {}\n", j+1));
            } else {
                result.push_str(&format!("\n            }} --scale {}\n", j+1));
            }
        }
        
        // Group
        if let Some(group) = shape.group {
            result.push_str(&format!("            group = {},\n", group));
        }

        // Features
        if let Some(features) = &shape.features {
            result.push_str(&format!("            features = \"{}\",\n", features.join("|")));
        }

        // Colors
        if let Some(color) = shape.fill_color {
            result.push_str(&format!("            fillColor = 0x{:08x},\n", color));
        }
        if let Some(color) = shape.fill_color1 {
            result.push_str(&format!("            fillColor1 = 0x{:08x},\n", color));
        }
        if let Some(color) = shape.line_color {
            result.push_str(&format!("            lineColor = 0x{:08x},\n", color));
        }

        // Physical properties
        if let Some(durability) = shape.durability {
            result.push_str(&format!("            durability = {},\n", durability));
        }
        if let Some(density) = shape.density {
            result.push_str(&format!("            density = {},\n", density));
        }
        if let Some(grow_rate) = shape.grow_rate {
            result.push_str(&format!("            growRate = {},\n", grow_rate));
        }

        // Launcher radial property
        if let Some(launcher_radial) = shape.launcher_radial {
            if launcher_radial {
                result.push_str("            launcher_radial = true,\n");
            } else {
                result.push_str("            launcher_radial = false,\n");
            }
        }

        // Mirror reference
        if let Some(mirror_of) = shape.mirror_of {
            result.push_str(&format!("            mirror_of = {},\n", mirror_of));
        }

        // Shroud components
        if let Some(shroud) = &shape.shroud {
            result.push_str("            shroud = {\n");
            for component in shroud {
                result.push_str(&format!("                {{size = {{{}, {}}}, offset = {{{}, {}, {}}}, taper = {}, count = {}, angle = {}, tri_color_id = {}, tri_color1_id = {}, line_color_id = {}, shape = {}}},\n",
                    component.size.0, component.size.1,
                    component.offset.0, component.offset.1, component.offset.2,
                    component.taper, component.count, component.angle,
                    component.tri_color_id, component.tri_color1_id, component.line_color_id,
                    component.shape));
            }
            result.push_str("            },\n");
        }

        // Cannon properties
        if let Some(cannon) = &shape.cannon {
            result.push_str("            cannon = {\n");
            result.push_str(&format!("                damage = {},\n", cannon.damage));
            result.push_str(&format!("                power = {},\n", cannon.power));
            result.push_str(&format!("                roundsPerSec = {},\n", cannon.rounds_per_sec));
            result.push_str(&format!("                muzzleVel = {},\n", cannon.muzzle_vel));
            result.push_str(&format!("                range = {},\n", cannon.range));
            result.push_str(&format!("                spread = {},\n", cannon.spread));
            
            if let Some(rounds) = cannon.rounds_per_burst {
                result.push_str(&format!("                roundsPerBurst = {},\n", rounds));
            }
            if let Some(burstyness) = cannon.burstyness {
                result.push_str(&format!("                burstyness = {},\n", burstyness));
            }
            if let Some(color) = cannon.color {
                result.push_str(&format!("                color = 0x{:08x},\n", color));
            }
            if let Some(explosive) = &cannon.explosive {
                result.push_str(&format!("                explosive = {},\n", explosive));
            }
            if let Some(fragment) = &cannon.fragment {
                result.push_str("                fragment = {\n");
                result.push_str(&format!("                    roundsPerBurst = {},\n", fragment.rounds_per_burst));
                result.push_str(&format!("                    muzzleVel = {},\n", fragment.muzzle_vel));
                result.push_str(&format!("                    spread = {},\n", fragment.spread));
                if let Some(pattern) = &fragment.pattern {
                    result.push_str(&format!("                    pattern = \"{}\",\n", pattern));
                }
                result.push_str(&format!("                    damage = {},\n", fragment.damage));
                result.push_str(&format!("                    range = {},\n", fragment.range));
                if let Some(color) = fragment.color {
                    result.push_str(&format!("                    color = 0x{:08x},\n", color));
                }
                result.push_str("                },\n");
            }
            result.push_str("            },\n");
        }

        // Thruster properties
        if let Some(thruster) = &shape.thruster {
            result.push_str("            thruster = {\n");
            result.push_str(&format!("                force = {},\n", thruster.force));
            result.push_str(&format!("                power = {},\n", thruster.power));
            if let Some(color) = thruster.color {
                result.push_str(&format!("                color = 0x{:08x},\n", color));
            }
            result.push_str("            },\n");
        }
        
        // End of shape properties block
        result.push_str("        }");
        
        // End of shape
        if i < shapes_file.shapes.len() - 1 {
            result.push_str("\n    },\n");
        } else {
            result.push_str("\n    }\n");
        }
    }
    
    result.push_str("}\n");
    result
}