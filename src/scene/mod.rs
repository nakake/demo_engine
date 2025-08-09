use crate::{
    input::InputState, resources::manager::ResourceManager, scene::render_object::RenderObject,
};

pub mod camera;
pub mod demo_scene;
pub mod manager;
pub mod render_object;

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

pub trait Scene {
    fn initialize(&mut self, resource_manager: &mut ResourceManager);
    fn get_render_objects(&self) -> &[RenderObject];
    fn get_camera_bind_group(&self) -> Option<&std::sync::Arc<wgpu::BindGroup>>;
    fn get_camera_buffer(&self) -> Option<&std::sync::Arc<wgpu::Buffer>>;
    fn get_camera_uniform(&self) -> &crate::resources::uniforms::CameraUniform;
    fn update(&mut self, dt: f32, input: &InputState);
    fn update_camera_uniform(&mut self);
}
