# Vertexify

# WORK IN PROGRESS (WIP)

A 3D modeling workspace for Linux optimized for integrated GPUs and non-GPU hardware. Combines a traditional GUI with a domain-specific language for procedural geometry generation.

## What It Does

Vertexify provides two ways to create 3D models:

1. **VX Script** - A custom DSL for procedural modeling
2. **GUI Interface** - Visual tools for modeling and scene management

The focus is on static modeling without animation overhead, targeting performance on systems with integrated graphics.

## How It Works

### VX Language

VX is a statically-typed scripting language designed for geometry operations. Variables are immutable by default, and operations return new geometry states.

```vx
let base = cube(width: 10.0, height: 2.0, depth: 10.0)
base.set_material(color: "#333333", roughness: 0.8)

let hole = cylinder(radius: 2.0, height: 4.0)
hole.translate(y: -1.0)

let part = difference(base, hole)
spawn(part)
```

### Architecture

- **Rust** for safe concurrency and predictable performance
- **glam** for SIMD-optimized math operations
- **Custom ECS** for scene management
- **Lazy CSG** - operations build a tree and evaluate on demand
- **Multi-threading** with rayon for vertex processing

### Performance Strategy

- SIMD optimization for all vertex transformations
- Parallel processing with rayon
- Cache-friendly data structures
- BVH for spatial queries and frustum culling
- Vulkan backend via wgpu for iGPU optimization

## Current Status

Phase 1 complete:
- Math library with transforms and bounding volumes
- Scene graph with entity-component system
- Mesh primitives (cube, sphere, cylinder, plane)
- VX lexer, parser, and interpreter
- Basic script execution

In progress:
- wgpu rendering pipeline
- egui interface
- Camera controls

## Building

Requires Rust 1.75 or newer:

```bash
cargo build --release
cargo run
```

## Project Structure

```
src/
├── math/          # Vector math, transforms, bounding volumes
├── scene/         # ECS, meshes, materials, entities
├── vx/            # VX language (lexer, parser, interpreter)
├── renderer/      # wgpu rendering (planned)
└── gui/           # egui interface (planned)
```

## Limitations

- No animation support
- CSG operations not yet implemented
- Rendering pipeline in development
- Linux only

## License

MIT
