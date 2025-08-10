use std::sync::Arc;

use crate::{
    core::config::{AppConfig, MovementConfig},
    input::InputState,
    resources::{
        manager::{ResourceId, ResourceManager},
        primitives::{Primitive, quad::Quad},
        uniforms::CameraUniform,
        vertex::{ColorVertex, VertexTrait},
    },
    scene::{Scene, camera::Camera, render_object::RenderObject},
};

pub struct DemoScene {
    render_objects: Vec<RenderObject>,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: Option<Arc<wgpu::Buffer>>,
    camera_bind_group: Option<Arc<wgpu::BindGroup>>,
    initialized: bool,
    config: MovementConfig,
}

impl DemoScene {
    pub fn new(aspect: f32, config: &AppConfig) -> Self {
        Self {
            render_objects: Vec::new(),
            camera: Camera::new(aspect, &config.camera),
            camera_uniform: CameraUniform::new(),
            camera_buffer: None,
            camera_bind_group: None,
            initialized: false,
            config: config.movement.clone(),
        }
    }
}

impl Scene for DemoScene {
    fn initialize(&mut self, resource_manager: &mut ResourceManager) {
        if self.initialized {
            return;
        }

        let shader_id = ResourceId::new("basic_shader");
        if let Err(e) = resource_manager.create_shader(
            shader_id,
            include_str!("../../assets/shaders/basic/triangle.wgsl"),
            Some("Basic Shader"),
        ) {
            eprintln!("Failed to create shader: {}", e);
            return;
        };

        let bind_group_layout = resource_manager.get_device().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            },
        );

        let pipeline_id = ResourceId::new("basic_pipeline");
        if let Err(e) = resource_manager.create_pipeline(
            pipeline_id,
            shader_id,
            ColorVertex::desc(),
            resource_manager.get_surface_format(),
            &[&bind_group_layout],
        ) {
            eprintln!("Failed to create pipeline: {}", e);
            return;
        };

        // カメラユニフォームバッファ作成
        self.camera_uniform.update_view_proj(&self.camera);
        let camera_buffer_id = ResourceId::new("camera_buffer");
        let camera_buffer = resource_manager
            .create_uniform_buffer(camera_buffer_id, &self.camera_uniform)
            .expect("Failed to create camera buffer");
        self.camera_buffer = Some(camera_buffer.clone());

        // BindGroup作成
        let bind_group_id = ResourceId::new("camera_bind_group");
        let camera_bind_group = resource_manager
            .create_bind_group(
                bind_group_id,
                &bind_group_layout,
                &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
            )
            .expect("Failed to create camera bind group");
        self.camera_bind_group = Some(camera_bind_group);

        let quad_mesh = Quad::create_mesh(resource_manager.get_device());

        let mesh_id = ResourceId::new("basic_mesh");
        resource_manager.register_mesh(mesh_id, Arc::new(quad_mesh));

        let render_object = RenderObject::new(mesh_id, pipeline_id);
        self.render_objects.push(render_object);

        self.initialized = true;
    }

    fn get_render_objects(&self) -> &[RenderObject] {
        &self.render_objects
    }

    fn get_camera_bind_group(&self) -> Option<&Arc<wgpu::BindGroup>> {
        self.camera_bind_group.as_ref()
    }

    fn get_camera_buffer(&self) -> Option<&Arc<wgpu::Buffer>> {
        self.camera_buffer.as_ref()
    }

    fn get_camera_uniform(&self) -> &CameraUniform {
        &self.camera_uniform
    }

    fn update_camera_uniform(&mut self) {
        self.camera_uniform.update_view_proj(&self.camera);
    }

    fn update(&mut self, dt: f32, input: &InputState) {
        use winit::keyboard::KeyCode;

        println!("DemoScene::update called with dt={}", dt);

        let move_speed = self.config.move_speed * dt;
        let rotation_speed = self.config.rotation_speed * dt;

        // WASD でカメラ移動
        if input.is_key_pressed(KeyCode::KeyW) {
            println!("W key pressed! Moving forward by {}", move_speed);
            println!("Camera position before: {:?}", self.camera.eye);
            self.camera.move_forward(move_speed);
            println!("Camera position after: {:?}", self.camera.eye);
        }
        if input.is_key_pressed(KeyCode::KeyS) {
            self.camera.move_forward(-move_speed);
        }
        if input.is_key_pressed(KeyCode::KeyA) {
            self.camera.move_right(-move_speed);
        }
        if input.is_key_pressed(KeyCode::KeyD) {
            self.camera.move_right(move_speed);
        }

        // Q/E で上下移動
        if input.is_key_pressed(KeyCode::KeyQ) {
            self.camera.move_up(-move_speed);
        }
        if input.is_key_pressed(KeyCode::KeyE) {
            self.camera.move_up(move_speed);
        }

        // 矢印キーで回転
        if input.is_key_pressed(KeyCode::ArrowLeft) {
            self.camera.rotate_horizontal(rotation_speed);
        }
        if input.is_key_pressed(KeyCode::ArrowRight) {
            self.camera.rotate_horizontal(-rotation_speed);
        }
        if input.is_key_pressed(KeyCode::ArrowUp) {
            self.camera.rotate_vertical(rotation_speed);
        }
        if input.is_key_pressed(KeyCode::ArrowDown) {
            self.camera.rotate_vertical(-rotation_speed);
        }
    }
}
