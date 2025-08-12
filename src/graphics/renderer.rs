use std::sync::Arc;

use crate::{core::error::EngineResult, resources::manager::ResourceManager, scene::Scene};

pub struct Renderer {
    device: Arc<wgpu::Device>,
    clear_color: [f32; 4],
}

impl Renderer {
    pub fn new(device: Arc<wgpu::Device>, clear_color: [f32; 4]) -> Self {
        Self {
            device,
            clear_color,
        }
    }

    pub fn render_scene(
        &self,
        surface_view: &wgpu::TextureView,
        scene: &dyn Scene,
        resource_manager: &ResourceManager,
    ) -> EngineResult<wgpu::CommandBuffer> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = self.create_render_pass(&mut encoder, surface_view);

            if let Some(camera_bind_group) = scene.get_camera_bind_group() {
                render_pass.set_bind_group(0, camera_bind_group.as_ref(), &[]);
            }

            for object in scene.get_render_objects() {
                if let (Some(pipeline), Some(mesh)) = (
                    resource_manager.get_pipeline(&object.pipeline_id),
                    resource_manager.get_mesh(&object.mesh_id),
                ) {
                    render_pass.set_pipeline(&pipeline);
                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));

                    if let Some(index_buffer) = &mesh.index_buffer {
                        render_pass
                            .set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                        render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                    } else {
                        render_pass.draw(0..mesh.vertex_count, 0..1);
                    }
                }
            }
        }

        Ok(encoder.finish())
    }

    fn create_render_pass<'a>(
        &self,
        encoder: &'a mut wgpu::CommandEncoder,
        view: &'a wgpu::TextureView,
    ) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: self.clear_color[0] as f64,
                        g: self.clear_color[1] as f64,
                        b: self.clear_color[2] as f64,
                        a: self.clear_color[3] as f64,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        })
    }
}
