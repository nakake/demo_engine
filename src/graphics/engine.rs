use std::sync::Arc;

use crate::{
    core::error::{EngineError, EngineResult},
    resources::{
        manager::{ResourceId, ResourceManager},
        vertex::{ColorVertex, VertexTrait},
    },
    window::window::Window,
};

pub struct GraficsEngine {
    resource_manager: ResourceManager,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
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
            .map_err(|e| EngineError::AdapterRequest(format!("Failed to request adapter: {}", e)))?;

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

        let surface = instance.create_surface(winit_window.clone())
            .map_err(|e| EngineError::SurfaceCreation(format!("Failed to create surface: {}", e)))?;

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

        let vertices = [
            ColorVertex {
                position: [0.0, 0.5, 0.0],
                color: [1.0, 0.0, 0.0],
            },
            ColorVertex {
                position: [-0.5, -0.5, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            ColorVertex {
                position: [0.5, -0.5, 0.0],
                color: [0.0, 0.0, 1.0],
            },
        ];

        let device = Arc::new(device);

        let queue: Arc<wgpu::Queue> = Arc::new(queue);

        let mut resource_manager = ResourceManager::new(device.clone(), queue.clone());

        let shader_id = ResourceId::new("triangle shader id");

        resource_manager.create_shader(
            shader_id,
            include_str!("../../assets/shaders/basic/triangle.wgsl"),
            Some("triangle shader"),
        )?;

        resource_manager.create_pipeline(
            ResourceId::new("triangle pipeline"),
            shader_id,
            ColorVertex::desc(),
            surface_format,
        )?;

        resource_manager.create_buffer_with_data(
            ResourceId::new("triangle buffer"),
            bytemuck::cast_slice(&vertices),
            wgpu::BufferUsages::VERTEX,
            Some("vertex buffer"),
        )?;

        Ok(GraficsEngine {
            resource_manager,
            device,
            queue,
            surface,
            surface_config: config,
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
        let frame = self
            .surface
            .get_current_texture()
            .map_err(|e| EngineError::RenderError(format!("Failed to acquire next surface texture: {}", e)))?;

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let pipeline = self
            .resource_manager
            .get_pipeline(&ResourceId::new("triangle pipeline"))
            .ok_or_else(|| EngineError::ResourceNotFound("triangle pipeline".to_string()))?;

        let vertex_buffer = self
            .resource_manager
            .get_buffer(&ResourceId::new("triangle buffer"))
            .ok_or_else(|| EngineError::ResourceNotFound("triangle buffer".to_string()))?;

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

            render_pass.set_pipeline(&pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
        Ok(())
    }
}
