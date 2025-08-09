# Phase 1: 即座のリファクタリング

> **目標**: 技術的負債の解消とコード品質の向上  
> **期間**: 1-2日  
> **リスク**: 低  

## 1. コード品質改善

### 1.1 警告とタイポの修正

**優先度**: 🔴 高

**対象**:
```rust
// 修正対象
src/core/error.rs   // unused import `write` 削除
src/graphics/engine.rs // unused imports 削除
src/resources/manager.rs // unused imports 削除
src/scene/demo_scene.rs // unused imports 削除
src/input/mod.rs    // unused imports 削除
```

**実施内容**:
- [ ] 未使用 import の削除 (19個の警告)
- [ ] 未使用変数の `_` prefix または削除
- [ ] 未使用構造体・関数の削除または `#[allow(dead_code)]`

**期待効果**: コンパイル警告ゼロ、可読性向上

### 1.2 Debug出力の制御

**優先度**: 🟡 中

**対象**:
```rust
// デバッグ出力を制御
src/scene/demo_scene.rs:146,151,154,155,156 // println! statements
src/input/mod.rs:29,33,37 // println! statements
src/app/mod.rs:106 // println! statement
src/graphics/engine.rs:126 // println! statement
```

**実施内容**:
```rust
// Before
println!("W key pressed! Moving forward by {}", move_speed);

// After
#[cfg(debug_assertions)]
eprintln!("[DEBUG] W key pressed! Moving forward by {}", move_speed);

// Or use log crate
log::debug!("W key pressed! Moving forward by {}", move_speed);
```

**期待効果**: 本番環境での不要な出力除去、デバッグ情報の構造化

### 1.3 マジックナンバーの定数化

**優先度**: 🟡 中

**対象**:
```rust
// src/app/mod.rs
.with_inner_size(winit::dpi::PhysicalSize::new(800.0, 600.0))

// src/scene/demo_scene.rs  
let move_speed = 5.0 * dt;
let rotation_speed = 1.0 * dt;

// src/scene/camera.rs
fovy: 45.0_f32.to_radians(),
znear: 0.1,
zfar: 100.0,
```

**実施内容**:
```rust
// src/core/constants.rs (新規作成)
pub mod constants {
    // Window
    pub const DEFAULT_WINDOW_WIDTH: u32 = 800;
    pub const DEFAULT_WINDOW_HEIGHT: u32 = 600;
    pub const DEFAULT_WINDOW_TITLE: &str = "Demo Engine";
    
    // Camera
    pub const DEFAULT_FOV_DEGREES: f32 = 45.0;
    pub const DEFAULT_ZNEAR: f32 = 0.1;
    pub const DEFAULT_ZFAR: f32 = 100.0;
    
    // Movement
    pub const DEFAULT_MOVE_SPEED: f32 = 5.0;
    pub const DEFAULT_ROTATION_SPEED: f32 = 1.0;
    
    // Colors
    pub const CLEAR_COLOR: wgpu::Color = wgpu::Color {
        r: 0.5, g: 0.2, b: 0.2, a: 1.0
    };
}
```

**期待効果**: 設定の一元管理、保守性向上

## 2. エラーハンドリング強化

### 2.1 未使用エラー型の整理

**優先度**: 🟡 中

**対象**: `src/core/error.rs`

**実施内容**:
```rust
// 使用されていないエラー型を削除またはコメント化
pub enum EngineError {
    // 使用中
    AdapterRequest(String),
    DeviceRequest(String),
    SurfaceCreation(String),
    RenderError(String),
    
    // 未使用（将来用のためコメント化）
    // WindowCreation(String),
    // SurfaceConfiguration(String),
    // ShaderCompilation(String),
    // BufferCreation(String),
    // PipelineCreation(String),
    // SceneNotFound(String),
}
```

**期待効果**: コード明確化、警告削減

### 2.2 エラーコンテキストの改善

**優先度**: 🟢 低

**実施内容**:
```rust
// Before
EngineError::AdapterRequest(format!("Failed to request adapter: {}", e))

// After  
EngineError::AdapterRequest {
    message: format!("Failed to request adapter: {}", e),
    source: Some(Box::new(e)),
}
```

## 3. ドキュメント整備

### 3.1 コード内ドキュメント

**優先度**: 🟢 低

**実施内容**:
```rust
/// WGPU を使用した 3D グラフィックスエンジン
/// 
/// # Examples
/// 
/// ```
/// let engine = GraphicsEngine::new(window, scene).await?;
/// engine.render(dt, input)?;
/// ```
pub struct GraphicsEngine {
    // ...
}
```

### 3.2 README.md 更新

**優先度**: 🟢 低

**実施内容**:
- キーボード操作の説明
- スクリーンショットの追加
- 依存関係の詳細
- ビルド要件の明確化

## 4. テスト基盤準備

### 4.1 テストディレクトリ構造

**優先度**: 🟢 低

**実施内容**:
```
tests/
├── integration/
│   ├── mod.rs
│   ├── engine_test.rs
│   └── scene_test.rs
├── unit/
│   ├── mod.rs
│   ├── resource_test.rs
│   └── camera_test.rs
└── fixtures/
    ├── test_scene.rs
    └── mock_resources.rs
```

### 4.2 基本テストの追加

**実施内容**:
```rust
// tests/unit/resource_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resource_id_generation() {
        let id1 = ResourceId::new("test");
        let id2 = ResourceId::new("test");
        assert_eq!(id1, id2);
    }
}
```

## 5. パフォーマンス監視

### 5.1 基本メトリクス

**優先度**: 🟢 低

**実施内容**:
```rust
// src/core/metrics.rs (新規作成)
pub struct EngineMetrics {
    pub frame_time: f32,
    pub fps: f32,
    pub render_objects_count: usize,
    pub memory_usage: usize,
}

impl EngineMetrics {
    pub fn update(&mut self, dt: f32) {
        self.frame_time = dt;
        self.fps = 1.0 / dt;
    }
}
```

## 実装チェックリスト

- [ ] **Step 1**: 未使用 import・変数の削除
- [ ] **Step 2**: Debug出力をログシステムに置換
- [ ] **Step 3**: constants.rs 作成とマジックナンバー移行
- [ ] **Step 4**: 未使用エラー型の整理
- [ ] **Step 5**: 基本的なドキュメント追加
- [ ] **Step 6**: テストディレクトリ構造作成
- [ ] **Step 7**: 基本メトリクス実装
- [ ] **Step 8**: cargo build で警告ゼロ確認
- [ ] **Step 9**: cargo test 実行確認
- [ ] **Step 10**: リファクタリング後の動作確認

## 期待される改善効果

1. **開発体験**: 警告のないクリーンなコード
2. **保守性**: 設定の一元化、構造化されたログ
3. **テスタビリティ**: テスト基盤の確立
4. **プロフェッショナル性**: ドキュメント整備、メトリクス監視

このフェーズは **リスクが低く効果が確実** で、後続のより大きなリファクタリングの基盤となります。