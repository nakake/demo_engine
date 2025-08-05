# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based graphics engine demo built with WGPU and Winit. The project demonstrates 3D graphics rendering with a modular architecture focused on resource management, scene management, and error handling. The engine now supports mesh-based rendering with a scene system for managing render objects.

## Build and Development Commands

```bash
# Build the project
cargo build

# Run the demo
cargo run

# Build with optimizations (release mode)
cargo build --release
cargo run --release

# Check for compilation errors without building
cargo check

# Run tests
cargo test

# Generate documentation
cargo doc --open

# Clean build artifacts
cargo clean
```

## Architecture Overview

The engine follows a modular architecture with clear separation of concerns:

### Core Modules

- **`app/`** - Application lifecycle management using Winit's ApplicationHandler
  - `App` struct handles window events, rendering, and engine coordination
  - Manages window creation with 800x600 default size and "Demo Engine" title
  - Initializes default scene with triangle mesh

- **`core/`** - Core engine types and error handling
  - `EngineError` enum with comprehensive error types (English messages)
  - `EngineResult<T>` type alias for consistent error handling
  - All unwrap() calls have been replaced with proper error handling

- **`graphics/`** - WGPU-based rendering engine
  - `GraficsEngine` manages WGPU device, queue, surface configuration
  - Handles render loop, surface resizing, and frame presentation
  - Integrates with scene system for rendering multiple objects
  - `initial_default_scene()` sets up basic triangle demo

- **`resources/`** - Resource management system
  - `ResourceManager` provides centralized management of GPU resources
  - `ResourceId` uses string hashing for resource identification
  - Manages buffers, shaders, render pipelines, and meshes with Arc<T> for sharing
  - `VertexTrait` defines vertex buffer layout interface
  - Includes `ColorVertex` and `Vertex` (PBR-ready) implementations
  - `Mesh` struct for vertex/index buffer management
  - Primitive generation (Triangle) for basic shapes

- **`secen/`** - Scene management system (Note: typo in module name)
  - `Secen` struct manages collections of render objects
  - `RenderObject` links mesh and pipeline for rendering
  - Supports adding/removing objects from scene

- **`window/`** - Window abstraction layer
  - Wraps Winit window in custom `Window` struct with Arc for sharing

### Key Dependencies

- `wgpu` (26.0.1) - Modern graphics API abstraction
- `winit` (0.30.12) - Cross-platform windowing
- `glam` (0.30.5) - Math library for 3D graphics
- `bytemuck` - Safe transmutation for vertex data
- `pollster` - Async runtime for WGPU initialization

### Resource System

The engine uses a hash-based resource identification system:
- Resources are identified by `ResourceId` generated from string names
- `ResourceManager` stores resources in HashMaps with Arc wrappers for sharing
- Supports: vertex buffers, index buffers, shaders (WGSL), render pipelines, and meshes
- `register_mesh()` automatically manages vertex/index buffer registration
- Primitive generation system for creating basic geometric shapes

### Rendering Pipeline

1. WGPU instance/adapter/device initialization in `GraficsEngine::new()`
2. Surface configuration with automatic format selection (prefers sRGB)
3. Scene initialization with `initial_default_scene()`
4. Shader compilation from WGSL files in `assets/shaders/`  
5. Render pipeline creation with vertex layout binding
6. Frame rendering iterates through scene objects, binding pipelines and meshes
7. Supports both indexed and non-indexed mesh rendering

### Scene System

- Scene objects are managed by the `Secen` struct
- Each `RenderObject` links a mesh ID with a pipeline ID
- Rendering loop iterates through all scene objects
- Automatic vertex/index buffer binding based on mesh type

### Error Handling

- Comprehensive error system with `EngineError` enum
- All error messages in English
- No unwrap() calls - all errors properly handled
- Graceful degradation when resources are missing

### Current Demo

The current implementation renders a single colored triangle:
- Generated using Triangle primitive with position and color attributes
- Basic WGSL shader with vertex and fragment stages
- Scene-based rendering system with proper resource management
- Clear color: RGB(0.5, 0.2, 0.2) - reddish brown background

## File Structure Notes

- Shaders are stored in `assets/shaders/basic/` and embedded at compile time
- Vertex types implement `VertexTrait` for WGPU buffer layout generation
- The `Vertex` struct is prepared for PBR rendering with normal, UV, and tangent data
- Module structure uses standard Rust conventions with `mod.rs` files
- Note: `secen` module has a typo in the name (should be `scene`)
- Primitive shapes are generated in `resources/primitives/` for basic geometry
- All async operations use `pollster::block_on` for simplicity

## Development Notes

- The engine prioritizes proper error handling over unwrap() calls
- Resource management uses Arc for safe sharing between components  
- Scene system allows for easy addition/removal of render objects
- The architecture supports extending with more complex rendering features
- Current focus is on basic triangle rendering with plans for more primitives