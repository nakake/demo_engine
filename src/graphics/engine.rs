use std::sync::Arc;

use crate::{
    core::error::{EngineError, EngineResult},
    resources::{
        manager::{ResourceId, ResourceManager},
        mesh::Mesh,
        primitives::{Primitive, triangle::Triangle},
        vertex::{ColorVertex, VertexTrait},
    },
    scene::{Scene, render_object::RenderObject},
    window::Window,
};

pub struct GraficsEngine {
    resource_manager: ResourceManager,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    secen: Scene,
}

impl GraficsEngine {
    pub async fn new(window: Window) -> EngineResult<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| {
                EngineError::AdapterRequest(format!("Failed to request adapter: {}", e))
            })?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::default(),
            })
            .await
            .map_err(|e| EngineError::DeviceRequest(format!("Failed to request device: {}", e)))?;

        let winit_window = window.get_window();

        let surface = instance.create_surface(winit_window.clone()).map_err(|e| {
            EngineError::SurfaceCreation(format!("Failed to create surface: {}", e))
        })?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: winit_window.inner_size().width,
            height: winit_window.inner_size().height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let device = Arc::new(device);

        let queue: Arc<wgpu::Queue> = Arc::new(queue);

        let resource_manager = ResourceManager::new(device.clone(), queue.clone());

        let secen = Scene::new();

        Ok(GraficsEngine {
            resource_manager,
            device,
            queue,
            surface,
            surface_config: config,
            secen,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn get_device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn get_queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn get_surface(&self) -> &wgpu::Surface<'static> {
        &self.surface
    }

    pub fn get_surface_config(&self) -> &wgpu::SurfaceConfiguration {
        &self.surface_config
    }

    pub fn render(&mut self) -> EngineResult<()> {
        let frame = self.surface.get_current_texture().map_err(|e| {
            EngineError::RenderError(format!("Failed to acquire next surface texture: {}", e))
        })?;

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.5,
                            g: 0.2,
                            b: 0.2,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            for object in self.secen.objects() {
                if let (Some(pipeline), Some(mesh)) = (
                    self.resource_manager.get_pipeline(&object.pipeline_id),
                    self.resource_manager.get_mesh(&object.mesh_id),
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

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        Ok(())
    }

    pub fn initial_default_scene(&mut self) {
        let shader_id = ResourceId::new("basic_shader");
        let _ = self.resource_manager.create_shader(
            shader_id,
            include_str!("../../assets/shaders/basic/triangle.wgsl"),
            Some("Basic Shader"),
        );

        let pipeline_id = ResourceId::new("basic_pipeline");
        let _ = self.resource_manager.create_pipeline(
            pipeline_id,
            shader_id,
            ColorVertex::desc(),
            self.surface_config.format,
        );

        let triangle_mesh = Triangle::create_mesh(self.device.clone());
        let mesh_id = ResourceId::new("basic_triangle_mesh");
        self.resource_manager
            .register_mesh(mesh_id, Arc::new(triangle_mesh));

        let triangle_object = RenderObject::new(mesh_id, pipeline_id);
        self.secen.add_object(triangle_object);
    }
}
