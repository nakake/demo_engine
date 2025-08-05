use std::{collections::HashMap, sync::Arc};

use wgpu::util::DeviceExt;

use crate::core::error::{EngineError, EngineResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId(u64);

impl ResourceId {
    pub fn new(name: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);

        ResourceId(hasher.finish())
    }
}

pub struct ResourceManager {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    buffers: HashMap<ResourceId, Arc<wgpu::Buffer>>,
    pipelines: HashMap<ResourceId, Arc<wgpu::RenderPipeline>>,
    shaders: HashMap<ResourceId, Arc<wgpu::ShaderModule>>,
}

impl ResourceManager {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        ResourceManager {
            device,
            queue,
            buffers: HashMap::new(),
            pipelines: HashMap::new(),
            shaders: HashMap::new(),
        }
    }
    pub fn create_buffer_with_data(
        &mut self,
        id: ResourceId,
        data: &[u8],
        usage: wgpu::BufferUsages,
        label: Option<&str>,
    ) -> EngineResult<Arc<wgpu::Buffer>> {
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                usage,
                contents: data,
            });

        let arc_buffer = Arc::new(buffer);
        self.buffers.insert(id, arc_buffer.clone());

        Ok(arc_buffer)
    }

    pub fn create_shader(
        &mut self,
        id: ResourceId,
        source: &str,
        label: Option<&str>,
    ) -> EngineResult<Arc<wgpu::ShaderModule>> {
        let shader = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label,
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });

        let shader = Arc::new(shader);
        self.shaders.insert(id, shader.clone());

        Ok(shader)
    }

    pub fn create_pipeline(
        &mut self,
        id: ResourceId,
        shader_id: ResourceId,
        vertex_layout: wgpu::VertexBufferLayout,
        surface_format: wgpu::TextureFormat,
    ) -> EngineResult<Arc<wgpu::RenderPipeline>> {
        let shader = self.shaders.get(&shader_id)
            .ok_or_else(|| EngineError::ResourceNotFound(format!("Shader not found: {:?}", shader_id)))?;

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let pipeline = self
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: Some("vs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[vertex_layout],
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader,
                    entry_point: Some("fs_main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

        let pipeline = Arc::new(pipeline);
        self.pipelines.insert(id, pipeline.clone());
        Ok(pipeline)
    }

    pub fn get_buffer(&self, id: &ResourceId) -> Option<Arc<wgpu::Buffer>> {
        self.buffers.get(id).cloned()
    }

    pub fn get_pipeline(&self, id: &ResourceId) -> Option<Arc<wgpu::RenderPipeline>> {
        self.pipelines.get(id).cloned()
    }

    pub fn get_shader(&self, id: &ResourceId) -> Option<Arc<wgpu::ShaderModule>> {
        self.shaders.get(id).cloned()
    }
}
