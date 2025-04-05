# Reassembly Shape Editor

A tool for creating and editing custom shapes for the game [Reassembly](https://store.steampowered.com/app/329130/Reassembly/).

## Features

- Visual shape editor with grid snapping
- Edit vertices and port positions
- Support for different port types (THRUSTER_IN, THRUSTER_OUT, etc.)
- Multiple scales per shape
- Import and export shapes.lua files
- Project generator for creating new Reassembly mods
- WebAssembly support for running in browsers

## Usage

### Shape Editor

Run the application normally to open the shape editor:

```
cargo run
```

### Project Generator

To generate a new Reassembly mod project structure:

```
cargo run -- --generate-project [project_name]
```

If no project name is provided, it will create a directory called "reassembly_mod" with the following structure:

```
reassembly_mod/
├── shapes.lua           # Sample custom shape
├── shape_reference.lua  # Template shapes for reference
├── blocks.lua           # Sample custom block
├── factions.lua         # Custom faction template
├── regions.lua          # Region template for placement in galaxy
├── cvars.txt            # Configuration variables
├── README.md            # Instructions
├── preview_placeholder.txt  # Reminder to create preview image
├── ships/               # Directory for faction ships
│   └── 20_starter.lua   # Placeholder starter ship
└── extra_ships/         # Directory for additional ships
```

This provides everything you need to start creating a Reassembly mod.

## Building From Source

1. Install Rust and Cargo: https://www.rust-lang.org/tools/install
2. Clone this repository
3. Build and run the project:

```
cargo build --release
cargo run --release
```

## WebAssembly Support

You can build and run the shape editor in a web browser using WebAssembly:

### Prerequisites for WASM build

1. Install wasm-pack: `cargo install wasm-pack`
2. Install a simple HTTP server (like Python's `http.server` module)

### Building for WASM

On Windows:
```
build_wasm.bat
```

On Linux/macOS:
```
./build_wasm.sh
```

### Running the Web Version

1. Start a local HTTP server in the web directory:
```
cd web
python -m http.server
```

2. Open your browser and navigate to http://localhost:8000

The shape editor should now be running in your browser!

## License

This project is open source.

## Acknowledgements

- Based on the Reassembly modding documentation by Arthur Danskin
- Uses the `nom` crate for parsing Lua shape files
- Built with egui for the user interface

## Introduction

This editor allows you to create, edit, and export custom block shapes for Reassembly mods in the `.lua` format. With this tool, you can:
- Create new shapes with vertices and connection ports
- Import existing shapes from `shapes.lua` files
- Edit and modify shapes with a visual interface
- Export your shapes to Lua format for use in Reassembly mods

## Getting Started

1. Launch the application
2. Use the grid and zoom controls at the top to adjust your view
3. Create a new shape or import existing shapes

## Creating Shapes

1. Click "Новая форма" (New Shape) in the top panel to create a new shape
2. Click on the canvas to add vertices
3. The shape will automatically form by connecting these vertices
4. The first vertex is highlighted in gold

## Managing Shapes

- In the left panel, you'll see a list of all shapes
- Click on a shape to select it for editing
- Each shape has an ID and a name that you can edit

## Editing Vertices

1. Select a vertex by clicking on it
2. Drag to move a selected vertex
3. Use the controls in the side panel to modify vertex coordinates
4. Click the "X" button to delete a vertex

## Working with Ports

Ports are connection points on the edges of your shape:

1. In the "Порты" (Ports) section of the side panel, click "Добавить порт" (Add Port)
2. Set the edge number (0-3, counting from the bottom edge clockwise)
3. Set the position (0.0-1.0, normalized along the edge)
4. Select a port type:
   - DEFAULT: Standard connection
   - THRUSTER_IN: Thruster input
   - THRUSTER_OUT: Thruster output
   - WEAPON_IN: Weapon input
   - WEAPON_OUT: Weapon output
   - MISSILE: Missile connection
   - LAUNCHER: Launcher connection
   - ROOT: Root connection
   - NONE: No special properties

## Importing and Exporting

### Importing

1. Set the import file path in the top panel (default is `shapes.lua`)
2. Click "Импорт" to import shapes from the specified file
3. Or click "Импорт shapes.lua" for the default file

### Exporting

1. Set the export file path in the top panel (default is `shapes.lua`)
2. Click "Экспорт" to export shapes to the specified file
3. Or click "Экспорт shapes.lua" for the default file

## Lua File Format

The `shapes.lua` file format follows this structure:

```lua
{
  {101,  --Shape_Name
    {
      {
        verts={
          {5, 5},
          {5, -5},
          {-5, -5},
          {-5, 5}
        },
        ports={
          {0, 0.5},
          {1, 0.5, THRUSTER_IN},
          {2, 0.5, THRUSTER_OUT},
          {3, 0.5}
        }
      }
    }
  }
}
```

In this structure:
- Each shape starts with an ID (e.g., `101`)
- After the ID, you can add a comment with the shape name
- The shape definition includes vertices (`verts`) and ports
- Vertices are defined by x,y coordinates
- Ports are defined by: `{edge_number, position_on_edge, [optional_type]}`

## Tips

1. Enable "Snap to Grid" for more precise vertex placement
2. The shape area is displayed in the upper right corner of the shape
3. Use Ctrl+Z to undo and Ctrl+Y to redo actions
4. Use the middle mouse button to pan the view and the mouse wheel to zoom

## Using Shapes in Reassembly

After exporting your shapes, use them in your Reassembly mod by referencing the shape ID in your block definition:

```lua
{
  {
    1, -- Block ID
    group=40, -- Your group
    name="My Custom Block",
    shape=101, -- This is the shape ID we defined
    fillColor=0x4b3d47,
    durability=1,
    growRate=1,
  },
}
```