use std::sync::Arc;

use crate::{
    core::error::{EngineError, EngineResult},
    resources::manager::ResourceManager,
    scene::Scene,
    window::Window,
};

/// WGPU-based 3D graphics rendering engine.
/// 
/// Manages GPU resources, handles scene rendering, and coordinates between
/// the graphics hardware and scene objects.
/// 
/// # Examples
/// 
/// ```rust
/// use demo_engine::graphics::GraphicsEngine;
/// use demo_engine::scene::DemoScene;
/// 
/// let scene = Box::new(DemoScene::new());
/// let engine = GraphicsEngine::new(window, scene).await?;
/// engine.render(dt, &input_state)?;
/// ```
pub struct GraphicsEngine {
    resource_manager: ResourceManager,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    scene: Box<dyn Scene>,
}

impl GraphicsEngine {
    /// Creates a new graphics engine with the specified window and scene.
    /// 
    /// Initializes WGPU resources including device, queue, surface, and configures
    /// the rendering pipeline for the given scene.
    /// 
    /// # Arguments
    /// 
    /// * `window` - The window to render to
    /// * `scene` - The scene to be rendered
    /// 
    /// # Returns
    /// 
    /// Returns a configured GraphicsEngine ready for rendering.
    /// 
    /// # Errors
    /// 
    /// Returns `EngineError` if WGPU initialization fails.
    pub async fn new(window: Window, mut scene: Box<dyn Scene>) -> EngineResult<Self> {
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

        let mut resource_manager =
            ResourceManager::new(device.clone(), queue.clone(), surface_format);

        // シーンを初期化
        scene.initialize(&mut resource_manager);

        Ok(GraphicsEngine {
            resource_manager,
            device,
            queue,
            surface,
            surface_config: config,
            scene,
        })
    }

    /// Resizes the rendering surface to the specified dimensions.
    /// 
    /// # Arguments
    /// 
    /// * `width` - New width in pixels (ignored if 0)
    /// * `height` - New height in pixels (ignored if 0)
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
    }

    /// Renders a single frame.
    /// 
    /// Updates the scene with delta time and input, then renders all scene objects
    /// to the surface. Also updates camera uniforms and handles GPU synchronization.
    /// 
    /// # Arguments
    /// 
    /// * `dt` - Delta time since last frame in seconds
    /// * `input` - Current input state for scene updates
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(())` on successful render, or `EngineError` if rendering fails.
    pub fn render(&mut self, dt: f32, input: &crate::input::InputState) -> EngineResult<()> {
        // シーン更新
        println!("GraphicsEngine::render called with dt={}", dt);
        self.scene.update(dt, input);
        let frame = self.surface.get_current_texture().map_err(|e| {
            EngineError::RenderError(format!("Failed to acquire next surface texture: {}", e))
        })?;

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // カメラユニフォーム更新（毎フレーム）
        self.scene.update_camera_uniform();
        if let Some(camera_buffer) = self.scene.get_camera_buffer() {
            self.resource_manager
                .update_uniform_buffer(camera_buffer.as_ref(), self.scene.get_camera_uniform());
        }

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

            // カメラバインドグループを一度だけ設定（全オブジェクト共通）
            if let Some(camera_bind_group) = self.scene.get_camera_bind_group() {
                render_pass.set_bind_group(0, camera_bind_group.as_ref(), &[]);
            }

            for object in self.scene.as_ref().get_render_objects() {
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
}
