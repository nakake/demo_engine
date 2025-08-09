///
/// ### 引数
/// - eye - カメラの位置
/// - target - カメラが向いている点
/// - up - カメラの上方向ベクトル
/// - aspect - アスペクト比 (window_width / window_hight)
/// - fovy - 視野角
/// - znear - 描画する最小距離
///   - これより近いオブジェクトは描画されない
///   - 一般的な値: 0.1 〜 1.0
///   - 注意: 0にすると計算エラーが発生
/// - zfar - 描画する最大距離
///   - これより遠いオブジェクトは描画されない
///   - 一般的な値: 100.0 〜 10000.0
///   - 効果: パフォーマンス最適化（遠すぎるものを描画しない）
///
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
}
