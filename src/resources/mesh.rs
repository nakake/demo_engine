use std::sync::Arc;

use wgpu::util::DeviceExt;

pub struct Mesh {
    pub vertex_buffer: Arc<wgpu::Buffer>,
    pub index_buffer: Option<Arc<wgpu::Buffer>>,
    pub vertex_count: u32,
    pub index_count: u32,
}

impl Mesh {
    pub fn new<V: bytemuck::Pod>(
        device: Arc<wgpu::Device>,
        vertices: &[V],
        indices: Option<&[u16]>,
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let (index_buffer, index_count) = if let Some(indices) = indices {
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            (Some(Arc::new(buffer)), indices.len() as u32)
        } else {
            (None, 0)
        };

        Self {
            vertex_buffer: Arc::new(vertex_buffer),
            index_buffer,
            vertex_count: vertices.len() as u32,
            index_count,
        }
    }
}
