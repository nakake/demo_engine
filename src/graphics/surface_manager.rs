use crate::{
    core::{
        config::RenderingConfig,
        error::{EngineError, EngineResult},
    },
    window::Window,
};

pub struct SurfaceManager {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    format: wgpu::TextureFormat,
    caps: wgpu::SurfaceCapabilities,
}

impl SurfaceManager {
    pub fn new(
        instance: &wgpu::Instance,
        window: &Window,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        render_config: &RenderingConfig,
    ) -> EngineResult<Self> {
        let surface = instance
            .create_surface(window.get_window().clone())
            .map_err(|e| {
                EngineError::SurfaceCreation(format!("Failed to create surface: {}", e))
            })?;

        let caps = surface.get_capabilities(adapter);

        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: window.get_window().inner_size().width,
            height: window.get_window().inner_size().height,
            present_mode: if render_config.vsync {
                wgpu::PresentMode::Fifo
            } else {
                wgpu::PresentMode::Immediate
            },
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(device, &config);

        Ok(Self {
            surface,
            config,
            format,
            caps,
        })
    }

    /// Resizes the rendering surface to the specified dimensions.
    ///
    /// # Arguments
    ///
    /// * `width` - New width in pixels (ignored if 0)
    /// * `height` - New height in pixels (ignored if 0)
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(device, &self.config);
    }

    pub fn acquire_frame(&self) -> EngineResult<SurfaceFrame> {
        let texture = self.surface.get_current_texture().map_err(|e| {
            EngineError::RenderError(format!("Failed to acquire next surface texture: {}", e))
        })?;

        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        Ok(SurfaceFrame { texture, view })
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }
}

pub struct SurfaceFrame {
    pub texture: wgpu::SurfaceTexture,
    pub view: wgpu::TextureView,
}

impl SurfaceFrame {
    pub fn present(self) {
        self.texture.present();
    }
}
