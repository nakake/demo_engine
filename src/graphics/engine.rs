use std::sync::Arc;

use crate::{
    core::{
        config::RenderingConfig,
        error::{EngineError, EngineResult},
        metrics::EngineMetrics,
    },
    graphics::{renderer::Renderer, surface_manager::SurfaceManager},
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
    scene: Box<dyn Scene>,
    config: RenderingConfig,
    metrics: EngineMetrics,
    surface_manager: SurfaceManager,
    renderer: Renderer,
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
    pub async fn new(
        window: Window,
        mut scene: Box<dyn Scene>,
        config: &RenderingConfig,
    ) -> EngineResult<Self> {
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

        let surface_manager = SurfaceManager::new(&instance, &window, &adapter, &device, config)?;

        let device = Arc::new(device);

        let queue: Arc<wgpu::Queue> = Arc::new(queue);

        let renderer = Renderer::new(device.clone(), config.clear_color);

        let mut resource_manager =
            ResourceManager::new(device.clone(), queue.clone(), surface_manager.format());

        // シーンを初期化
        scene.initialize(&mut resource_manager);

        let metrics = EngineMetrics::new();

        Ok(GraphicsEngine {
            resource_manager,
            device,
            queue,
            scene,
            config: config.clone(),
            metrics,
            surface_manager,
            renderer,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_manager.resize(&self.device, width, height);
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
        self.metrics
            .update(dt, self.scene.get_render_objects().len());
        self.metrics.check_performance();

        // シーン更新
        log::debug!("GraphicsEngine::render called with dt={}", dt);
        self.scene.update(dt, input);

        // カメラユニフォーム更新（毎フレーム）
        self.scene.update_camera_uniform();
        if let Some(camera_buffer) = self.scene.get_camera_buffer() {
            self.resource_manager
                .update_uniform_buffer(camera_buffer.as_ref(), self.scene.get_camera_uniform());
        }

        let surface_frame = self.surface_manager.acquire_frame()?;

        let command_buffer = self.renderer.render_scene(
            &surface_frame.view,
            self.scene.as_ref(),
            &self.resource_manager,
        )?;

        self.queue.submit(std::iter::once(command_buffer));
        surface_frame.present();
        Ok(())
    }
}
