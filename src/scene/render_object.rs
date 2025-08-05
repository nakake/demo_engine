use crate::resources::manager::ResourceId;

pub struct RenderObject {
    pub mesh_id: ResourceId,
    pub pipeline_id: ResourceId,
    pub transform: glam::Mat4,
}

impl RenderObject {
    pub fn new(mesh_id: ResourceId, pipeline_id: ResourceId) -> Self {
        Self {
            mesh_id,
            pipeline_id,
            transform: glam::Mat4::IDENTITY,
        }
    }
}
