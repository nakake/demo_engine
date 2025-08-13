pub struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: glam::vec3(0.0, 0.0, 0.0),
            rotation: glam::Quat::IDENTITY,
            scale: glam::vec3(1.0, 1.0, 1.0),
        }
    }

    pub fn with_position(mut self, position: glam::Vec3) -> Self {
        self.position = position;
        self
    }

    pub fn with_rotation(mut self, rotation: glam::Quat) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_scale(mut self, scale: glam::Vec3) -> Self {
        self.scale = scale;
        self
    }

    pub fn matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    pub fn forward(&self) -> glam::Vec3 {
        self.rotation * glam::Vec3::NEG_Z
    }

    pub fn right(&self) -> glam::Vec3 {
        self.rotation * glam::Vec3::X
    }

    pub fn up(&self) -> glam::Vec3 {
        self.rotation * glam::Vec3::Y
    }

    pub fn set_position(&mut self, position: glam::Vec3) {
        self.position = position;
    }
}
