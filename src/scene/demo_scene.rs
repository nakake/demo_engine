use std::sync::Arc;

use crate::{
    core::config::{AppConfig, MovementConfig},
    input::InputState,
    resources::{
        manager::{ResourceId, ResourceManager},
        primitives::{ObjectType, Primitive, quad::Quad, triangle::Triangle},
        uniforms::CameraUniform,
        vertex::{ColorVertex, VertexTrait},
    },
    scene::{
        Scene,
        camera::Camera,
        render_object::{ObjectId, RenderObject},
        transform::Transform,
    },
};

pub struct DemoScene {
    render_objects: Vec<RenderObject>,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: Option<Arc<wgpu::Buffer>>,
    camera_bind_group: Option<Arc<wgpu::BindGroup>>,
    initialized: bool,
    config: MovementConfig,
    resource_manager: Option<ResourceManager>,
    pipeline_id: ResourceId,
}

impl DemoScene {
    pub fn new(aspect: f32, config: Arc<AppConfig>) -> Self {
        Self {
            render_objects: Vec::new(),
            camera: Camera::new(aspect, &config.camera),
            camera_uniform: CameraUniform::new(),
            camera_buffer: None,
            camera_bind_group: None,
            initialized: false,
            config: config.movement.clone(),
            resource_manager: None,
            pipeline_id: ResourceId::new("basic_pipeline"),
        }
    }

    fn add_quad(&mut self, position: glam::Vec3) -> ObjectId {
        let quad_mesh = Quad::create_mesh(self.get_resource_manager_mut().get_device());

        let mesh_id = ResourceId::new(&format!("quad_mesh_{}", self.render_objects.len()));
        self.get_resource_manager_mut()
            .register_mesh(mesh_id, Arc::new(quad_mesh));

        let transform = Transform::new().with_position(position);
        let render_object = RenderObject::new(mesh_id, self.pipeline_id).with_transform(transform);
        let render_object_id = render_object.id;
        self.render_objects.push(render_object);

        render_object_id
    }

    fn add_triangle(&mut self, position: glam::Vec3) -> ObjectId {
        let triangle_mesh = Triangle::create_mesh(self.get_resource_manager_mut().get_device());

        let mesh_id = ResourceId::new(&format!("triangle_mesh_{}", self.render_objects.len()));
        self.get_resource_manager_mut()
            .register_mesh(mesh_id, Arc::new(triangle_mesh));

        let transform = Transform::new().with_position(position);
        let render_object = RenderObject::new(mesh_id, self.pipeline_id).with_transform(transform);
        let render_object_id = render_object.id;
        self.render_objects.push(render_object);

        render_object_id
    }

    fn get_resource_manager_mut(&mut self) -> &mut ResourceManager {
        self.resource_manager
            .as_mut()
            .expect("Scene not initialized")
    }
}

impl Scene for DemoScene {
    fn initialize(&mut self, resource_manager: ResourceManager) {
        if self.initialized {
            return;
        }

        self.resource_manager = Some(resource_manager);

        let shader_id = ResourceId::new("basic_shader");
        if let Err(e) = self.get_resource_manager_mut().create_shader(
            shader_id,
            include_str!("../../assets/shaders/basic/triangle.wgsl"),
            Some("Basic Shader"),
        ) {
            log::error!("Failed to create shader: {}", e);
            return;
        };

        let bind_group_layout = self
            .get_resource_manager_mut()
            .get_device()
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            });

        let pipeline_id = self.pipeline_id;
        let surface_format = self.get_resource_manager_mut().get_surface_format();

        if let Err(e) = self.get_resource_manager_mut().create_pipeline(
            pipeline_id,
            shader_id,
            ColorVertex::desc(),
            surface_format,
            &[&bind_group_layout],
        ) {
            log::error!("Failed to create pipeline: {}", e);
            return;
        };

        // カメラユニフォームバッファ作成
        self.camera_uniform.update_view_proj(&self.camera);
        let camera_buffer_id = ResourceId::new("camera_buffer");

        let camera_uniform = self.camera_uniform;
        let camera_buffer = self
            .get_resource_manager_mut()
            .create_uniform_buffer(camera_buffer_id, &camera_uniform)
            .expect("Failed to create camera buffer");
        self.camera_buffer = Some(camera_buffer.clone());

        // BindGroup作成
        let bind_group_id = ResourceId::new("camera_bind_group");
        let camera_bind_group = self
            .get_resource_manager_mut()
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

    fn get_resource_manager(&self) -> &ResourceManager {
        self.resource_manager
            .as_ref()
            .expect("Scene not initialized")
    }

    fn add_object(
        &mut self,
        object_type: crate::resources::primitives::ObjectType,
        position: glam::Vec3,
    ) -> ObjectId {
        match object_type {
            ObjectType::Quad => self.add_quad(position),
            ObjectType::Triangle => self.add_triangle(position),
        }
    }

    fn move_object(&mut self, object_id: ObjectId, position: glam::Vec3) -> bool {
        if let Some(obj) = self
            .render_objects
            .iter_mut()
            .find(|obj| obj.id == object_id)
        {
            obj.transform.set_position(position);
            true
        } else {
            false
        }
    }

    fn remove_object(&mut self, object_id: ObjectId) -> bool {
        let before_len = self.render_objects.len();
        self.render_objects.retain(|obj| obj.id != object_id);
        self.render_objects.len() < before_len
    }

    fn set_object_visible(&mut self, object_id: ObjectId, visible: bool) -> bool {
        if let Some(obj) = self
            .render_objects
            .iter_mut()
            .find(|obj| obj.id == object_id)
        {
            obj.set_visible(visible);
            true
        } else {
            false
        }
    }

    fn update_camera_uniform(&mut self) {
        self.camera_uniform.update_view_proj(&self.camera);

        if let (Some(camera_buffer), Some(resource_manager)) =
            (self.camera_buffer.as_ref(), self.resource_manager.as_mut())
        {
            resource_manager.update_uniform_buffer(camera_buffer.as_ref(), &self.camera_uniform);
        }
    }

    fn update(&mut self, dt: f32, input: &InputState) {
        use winit::keyboard::KeyCode;

        log::debug!("DemoScene::update called with dt={}", dt);

        let move_speed = self.config.move_speed * dt;
        let rotation_speed = self.config.rotation_speed * dt;

        // WASD でカメラ移動
        if input.is_key_pressed(KeyCode::KeyW) {
            log::debug!("W key pressed! Moving forward by {}", move_speed);
            log::debug!("Camera position before: {:?}", self.camera.eye);
            self.camera.move_forward(move_speed);
            log::debug!("Camera position after: {:?}", self.camera.eye);
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
