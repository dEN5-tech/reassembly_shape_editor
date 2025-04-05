// Project generator for Reassembly mods
use std::fs;
use std::path::{Path, PathBuf};
use std::io;
use std::io::Write;

// Main function to generate a new Reassembly mod project
pub fn generate_project(project_name: &str) -> Result<(), io::Error> {
    println!("Generating Reassembly mod project: {}", project_name);
    
    // Create the project directory
    let project_dir = PathBuf::from(project_name);
    if project_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Project directory '{}' already exists", project_name)
        ));
    }
    
    fs::create_dir(&project_dir)?;
    
    // Create necessary sub-directories
    fs::create_dir(project_dir.join("ships"))?;
    fs::create_dir(project_dir.join("extra_ships"))?;
    
    // Create the shapes.lua file
    create_shapes_lua(&project_dir)?;
    
    // Create shape reference with common patterns
    create_shape_reference(&project_dir)?;
    
    // Create the blocks.lua file (template)
    create_blocks_lua(&project_dir)?;
    
    // Create factions.lua file (template)
    create_factions_lua(&project_dir)?;
    
    // Create regions.lua file (template)
    create_regions_lua(&project_dir)?;
    
    // Create a sample starter ship file
    create_sample_ship(&project_dir)?;
    
    // Create a README.md file with instructions
    create_readme(&project_dir, project_name)?;
    
    // Create cvars.txt file
    create_cvars(&project_dir)?;
    
    // Create preview.png placeholder reminder
    create_preview_reminder(&project_dir)?;
    
    println!("Project created successfully. Open the README.md file for instructions.");
    
    Ok(())
}

// Create a basic shapes.lua file with a sample shape
fn create_shapes_lua(project_dir: &Path) -> Result<(), io::Error> {
    let path = project_dir.join("shapes.lua");
    let mut file = fs::File::create(path)?;
    
    write!(file, "{}", r#"{
    {5001  --Square
        {
            {
                verts={
                    {5, -5},
                    {-5, -5},
                    {-5, 5},
                    {5, 5},
                },
                ports={
                    {0, 0.5},
                    {1, 0.5},
                    {2, 0.5},
                    {3, 0.5},
                }
            },
            {
                verts={
                    {10, -10},
                    {-10, -10},
                    {-10, 10},
                    {10, 10},
                },
                ports={
                    {0, 0.25},
                    {0, 0.75},
                    {1, 0.25},
                    {1, 0.75},
                    {2, 0.25},
                    {2, 0.75},
                    {3, 0.25},
                    {3, 0.75},
                }
            }
        }
    },
}
"#)?;
    
    Ok(())
}

// Create a template blocks.lua file
fn create_blocks_lua(project_dir: &Path) -> Result<(), io::Error> {
    let path = project_dir.join("blocks.lua");
    let mut file = fs::File::create(path)?;
    
    write!(file, "{}", r#"{
    -- New blocks should use IDs between 1 and 199 or 17000-26000
    {1,
        name="Custom Block",
        features=TURRET|CANNON,  -- Use modifiers like CANNON, TURRET, SHIELD etc.
        group=20,  -- Set this to your faction number
        shape=5001, -- Uses custom shape ID from shapes.lua
        points=30,
        durability=0.500,
        blurb="A custom block using a custom shape",
        density=0.150,
        fillColor=0x113077,
        fillColor1=0x205079,
        lineColor=0x3390eb,
        cannon={
            roundsPerSec=4.000,
            roundsPerBurst=3,
            muzzleVel=1400.000,
            spread=0.020,
            damage=120.000,
            color=0x47081,
            range=1200.000
        }
    }
}
"#)?;
    
    Ok(())
}

// Create a template factions.lua file
fn create_factions_lua(project_dir: &Path) -> Result<(), io::Error> {
    let path = project_dir.join("factions.lua");
    let mut file = fs::File::create(path)?;
    
    write!(file, "{}", r#"{
    -- Faction ID (should be between 20 and 100)
    {20,
        name="Custom Faction",
        color0=0x113077, -- Primary color
        color1=0x205079, -- Secondary color
        primaries=2,     -- Number of colors player can select (2 or 3)
        playable=2,      -- 2=unlocked by default, 1=needs to be unlocked, 0=not playable
        aiflags=WANDER|SOCIAL|DODGES|FLOCKING, -- AI behavior flags
        start="20_starter", -- Starting ship file in ships/ directory
    }
}
"#)?;
    
    Ok(())
}

// Create a template regions.lua file
fn create_regions_lua(project_dir: &Path) -> Result<(), io::Error> {
    let path = project_dir.join("regions.lua");
    let mut file = fs::File::create(path)?;
    
    write!(file, "{}", r#"{
    -- This adds a new region to the game without replacing the default ones
    subregions = {
        {
            ident = 208, -- Region identifier (will be relocated)
            faction = 20, -- Your faction ID
            count = 4,    -- Number of regions to generate
            radius = { 0.1, 0.15 }, -- Region size
            position = { 0.3, 0.8 }, -- Position in galaxy
            fleets = { { 20, { { 0, 1000}, {1, 600} } } }, -- Ship point values based on distance
            ambient = { 0 },
            -- Define unique ships that will appear in this region
            unique = {
                { "20_ship1", "20_ship2", "20_station1" }
            },
            fortressCount = { 1, 3 },
        }
    }
}
"#)?;
    
    Ok(())
}

// Create a sample ship file
fn create_sample_ship(project_dir: &Path) -> Result<(), io::Error> {
    let ships_dir = project_dir.join("ships");
    let path = ships_dir.join("20_starter.lua");
    let mut file = fs::File::create(path)?;
    
    write!(file, "{}", r#"-- This is a placeholder for your starter ship
-- Use the Export Ship feature in the game or create manually
{blocks={}}
"#)?;
    
    Ok(())
}

// Create a README file with instructions
fn create_readme(project_dir: &Path, project_name: &str) -> Result<(), io::Error> {
    let path = project_dir.join("README.md");
    let mut file = fs::File::create(path)?;
    
    write!(file, "{}", format!(r#"# {} - Reassembly Mod

This is a mod project for the game Reassembly.

## Installation Instructions

1. Copy this folder to your Reassembly mods directory:
   - Windows: C:/Users/[YourName]/Saved Games/Reassembly/mods/
   - Mac: /Users/[YourName]/Library/Application Support/Reassembly/mods/
   - Linux: /home/[YourName]/.local/share/Reassembly/mods/

2. Start Reassembly. Your mod should appear in the Mods menu.

## Structure

- `shapes.lua`: Defines custom block shapes
- `shape_reference.lua`: Contains template shapes you can copy and modify
- `blocks.lua`: Defines custom blocks using both built-in and custom shapes
- `factions.lua`: Defines your custom faction
- `regions.lua`: Defines where your faction appears in the galaxy
- `ships/`: Contains ship designs for your faction
- `extra_ships/`: Contains extra ships that can be added without a full faction

## Development Guide

### Creating Custom Shapes

Use the Reassembly Shape Editor to create and edit shapes, then export them to shapes.lua.

Alternatively, you can modify the templates in `shape_reference.lua` which includes:
- Square (already in shapes.lua)
- Triangle
- Hexagon
- Octagon
- Specialized thruster shape
- Specialized weapon shape

Shape IDs should be in the range 100-10000.

### Creating Custom Blocks

Edit blocks.lua to create new blocks. Block IDs should be in the range 1-199 or 17000-26000.

### Creating Ships

Create ships in-game using your custom blocks, then:
1. Enter Sandbox mode (open console with ` and type 'sandbox')
2. Build your ship
3. Save it with the command 'ssave [shipname]'
4. Copy the ship file to the ships/ directory and rename to match your faction: '20_[shipname].lua'

### Preview Image

Create a preview.png image (less than 5MB) for your mod to display in the workshop.

## Publishing

Use the 'Publish' button in the Mods menu to upload your mod to the Steam Workshop.
"#, project_name))?;
    
    Ok(())
}

// Create a cvars.txt file with useful settings
fn create_cvars(project_dir: &Path) -> Result<(), io::Error> {
    let path = project_dir.join("cvars.txt");
    let mut file = fs::File::create(path)?;
    
    write!(file, "{}", r#"# Custom variables for your mod
# Uncomment and adjust as needed

# kWriteBlocks=1          # Set to 1 to generate blocks.lua file when game exits
# kExtraShipsFaction=20   # Change which faction is used for extra_ships
# kDefaultFontFile=font.ttf # Custom font file if included
"#)?;
    
    Ok(())
}

// Create a reminder for the preview image
fn create_preview_reminder(project_dir: &Path) -> Result<(), io::Error> {
    let path = project_dir.join("preview_placeholder.txt");
    let mut file = fs::File::create(path)?;
    
    write!(file, "{}", r#"To add a preview image for your mod:
1. Create an image showing your mod's content
2. Save it as "preview.png" in this directory
3. Make sure it's less than 5MB in size
4. Delete this placeholder file once you've added your preview image
"#)?;
    
    Ok(())
}

// Create a reference file with common shape patterns
fn create_shape_reference(project_dir: &Path) -> Result<(), io::Error> {
    let path = project_dir.join("shape_reference.lua");
    let mut file = fs::File::create(path)?;
    
    write!(file, "{}", r#"-- This file contains reference shapes that you can use as templates
-- Copy these into your shapes.lua file as needed and modify them
-- Note: Shape IDs should be in the range 100-10000

-- Triangle
{5002  --Triangle
    {
        {
            verts={
                {0, -5.77},   -- Bottom point (0, -10*sin(60°))
                {-5, 2.89},   -- Left point  (-10*cos(60°), 10*sin(30°))
                {5, 2.89},    -- Right point (10*cos(60°), 10*sin(30°))
            },
            ports={
                {0, 0.5},  -- Bottom edge, middle
                {1, 0.5},  -- Left edge, middle
                {2, 0.5},  -- Right edge, middle
            }
        }
    }
},

-- Hexagon
{5003  --Hexagon
    {
        {
            verts={
                {5, 0},        -- Right
                {2.5, 4.33},   -- Upper right
                {-2.5, 4.33},  -- Upper left
                {-5, 0},       -- Left
                {-2.5, -4.33}, -- Lower left
                {2.5, -4.33},  -- Lower right
            },
            ports={
                {0, 0.5},  -- Right edge, middle
                {1, 0.5},  -- Upper right edge, middle
                {2, 0.5},  -- Upper left edge, middle
                {3, 0.5},  -- Left edge, middle
                {4, 0.5},  -- Lower left edge, middle
                {5, 0.5},  -- Lower right edge, middle
            }
        }
    }
},

-- Octagon
{5004  --Octagon
    {
        {
            verts={
                {3.54, 3.54},   -- Upper right (5*cos(45°), 5*sin(45°))
                {0, 5},         -- Top
                {-3.54, 3.54},  -- Upper left
                {-5, 0},        -- Left
                {-3.54, -3.54}, -- Lower left
                {0, -5},        -- Bottom
                {3.54, -3.54},  -- Lower right
                {5, 0},         -- Right
            },
            ports={
                {0, 0.5},  -- Upper right edge, middle
                {1, 0.5},  -- Top edge, middle
                {2, 0.5},  -- Upper left edge, middle
                {3, 0.5},  -- Left edge, middle
                {4, 0.5},  -- Lower left edge, middle
                {5, 0.5},  -- Bottom edge, middle
                {6, 0.5},  -- Lower right edge, middle
                {7, 0.5},  -- Right edge, middle
            }
        }
    }
},

-- Specialized Thruster Shape
{5005  --Thruster
    {
        {
            verts={
                {-5, -5},     -- Bottom left
                {5, -5},      -- Bottom right
                {7, 0},       -- Middle right
                {5, 5},       -- Top right
                {-5, 5},      -- Top left
                {-7, 0},      -- Middle left
            },
            ports={
                {0, 0.5},             -- Bottom edge, middle
                {1, 0.5},             -- Bottom-right edge, middle
                {2, 0.5},             -- Right edge, middle
                {3, 0.5},             -- Top-right edge, middle
                {4, 0.5},             -- Top edge, middle
                {5, 0.5, THRUSTER_OUT} -- Left edge, middle with THRUSTER_OUT port type
            }
        }
    }
},

-- Weapon Shape with specialized ports
{5006  --Weapon
    {
        {
            verts={
                {-3, -5},    -- Bottom left
                {3, -5},     -- Bottom right
                {5, 0},      -- Middle right
                {3, 5},      -- Top right
                {-3, 5},     -- Top left
                {-5, 0},     -- Middle left
            },
            ports={
                {0, 0.5, WEAPON_IN},  -- Bottom edge, middle with WEAPON_IN port
                {1, 0.5},             -- Bottom-right edge, middle
                {2, 0.5, WEAPON_OUT}, -- Right edge, middle with WEAPON_OUT port
                {3, 0.5},             -- Top-right edge, middle
                {4, 0.5},             -- Top edge, middle
                {5, 0.5},             -- Left edge, middle
            }
        }
    }
}
"#)?;
    
    Ok(())
} 