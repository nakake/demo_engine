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

**優先度**: 🔴 高

**現在の問題**:
```rust
// GraphicsEngine が複数の責任を持っている
impl GraphicsEngine {
    pub fn new() { ... }      // 初期化
    pub fn resize() { ... }   // サイズ変更
    pub fn render() { ... }   // レンダリング
    // シーン管理、リソース管理、サーフェス管理...
}
```

**改善後の設計**:
```rust
// src/graphics/renderer.rs (新規作成)
pub struct Renderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    command_encoder_pool: Vec<wgpu::CommandEncoder>,
}

// src/graphics/surface_manager.rs (新規作成)  
pub struct SurfaceManager {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

// src/graphics/engine.rs (リファクタリング)
pub struct GraphicsEngine {
    renderer: Renderer,
    surface_manager: SurfaceManager,
    // scene は外部から注入
}
```

**実装ステップ**:
1. `Renderer` 構造体作成
2. `SurfaceManager` 構造体作成  
3. `GraphicsEngine` から機能を移行
4. テスト追加
5. 統合テスト

**期待効果**: 単一責任原則の遵守、テスタビリティ向上

### 1.2 設定システム統合完了

**ステータス**: ✅ **完了**

**実装済み内容**:
- ✅ constants.rs の役割も兼ねる統合設計
- ✅ config.toml からの読み込み対応
- ✅ デフォルト値の提供
- ✅ 実行時設定変更対応
- ✅ 全モジュールでの設定活用（App、Scene、Camera、GraphicsEngine）
- ✅ 包括的なテストカバレッジ（15個のテストが通過）

## 2. Scene システム強化

### 2.1 SceneManager の実装

**優先度**: 🟡 中

**現状**: SceneManager が未使用状態

**改善内容**:
```rust  
// src/scene/manager.rs (リファクタリング)
pub struct SceneManager {
    scenes: HashMap<SceneId, Box<dyn Scene>>,
    current_scene: Option<SceneId>,
    transition_state: Option<SceneTransition>,
}

pub enum SceneTransition {
    FadeOut { duration: f32, elapsed: f32 },
    FadeIn { duration: f32, elapsed: f32 },
}

impl SceneManager {
    pub fn transition_to(&mut self, scene_id: SceneId, transition: SceneTransition) {
        self.transition_state = Some(transition);
        // transition logic
    }
    
    pub fn update(&mut self, dt: f32, input: &InputState) {
        // トランジション処理
        if let Some(transition) = &mut self.transition_state {
            // ...
        }
        
        // 現在のシーン更新
        if let Some(scene) = self.get_current_scene_mut() {
            scene.update(dt, input);
        }
    }
}
```

### 2.2 複数シーンサポート

**実装内容**:
```rust
// src/scene/menu_scene.rs (新規作成)
pub struct MenuScene {
    // UI elements, buttons, etc.
}

// src/scene/game_scene.rs (新規作成)
pub struct GameScene {
    // Game objects, physics, etc.
}

// src/scene/mod.rs
pub enum SceneType {
    Menu,
    Game,
    Demo,
}
```

## 3. 入力システム改善

### 3.1 入力バインディング設定

**優先度**: 🟡 中

**実装内容**:
```rust
// src/input/bindings.rs (新規作成)
use std::collections::HashMap;
use winit::keyboard::KeyCode;

#[derive(Debug, Clone)]
pub struct InputBindings {
    key_bindings: HashMap<String, Vec<KeyCode>>,
    mouse_bindings: HashMap<String, MouseButton>,
}

impl InputBindings {
    pub fn default_bindings() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert("move_forward".to_string(), vec![KeyCode::KeyW]);
        bindings.insert("move_backward".to_string(), vec![KeyCode::KeyS]);
        bindings.insert("move_left".to_string(), vec![KeyCode::KeyA]);
        bindings.insert("move_right".to_string(), vec![KeyCode::KeyD]);
        bindings.insert("move_up".to_string(), vec![KeyCode::KeyE]);
        bindings.insert("move_down".to_string(), vec![KeyCode::KeyQ]);
        
        Self { key_bindings: bindings, mouse_bindings: HashMap::new() }
    }
    
    pub fn is_action_pressed(&self, input_state: &InputState, action: &str) -> bool {
        if let Some(keys) = self.key_bindings.get(action) {
            keys.iter().any(|key| input_state.is_key_pressed(*key))
        } else {
            false
        }
    }
}

// src/input/actions.rs (新規作成)
pub enum InputAction {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    MoveUp,  
    MoveDown,
    RotateLeft,
    RotateRight,
    RotateUp,
    RotateDown,
    Exit,
    Pause,
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

**改善内容**:
```rust
// src/scene/demo_scene.rs
impl Scene for DemoScene {
    fn initialize(&mut self, resource_manager: &mut ResourceManager) {
        // 複数オブジェクトの追加
        self.add_quad(glam::Vec3::new(0.0, 0.0, 0.0));
        self.add_quad(glam::Vec3::new(2.0, 0.0, 0.0));
        self.add_triangle(glam::Vec3::new(-2.0, 0.0, 0.0));
    }
    
    fn add_quad(&mut self, position: glam::Vec3) {
        // Quad の生成とRenderObjectの追加
    }
    
    fn add_triangle(&mut self, position: glam::Vec3) {
        // Triangle の生成とRenderObjectの追加
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

### Week 1 🚧 **進行中**
- [ ] GraphicsEngine 分割設計
- [ ] Renderer 構造体実装
- [ ] SurfaceManager 構造体実装
- [ ] SceneManager リファクタリング

### Week 2  
- [ ] 入力バインディングシステム
- [ ] Transform システム
- [ ] マルチオブジェクト対応
- [ ] エラー回復機能
- [ ] ログシステム統合

### テスト ✅ **部分完了**
- [x] **Config系ユニットテスト** - 7個のテスト通過
- [x] **Camera系ユニットテスト** - 8個のテスト通過
- [ ] 各新機能のユニットテスト
- [ ] 統合テスト
- [ ] パフォーマンステスト

## 期待される改善効果

1. **保守性**: 責任分離、設定外部化
2. **拡張性**: マルチシーン、Transform システム  
3. **安定性**: エラー回復、ログシステム
4. **ユーザビリティ**: 設定可能な入力、複数オブジェクト
5. **開発体験**: プロファイリング、構造化ログ

このフェーズの完了により、より本格的な 3D アプリケーション開発の基盤が整います。