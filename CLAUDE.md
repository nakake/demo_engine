# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based graphics engine demo built with WGPU and Winit. The project demonstrates basic 3D graphics rendering with a modular architecture focused on resource management and graphics abstraction.

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

- **`graphics/`** - WGPU-based rendering engine
  - `GraficsEngine` manages WGPU device, queue, surface configuration
  - Handles render loop, surface resizing, and frame presentation
  - Currently renders a colored triangle as proof-of-concept

- **`resources/`** - Resource management system
  - `ResourceManager` provides centralized management of GPU resources
  - `ResourceId` uses string hashing for resource identification
  - Manages buffers, shaders, and render pipelines with Arc<T> for shared ownership
  - `VertexTrait` defines vertex buffer layout interface
  - Includes `ColorVertex` and `Vertex` (PBR-ready) implementations

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
- Currently supports: vertex buffers, shaders (WGSL), and render pipelines

### Rendering Pipeline

1. WGPU instance/adapter/device initialization in `GraficsEngine::new()`
2. Surface configuration with automatic format selection (prefers sRGB)
3. Shader compilation from WGSL files in `assets/shaders/`  
4. Render pipeline creation with vertex layout binding
5. Frame rendering via command encoder and render pass

### Current Demo

The current implementation renders a single colored triangle:
- Vertices defined with position and color attributes
- Basic WGSL shader with vertex and fragment stages
- Clear color: RGB(0.5, 0.2, 0.2) - reddish brown background

## File Structure Notes

- Shaders are stored in `assets/shaders/basic/` and embedded at compile time
- Vertex types implement `VertexTrait` for WGPU buffer layout generation
- The `Vertex` struct is prepared for PBR rendering with normal, UV, and tangent data
- Module structure uses standard Rust conventions with `mod.rs` files