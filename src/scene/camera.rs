/// 3D camera for view and projection matrix calculations.
/// 
/// Provides first-person camera controls with position, target-based rotation,
/// and perspective projection. Supports movement and rotation operations
/// commonly used in 3D applications.
/// 
/// # Fields
/// 
/// - `eye` - Camera position in world space
/// - `target` - Point the camera is looking at
/// - `up` - Camera's up direction vector (usually Y-axis)
/// - `aspect` - Aspect ratio (window_width / window_height)
/// - `fovy` - Field of view angle in radians
/// - `znear` - Near clipping plane distance (0.1 - 1.0 typical)
/// - `zfar` - Far clipping plane distance (100.0 - 10000.0 typical)
/// 
/// # Examples
/// 
/// ```rust
/// let mut camera = Camera::new(800.0 / 600.0);
/// camera.move_forward(1.0);
/// camera.rotate_horizontal(0.1);
/// let view_proj_matrix = camera.build_view_proj_matrix();
/// ```
pub struct Camera {
    pub eye: glam::Vec3,
    pub target: glam::Vec3,
    pub up: glam::Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            eye: glam::Vec3 {
                x: 0.0,
                y: 0.0,
                z: 3.0,
            },
            target: glam::Vec3::ZERO,
            up: glam::Vec3::Y,
            aspect,
            fovy: 45.0_f32.to_radians(),
            znear: 0.1,
            zfar: 100.0,
        }
    }

    pub fn build_view_proj_matrix(&self) -> glam::Mat4 {
        let veiw = glam::Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = glam::Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);

        proj * veiw
    }

    /// カメラを前後に移動
    pub fn move_forward(&mut self, delta: f32) {
        let forward = (self.target - self.eye).normalize();
        self.eye += forward * delta;
        self.target += forward * delta;
    }

    /// カメラを左右に移動
    pub fn move_right(&mut self, delta: f32) {
        let forward = (self.target - self.eye).normalize();
        let right = forward.cross(self.up).normalize();
        self.eye += right * delta;
        self.target += right * delta;
    }

    /// カメラを上下に移動
    pub fn move_up(&mut self, delta: f32) {
        self.eye += self.up * delta;
        self.target += self.up * delta;
    }

    /// カメラを回転（水平）
    pub fn rotate_horizontal(&mut self, angle: f32) {
        let rotation = glam::Mat3::from_rotation_y(angle);
        let direction = self.target - self.eye;
        let new_direction = rotation * direction;
        self.target = self.eye + new_direction;
    }

    /// カメラを回転（垂直）
    pub fn rotate_vertical(&mut self, angle: f32) {
        let forward = (self.target - self.eye).normalize();
        let right = forward.cross(self.up).normalize();
        let rotation = glam::Mat3::from_axis_angle(right, angle);
        let new_direction = rotation * forward;
        self.target = self.eye + new_direction;
    }
}
