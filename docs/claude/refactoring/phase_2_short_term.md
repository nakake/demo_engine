# Phase 2: 短期改善

> **目標**: アーキテクチャ改善と機能拡張 + Phase 1延期項目  
> **期間**: 1-2週間  
> **リスク**: 中  
> **前提条件**: Phase 1 完了

## 0. Phase 1 延期項目の完了

**優先度**: 🔴 最高（Phase 1からの引き継ぎ）

### 0.1 統合設定システム（constants.rs + config.rs）

**ステータス**: ✅ **完了** 

**実装済み内容**:
```rust
// src/core/config.rs (新規作成)
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub window: WindowConfig,
    pub camera: CameraConfig,
    pub movement: MovementConfig,
    pub rendering: RenderingConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub resizable: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CameraConfig {
    pub fov_degrees: f32,
    pub znear: f32,
    pub zfar: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MovementConfig {
    pub move_speed: f32,
    pub rotation_speed: f32,
    pub mouse_sensitivity: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RenderingConfig {
    pub clear_color: [f32; 4],
    pub vsync: bool,
    pub msaa_samples: u32,
}

// デフォルト値（constants.rsの代替）
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig {
                width: 800,
                height: 600,
                title: "Demo Engine".to_string(),
                resizable: true,
            },
            camera: CameraConfig {
                fov_degrees: 45.0,
                znear: 0.1,
                zfar: 100.0,
            },
            movement: MovementConfig {
                move_speed: 5.0,
                rotation_speed: 1.0,
                mouse_sensitivity: 0.001,
            },
            rendering: RenderingConfig {
                clear_color: [0.5, 0.2, 0.2, 1.0],
                vsync: true,
                msaa_samples: 1,
            },
        }
    }
}

// 設定ファイル読み込み
impl AppConfig {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn load_or_default() -> Self {
        Self::load_from_file("config.toml").unwrap_or_else(|_| {
            log::info!("config.toml not found, using defaults");
            Self::default()
        })
    }
    
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

// config.toml ファイル例
/*
[window]
width = 1920
height = 1080
title = "Demo Engine"
resizable = true

[camera]
fov_degrees = 60.0
znear = 0.1
zfar = 1000.0

[movement]
move_speed = 10.0
rotation_speed = 2.0
mouse_sensitivity = 0.002

[rendering]
clear_color = [0.1, 0.1, 0.2, 1.0]
vsync = true
msaa_samples = 4
*/
```

**実装内容**:
- ✅ `src/core/config.rs` 作成完了
- ✅ `config.toml` ファイル配置完了
- ✅ AppConfig構造体とサブ構造体（Window, Camera, Movement, Rendering）実装
- ✅ デフォルト値設定、ファイル読み書き機能実装
- ✅ 全7個のconfig関連テストが通過
- ✅ `src/app/mod.rs`でconfig読み込み統合済み
- ✅ `src/scene/demo_scene.rs`でmovement config使用済み
- ✅ `src/scene/camera.rs`でcamera config使用済み
- ✅ `src/graphics/engine.rs`でrendering config使用済み（VSync、clear_color）

**追加済みCargo.toml依存関係**:
```toml
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
tempfile = "3.12" # テスト用
```

### 0.2 ログシステム導入（println! 置換）

**実装内容**:
```rust
// Cargo.toml に追加
[dependencies]
log = "0.4"
env_logger = "0.10"

// src/core/logging.rs (新規作成)
use log::{debug, info, warn, error};

pub fn init_logger() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .init();
}

// 置換例
// Before: println!("W key pressed! Moving forward by {}", move_speed);
// After:  debug!("W key pressed! Moving forward by {}", move_speed);
```

**対象ファイル**:
- `src/scene/demo_scene.rs`: デバッグメッセージ
- `src/input/mod.rs`: 入力ログ
- `src/app/mod.rs`: アプリケーションログ
- `src/graphics/engine.rs`: レンダリングログ

### 0.3 基本メトリクス実装

**実装内容**:
```rust
// src/core/metrics.rs (新規作成)
use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct EngineMetrics {
    frame_times: VecDeque<f32>,
    fps: f32,
    render_objects_count: usize,
    last_update: Instant,
}

impl EngineMetrics {
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(60), // 1秒分
            fps: 0.0,
            render_objects_count: 0,
            last_update: Instant::now(),
        }
    }
    
    pub fn update(&mut self, dt: f32, object_count: usize) {
        self.frame_times.push_back(dt);
        if self.frame_times.len() > 60 {
            self.frame_times.pop_front();
        }
        
        // 移動平均でFPS計算
        let avg_frame_time: f32 = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        self.fps = 1.0 / avg_frame_time;
        self.render_objects_count = object_count;
    }
    
    pub fn get_fps(&self) -> f32 { self.fps }
    pub fn get_frame_time_ms(&self) -> f32 { 
        self.frame_times.back().unwrap_or(&0.0) * 1000.0 
    }
    pub fn get_object_count(&self) -> usize { self.render_objects_count }
    
    // フレームレート警告
    pub fn check_performance(&self) {
        if self.fps < 30.0 {
            log::warn!("Low FPS: {:.1} fps", self.fps);
        }
        if self.get_frame_time_ms() > 33.0 { // 30fps threshold
            log::warn!("High frame time: {:.1}ms", self.get_frame_time_ms());
        }
    }
}

// GraphicsEngine統合
impl GraphicsEngine {
    pub fn render(&mut self, dt: f32, input: &InputState) -> EngineResult<()> {
        self.metrics.update(dt, self.scene.get_render_objects().len());
        self.metrics.check_performance();
        
        #[cfg(debug_assertions)]
        if self.frame_counter % 60 == 0 { // 1秒おき
            log::info!("FPS: {:.1}, Frame time: {:.1}ms, Objects: {}", 
                      self.metrics.get_fps(),
                      self.metrics.get_frame_time_ms(),
                      self.metrics.get_object_count());
        }
        
        // 既存のレンダリング処理...
    }
}
```

## 1. 責任分離とアーキテクチャ改善

### 1.1 GraphicsEngine の分割

**ステータス**: ✅ **完了** (2025-08-12)

**実装済み内容**:
```rust
// src/graphics/renderer.rs (新規作成完了)
pub struct Renderer {
    device: Arc<wgpu::Device>,
    clear_color: [f32; 4],
}

impl Renderer {
    pub fn render_scene(
        &self,
        surface_view: &wgpu::TextureView,
        scene: &dyn Scene,
        resource_manager: &ResourceManager,
    ) -> EngineResult<wgpu::CommandBuffer> {
        // 純粋なレンダリングロジック、CommandBuffer返却
    }
}

// src/graphics/surface_manager.rs (新規作成完了)  
pub struct SurfaceManager {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    format: wgpu::TextureFormat,
    caps: wgpu::SurfaceCapabilities,
}

impl SurfaceManager {
    pub fn acquire_frame(&self) -> EngineResult<SurfaceFrame> {
        // フレーム取得・管理
    }
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        // リサイズ処理
    }
}

// src/graphics/engine.rs (リファクタリング完了)
pub struct GraphicsEngine {
    surface_manager: SurfaceManager,
    renderer: Renderer,
    resource_manager: ResourceManager,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    scene: Box<dyn Scene>,
    config: RenderingConfig,
    metrics: EngineMetrics,
}
```

**達成効果**: 
- ✅ God Object（253行）→ 3コンポーネント分離
- ✅ 単一責任原則の遵守
- ✅ 後方互換性100%維持
- ✅ テスタビリティ向上

### 1.2 設定システム統合完了

**ステータス**: ✅ **完了**

**実装済み内容**:
- ✅ constants.rs の役割も兼ねる統合設計
- ✅ config.toml からの読み込み対応
- ✅ デフォルト値の提供
- ✅ 実行時設定変更対応
- ✅ 全モジュールでの設定活用（App、Scene、Camera、GraphicsEngine）
- ✅ 包括的なテストカバレッジ（15個のテストが通過）

## 2. Phase 2.2 残り項目（基盤整備）

### 2.1 統合設定システム（constants.rs作成）

**優先度**: 🔴 高

**実装内容**:
```rust
// src/constants.rs (新規作成予定)
// マジックナンバーの統一管理

// ウィンドウ関連
pub const DEFAULT_WINDOW_WIDTH: u32 = 800;
pub const DEFAULT_WINDOW_HEIGHT: u32 = 600;
pub const DEFAULT_WINDOW_TITLE: &str = "Demo Engine";

// カメラ関連
pub const DEFAULT_FOV_DEGREES: f32 = 45.0;
pub const DEFAULT_Z_NEAR: f32 = 0.1;
pub const DEFAULT_Z_FAR: f32 = 100.0;

// 移動関連
pub const DEFAULT_MOVE_SPEED: f32 = 5.0;
pub const DEFAULT_ROTATION_SPEED: f32 = 1.0;
pub const DEFAULT_MOUSE_SENSITIVITY: f32 = 0.001;

// レンダリング関連
pub const DEFAULT_CLEAR_COLOR: [f32; 4] = [0.5, 0.2, 0.2, 1.0];
pub const DEFAULT_MSAA_SAMPLES: u32 = 1;

// パフォーマンス関連
pub const FRAME_TIME_BUFFER_SIZE: usize = 60;
pub const LOW_FPS_THRESHOLD: f32 = 30.0;
pub const HIGH_FRAME_TIME_THRESHOLD: f32 = 33.0; // ms
```

### 2.2 ログシステム導入

**優先度**: 🔴 高

**実装内容**:
```rust
// Cargo.toml追加予定
[dependencies]
log = "0.4"
env_logger = "0.10"

// src/main.rs
use log::{debug, info, warn, error};

fn main() {
    env_logger::init();
    info!("Starting Demo Engine");
    // ...
}

// 各ファイルでのprintln!置換
// Before: println!("GraphicsEngine::render called with dt={}", dt);
// After:  debug!("GraphicsEngine::render called with dt={}", dt);
```

### 2.3 基本メトリクス実装拡張

**優先度**: 🔴 高

**実装内容**:
```rust
// src/core/metrics.rs 拡張予定
impl EngineMetrics {
    // 詳細統計の追加
    pub fn get_avg_frame_time(&self) -> f32 { /* 平均フレーム時間 */ }
    pub fn get_min_frame_time(&self) -> f32 { /* 最小フレーム時間 */ }
    pub fn get_max_frame_time(&self) -> f32 { /* 最大フレーム時間 */ }
    
    // メモリ使用量監視
    pub fn track_memory_usage(&mut self) { /* メモリ監視 */ }
    
    // GPU統計
    pub fn track_gpu_time(&mut self, gpu_time: f32) { /* GPU時間追跡 */ }
}
```

## 3. Phase 3移行項目（エンジン機能）

### 3.1 Scene管理システム（Phase 3へ移行）

**移行理由**: エンジン仕様策定が必要

**Phase 3での実装予定**:
```rust
// 本格的なSceneManager設計
pub struct SceneManager {
    scenes: HashMap<SceneId, Box<dyn Scene>>,
    current_scene: Option<SceneId>,
    transition_state: Option<SceneTransition>,
}

pub enum SceneTransition {
    Instant,
    Fade { duration: f32, elapsed: f32 },
    Slide { direction: SlideDirection, duration: f32, elapsed: f32 },
}
```

### 3.2 入力システム設計（Phase 3へ移行）

**移行理由**: カスタマイズ仕様の検討が必要

**Phase 3での設計予定**:
```rust
// 本格的な入力バインディングシステム
pub struct InputBindings {
    key_bindings: HashMap<String, Vec<KeyCode>>,
    mouse_bindings: HashMap<String, MouseButton>,
    gamepad_bindings: HashMap<String, GamepadButton>,
}

pub enum InputAction {
    Movement(MovementAction),
    Camera(CameraAction),
    UI(UIAction),
    System(SystemAction),
}
```

### 3.2 入力の記録・再生機能

**優先度**: 🟢 低

**実装内容**:
```rust
// src/input/recorder.rs (新規作成)
#[derive(Debug, Clone)]
pub struct InputFrame {
    pub timestamp: f32,
    pub keys: Vec<KeyCode>,
    pub mouse_pos: (f32, f32),
    pub mouse_buttons: Vec<MouseButton>,
}

pub struct InputRecorder {
    frames: Vec<InputFrame>,
    is_recording: bool,
    start_time: f32,
}

pub struct InputPlayer {
    frames: Vec<InputFrame>,
    current_frame: usize,
    start_time: f32,
}
```

## 4. レンダリング機能拡張

### 4.1 マルチオブジェクトサポート

**優先度**: 🟡 中

**現状**: 単一クワッドのみレンダリング

#### **Transform システム導入**

```rust
// src/scene/transform.rs (新規作成)
#[derive(Debug, Clone)]
pub struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
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
    
    pub fn matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(
            self.scale,
            self.rotation,
            self.position,
        )
    }
}
```

#### **RenderObject拡張**

```rust
// src/scene/render_object.rs (リファクタリング)
pub struct RenderObject {
    pub mesh_id: ResourceId,
    pub pipeline_id: ResourceId,
    pub transform: Transform, // _transform → transform
    pub visible: bool,
    pub id: ObjectId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(u32);

impl RenderObject {
    pub fn new(mesh_id: ResourceId, pipeline_id: ResourceId) -> Self {
        Self {
            mesh_id,
            pipeline_id,
            transform: Transform::new(),
            visible: true,
            id: ObjectId::generate(),
        }
    }
    
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }
    
    pub fn get_model_matrix(&self) -> glam::Mat4 {
        self.transform.matrix()
    }
}
```

#### **複数オブジェクト管理**

```rust
// src/scene/demo_scene.rs (拡張)
impl DemoScene {
    pub fn add_quad(&mut self, position: glam::Vec3) {
        let mesh_id = ResourceId::new("basic_mesh");
        let pipeline_id = ResourceId::new("basic_pipeline");
        
        let transform = Transform::new()
            .with_position(position)
            .with_scale(glam::Vec3::new(0.8, 0.8, 1.0));
        
        let render_object = RenderObject::new(mesh_id, pipeline_id)
            .with_transform(transform);
        
        self.render_objects.push(render_object);
    }
    
    pub fn add_triangle(&mut self, position: glam::Vec3) {
        let mesh_id = ResourceId::new("triangle_mesh");
        let pipeline_id = ResourceId::new("basic_pipeline");
        
        let transform = Transform::new()
            .with_position(position)
            .with_rotation(glam::Quat::from_rotation_z(0.3));
        
        let render_object = RenderObject::new(mesh_id, pipeline_id)
            .with_transform(transform);
        
        self.render_objects.push(render_object);
    }
    
    pub fn remove_object(&mut self, id: ObjectId) -> bool {
        let before_len = self.render_objects.len();
        self.render_objects.retain(|obj| obj.id != id);
        self.render_objects.len() < before_len
    }
}
```

### 4.1.5 プリミティブ種類拡張

**優先度**: 🟡 中

**拡張対象**: Quad, Triangle → Circle, Pentagon, Cube

#### **新プリミティブ実装**

```rust
// src/resources/primitives/circle.rs (新規作成)
pub struct Circle {
    pub radius: f32,
    pub segments: u32,
}

impl Circle {
    pub fn new(radius: f32, segments: u32) -> Self {
        Self { radius, segments }
    }
}

impl Primitive for Circle {
    type Vertex = ColorVertex;
    
    fn create_vertices() -> Vec<Self::Vertex> {
        Self::new(0.5, 32).create_vertices_with_params()
    }
    
    fn create_indices() -> Option<Vec<u16>> {
        Self::new(0.5, 32).create_indices_with_params()
    }
}

impl Circle {
    fn create_vertices_with_params(&self) -> Vec<ColorVertex> {
        let mut vertices = vec![
            // 中心点
            ColorVertex {
                position: [0.0, 0.0, 0.0],
                color: [1.0, 1.0, 1.0],
            }
        ];
        
        // 円周の頂点（グラデーション色）
        for i in 0..=self.segments {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / self.segments as f32;
            let x = self.radius * angle.cos();
            let y = self.radius * angle.sin();
            
            let hue = i as f32 / self.segments as f32;
            let color = hsv_to_rgb(hue, 1.0, 1.0);
            
            vertices.push(ColorVertex {
                position: [x, y, 0.0],
                color,
            });
        }
        
        vertices
    }
}

// src/resources/primitives/pentagon.rs (新規作成)
pub struct Pentagon;

impl Primitive for Pentagon {
    type Vertex = ColorVertex;
    
    fn create_vertices() -> Vec<Self::Vertex> {
        let mut vertices = vec![
            ColorVertex { position: [0.0, 0.0, 0.0], color: [1.0, 1.0, 1.0] }
        ];
        
        // 五角形の頂点
        for i in 0..5 {
            let angle = 2.0 * std::f32::consts::PI * i as f32 / 5.0 - std::f32::consts::PI / 2.0;
            let x = 0.5 * angle.cos();
            let y = 0.5 * angle.sin();
            
            let color = match i {
                0 => [1.0, 0.0, 0.0], 1 => [0.0, 1.0, 0.0], 2 => [0.0, 0.0, 1.0],
                3 => [1.0, 1.0, 0.0], 4 => [1.0, 0.0, 1.0], _ => [1.0, 1.0, 1.0],
            };
            
            vertices.push(ColorVertex { position: [x, y, 0.0], color });
        }
        
        vertices
    }
    
    fn create_indices() -> Option<Vec<u16>> {
        Some(vec![0, 1, 2,  0, 2, 3,  0, 3, 4,  0, 4, 5,  0, 5, 1])
    }
}

// src/resources/primitives/cube.rs (新規作成)
pub struct Cube {
    pub size: f32,
}

impl Cube {
    pub fn new(size: f32) -> Self { Self { size } }
    
    fn create_vertices_with_size(&self) -> Vec<ColorVertex> {
        let s = self.size * 0.5;
        vec![
            // 前面 (Z+) - 赤系
            ColorVertex { position: [-s, -s,  s], color: [1.0, 0.0, 0.0] },
            ColorVertex { position: [ s, -s,  s], color: [0.0, 1.0, 0.0] },
            ColorVertex { position: [ s,  s,  s], color: [0.0, 0.0, 1.0] },
            ColorVertex { position: [-s,  s,  s], color: [1.0, 1.0, 0.0] },
            
            // 背面 (Z-) - 青系
            ColorVertex { position: [-s, -s, -s], color: [1.0, 0.0, 1.0] },
            ColorVertex { position: [ s, -s, -s], color: [0.0, 1.0, 1.0] },
            ColorVertex { position: [ s,  s, -s], color: [1.0, 1.0, 1.0] },
            ColorVertex { position: [-s,  s, -s], color: [0.5, 0.5, 0.5] },
        ]
    }
    
    fn create_indices_cube(&self) -> Option<Vec<u16>> {
        Some(vec![
            0, 1, 2,  0, 2, 3,  // 前面
            4, 6, 5,  4, 7, 6,  // 背面
            4, 0, 3,  4, 3, 7,  // 左面
            1, 5, 6,  1, 6, 2,  // 右面
            3, 2, 6,  3, 6, 7,  // 上面
            4, 5, 1,  4, 1, 0,  // 下面
        ])
    }
}
```

#### **統合デモシーン**

```rust
impl DemoScene {
    pub fn create_demo_objects(&mut self) {
        // バラエティに富んだオブジェクト配置
        self.add_quad(glam::Vec3::new(0.0, 0.0, 0.0));        // 中央四角形
        self.add_triangle(glam::Vec3::new(-2.0, 0.0, 0.0));   // 左三角形
        self.add_circle(glam::Vec3::new(2.0, 0.0, 0.0), 0.6); // 右円
        self.add_pentagon(glam::Vec3::new(0.0, 2.0, 0.0));    // 上五角形
        self.add_cube(glam::Vec3::new(0.0, -2.0, -1.0), 0.8); // 下立方体
    }
}
```

### 4.2 MSAA実装（Phase 2でスキップ）

**優先度**: ⭕ スキップ  
**理由**: 他の優先機能に集中するため一時的にスキップ

**Note**: config.tomlの`msaa_samples`設定はあるが、GraphicsEngineでの実装は後回し。現在は1x（オフ）で動作継続。

### 4.3 Transform システム

**優先度**: 🟡 中

**実装内容**:
```rust
// src/scene/transform.rs (新規作成)
#[derive(Debug, Clone)]
pub struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        }
    }
    
    pub fn matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(
            self.scale,
            self.rotation,  
            self.position,
        )
    }
}

// src/scene/render_object.rs (リファクタリング)
pub struct RenderObject {
    pub mesh_id: ResourceId,
    pub pipeline_id: ResourceId,
    pub transform: Transform, // 現在は未使用のMat4から変更
}
```

### 4.4 インスタンシング対応

**優先度**: 🟢 低

**実装内容**:
```rust
// src/graphics/instancing.rs (新規作成)
#[derive(Debug, Clone)]
pub struct InstanceData {
    pub model_matrix: glam::Mat4,
    pub color: [f32; 4],
}

impl InstanceData {
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

pub struct InstanceBuffer {
    buffer: Arc<wgpu::Buffer>,
    capacity: usize,
    count: usize,
}
```

## 5. エラーハンドリング強化

### 5.1 エラー回復機能

**優先度**: 🟡 中

**実装内容**:
```rust
// src/core/error.rs (拡張)
pub enum RecoveryAction {
    Retry,
    Fallback,
    Shutdown,
    Ignore,
}

pub trait Recoverable {
    fn recovery_action(&self) -> RecoveryAction;
    fn attempt_recovery(&self) -> Result<(), EngineError>;
}

impl Recoverable for EngineError {
    fn recovery_action(&self) -> RecoveryAction {
        match self {
            EngineError::RenderError(_) => RecoveryAction::Retry,
            EngineError::SurfaceCreation(_) => RecoveryAction::Fallback,
            _ => RecoveryAction::Shutdown,
        }
    }
}
```

### 5.2 ログシステム導入

**優先度**: 🟡 中

**実装内容**:
```rust
// Cargo.toml
[dependencies]
log = "0.4"
env_logger = "0.10"

// src/main.rs
fn main() {
    env_logger::init();
    
    log::info!("Starting Demo Engine");
    // ...
}

// src/graphics/engine.rs  
log::debug!("Rendering frame with {} objects", object_count);
```

## 6. パフォーマンス改善

### 6.1 プロファイリング機能

**優先度**: 🟢 低

**実装内容**:
```rust
// src/core/profiler.rs (新規作成)
pub struct Profiler {
    timings: HashMap<String, Vec<f32>>,
    current_frame: HashMap<String, std::time::Instant>,
}

impl Profiler {
    pub fn start(&mut self, name: &str) {
        self.current_frame.insert(name.to_string(), std::time::Instant::now());
    }
    
    pub fn end(&mut self, name: &str) {
        if let Some(start) = self.current_frame.remove(name) {
            let duration = start.elapsed().as_secs_f32();
            self.timings.entry(name.to_string()).or_insert_with(Vec::new).push(duration);
        }
    }
}
```

## 実装チェックリスト

### Phase 1 延期項目 ✅ **完了**
- [x] **設定システム基盤実装** - 統合AppConfig システム完了
- [x] **テストインフラ** - ConfigとCameraのユニットテスト完了（15個のテスト通過）
- [x] **設定統合** - App、Scene、Camera、GraphicsEngineに設定適用完了

### Phase 2.1 ✅ **完了** (2025-08-12)
- [x] **GraphicsEngine 分割設計** - 3層アーキテクチャ設計完了
- [x] **Renderer 構造体実装** - CommandBuffer返却方式
- [x] **SurfaceManager 構造体実装** - フレーム管理・リサイズ対応
- [x] **統合・後方互換性** - 既存API維持、テスト通過

### Phase 2.2 🚧 **残り項目** (3-5日予定)
- [ ] **constants.rs作成** - マジックナンバー統一管理
- [ ] **ログシステム導入** - println! → log::debug! 置換
- [ ] **基本メトリクス拡張** - 詳細統計・メモリ監視

### Phase 2.3 🎨 **レンダリング拡張** (オプション)
- [ ] **Transform システム** - 位置・回転・スケール制御
- [ ] **マルチオブジェクト** - 複数オブジェクト同時表示
- [ ] **プリミティブ拡張** - Circle, Pentagon, Cube追加

### Phase 3移行項目 📋 **仕様策定段階**
- [ ] **Scene管理システム** - SceneManager設計・遷移システム
- [ ] **入力システム設計** - InputBinding・カスタマイズ対応
- [ ] **リソース管理拡張** - 動的ロード・最適化

### テスト ✅ **部分完了**
- [x] **Config系ユニットテスト** - 7個のテスト通過
- [x] **Camera系ユニットテスト** - 8個のテスト通過  
- [x] **GraphicsEngine統合テスト** - 分割後の動作確認完了
- [ ] Phase 2.2新機能のユニットテスト
- [ ] パフォーマンステスト

## 期待される改善効果

### Phase 2.1完了による効果 ✅
1. **保守性向上**: God Object解決、単一責任原則の実現
2. **テスタビリティ**: 独立したコンポーネントテスト可能
3. **拡張性**: 各コンポーネントの独立した拡張
4. **コード品質**: 253行→3つの責任特化コンポーネント

### Phase 2.2完了予想効果 🎯
1. **開発効率**: 構造化ログによるデバッグ改善
2. **設定管理**: constants.rsによるマジックナンバー解消
3. **パフォーマンス**: 詳細メトリクスによる最適化指針
4. **安定性**: 統一された基盤システム

### Phase 2.3完了予想効果 🎨
1. **視覚的豊かさ**: 多様な図形の同時表示 
2. **3D表現**: 立方体による奥行き感
3. **Transform制御**: 位置・回転・スケールの独立制御
4. **拡張性**: 新プリミティブの容易な追加

### Phase 3準備効果 🚀  
1. **設計品質**: エンジン仕様の慎重な策定
2. **機能完成度**: Scene管理・入力システムの本格実装
3. **拡張性**: 将来の機能追加基盤
4. **エンジン成熟度**: プロダクションレディな設計

このフェーズ区分により、基盤整備とエンジン機能開発の明確な分離が実現されます。