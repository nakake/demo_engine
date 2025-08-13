use crate::{
    input::InputState,
    resources::{manager::ResourceManager, primitives::ObjectType},
    scene::render_object::{ObjectId, RenderObject},
};

pub mod camera;
pub mod demo_scene;
pub mod manager;
pub mod render_object;
pub mod transform;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SceneId(u64);

impl SceneId {
    pub fn new(name: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);

        SceneId(hasher.finish())
    }
}

/// Abstraction for 3D scenes containing renderable objects and cameras.
///
/// A Scene manages its own objects, camera, and rendering resources. It provides
/// the graphics engine with the necessary data for rendering while encapsulating
/// scene-specific logic and state management.
///
/// # Lifecycle
///
/// 1. `initialize()` - Set up GPU resources and objects
/// 2. `update()` - Handle input and animations each frame
/// 3. `update_camera_uniform()` - Sync camera data to GPU
/// 4. Rendering methods provide access to render data
pub trait Scene {
    /// Initialize scene resources using the provided resource manager.
    ///
    /// Creates meshes, shaders, pipelines, and other GPU resources needed
    /// for rendering this scene's objects.
    fn initialize(&mut self, resource_manager: ResourceManager);

    /// Returns the list of objects to be rendered in this scene.
    fn get_render_objects(&self) -> &[RenderObject];

    /// Returns the camera's bind group for shader uniform binding.
    fn get_camera_bind_group(&self) -> Option<&std::sync::Arc<wgpu::BindGroup>>;

    /// Returns the camera's uniform buffer for GPU data updates.
    fn get_camera_buffer(&self) -> Option<&std::sync::Arc<wgpu::Buffer>>;

    /// Returns the current camera uniform data.
    fn get_camera_uniform(&self) -> &crate::resources::uniforms::CameraUniform;

    /// Update scene state based on delta time and user input.
    ///
    /// # Arguments
    ///
    /// * `dt` - Time elapsed since last frame in seconds
    /// * `input` - Current input state (keyboard, mouse, etc.)
    fn update(&mut self, dt: f32, input: &InputState);

    /// Update camera uniform data from current camera state.
    ///
    /// Should be called after camera modifications to sync view/projection
    /// matrices with GPU uniform buffer.
    fn update_camera_uniform(&mut self);

    fn get_resource_manager(&self) -> &ResourceManager;
    fn add_object(&mut self, object_type: ObjectType, position: glam::Vec3) -> ObjectId;
    fn remove_object(&mut self, object_id: ObjectId) -> bool;
    fn move_object(&mut self, object_id: ObjectId, position: glam::Vec3) -> bool;
    fn set_object_visible(&mut self, object_id: ObjectId, visible: bool) -> bool;
}
