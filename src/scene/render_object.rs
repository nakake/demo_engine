use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use crate::{
    resources::{manager::ResourceId, uniforms::ModelUniform},
    scene::transform::Transform,
};

static NEXT_OBJECT_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(u32);

impl ObjectId {
    pub fn generate() -> Self {
        Self(NEXT_OBJECT_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub fn as_u32(self) -> u32 {
        self.0
    }
}
pub struct RenderObject {
    pub mesh_id: ResourceId,
    pub pipeline_id: ResourceId,
    pub transform: Transform,
    pub visible: bool,
    pub id: ObjectId,
    pub model_buffer: Option<Arc<wgpu::Buffer>>,
    pub model_bind_group: Option<Arc<wgpu::BindGroup>>,
}

impl RenderObject {
    pub fn new(mesh_id: ResourceId, pipeline_id: ResourceId) -> Self {
        Self {
            mesh_id,
            pipeline_id,
            transform: Transform::new(),
            visible: true,
            id: ObjectId::generate(),
            model_buffer: None,
            model_bind_group: None,
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    pub fn get_model_matrix(&self) -> glam::Mat4 {
        self.transform.matrix()
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible
    }

    pub fn get_model_uniform_data(&self) -> ModelUniform {
        ModelUniform {
            model: self.transform.matrix().to_cols_array_2d(),
        }
    }
}
