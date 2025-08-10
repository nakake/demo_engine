use crate::core::config::CameraConfig;

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
    pub fn new(aspect: f32, config: &CameraConfig) -> Self {
        Self {
            eye: glam::Vec3 {
                x: 0.0,
                y: 0.0,
                z: 3.0,
            },
            target: glam::Vec3::ZERO,
            up: glam::Vec3::Y,
            aspect,
            fovy: config.fov_degrees.to_radians(),
            znear: config.znear,
            zfar: config.zfar,
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

#[cfg(test)]
mod tests {
    use crate::core::config::AppConfig;

    use super::*;

    #[test]
    fn test_camera_initialization() {
        let config = AppConfig::default();
        let camera = Camera::new(16.0 / 9.0, &config.camera);

        assert_eq!(camera.eye, glam::Vec3::new(0.0, 0.0, 3.0));
        assert_eq!(camera.target, glam::Vec3::ZERO);
        assert_eq!(camera.up, glam::Vec3::Y);
        assert_eq!(camera.aspect, 16.0 / 9.0);
    }

    #[test]
    fn test_camera_move_forward() {
        let config = AppConfig::default();
        let mut camera = Camera::new(1.0, &config.camera);
        let initial_eye = camera.eye;
        let initial_target = camera.target;

        camera.move_forward(1.0);

        // 前方向に移動したので位置が変わっているはず
        assert_ne!(camera.eye, initial_eye);
        assert_ne!(camera.target, initial_target);

        // eyeとtargetの相対位置（方向）は保持されるべき
        let initial_direction = initial_target - initial_eye;
        let new_direction = camera.target - camera.eye;

        // 方向ベクトルの長さは同じであるべき
        assert!((initial_direction.length() - new_direction.length()).abs() < f32::EPSILON);
    }

    #[test]
    fn test_camera_move_right() {
        let config = AppConfig::default();
        let mut camera = Camera::new(1.0, &config.camera);
        let initial_eye = camera.eye;

        camera.move_right(1.0);

        assert_ne!(camera.eye, initial_eye);
    }

    #[test]
    fn test_camera_move_up() {
        let config = AppConfig::default();
        let mut camera = Camera::new(1.0, &config.camera);
        let initial_eye = camera.eye;

        camera.move_up(1.0);

        // Y方向に移動したはず
        assert!(camera.eye.y > initial_eye.y);
    }

    #[test]
    fn test_camera_rotate_horizontal() {
        let config = AppConfig::default();
        let mut camera = Camera::new(1.0, &config.camera);
        let initial_target = camera.target;

        camera.rotate_horizontal(std::f32::consts::FRAC_PI_2); // 90度回転

        // targetが変わったはず
        assert_ne!(camera.target, initial_target);

        // eyeは変わらないはず
        assert_eq!(camera.eye, glam::Vec3::new(0.0, 0.0, 3.0));
    }

    #[test]
    fn test_view_projection_matrix() {
        let config = AppConfig::default();
        let camera = Camera::new(16.0 / 9.0, &config.camera);
        let matrix = camera.build_view_proj_matrix();

        // 行列が有効な値を持っているかチェック
        for i in 0..4 {
            for j in 0..4 {
                assert!(!matrix.col(i)[j].is_nan(), "行列にNaNが含まれている");
                assert!(
                    !matrix.col(i)[j].is_infinite(),
                    "行列に無限大が含まれている"
                );
            }
        }

        // 行列式が0でないことを確認（逆行列が存在する）
        let det = matrix.determinant();
        assert!(det.abs() > f32::EPSILON, "行列式が0に近すぎる: {}", det);
    }

    #[test]
    fn test_camera_aspect_ratio() {
        let config = AppConfig::default();
        let wide_camera = Camera::new(21.0 / 9.0, &config.camera); // ウルトラワイド
        let square_camera = Camera::new(1.0, &config.camera); // 正方形
        let tall_camera = Camera::new(9.0 / 16.0, &config.camera); // 縦長

        assert_eq!(wide_camera.aspect, 21.0 / 9.0);
        assert_eq!(square_camera.aspect, 1.0);
        assert_eq!(tall_camera.aspect, 9.0 / 16.0);
    }

    #[test]
    fn test_camera_fov_range() {
        let config = AppConfig::default();
        let camera = Camera::new(1.0, &config.camera);

        // 視野角が妥当な範囲内にあることを確認
        assert!(camera.fovy > 0.0 && camera.fovy < std::f32::consts::PI);
        assert!(camera.znear > 0.0);
        assert!(camera.zfar > camera.znear);
    }
}
