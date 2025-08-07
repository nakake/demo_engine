pub mod quad;
pub mod triangle;

use std::sync::Arc;

use crate::resources::mesh::Mesh;

pub trait Primitive {
    type Vertex: bytemuck::Pod;

    fn create_vertices() -> Vec<Self::Vertex>;
    fn create_indices() -> Option<Vec<u16>>;

    fn create_mesh(device: Arc<wgpu::Device>) -> Mesh {
        let vertices = Self::create_vertices();
        let indices = Self::create_indices();

        Mesh::new(device, &vertices, indices.as_deref())
    }
}
