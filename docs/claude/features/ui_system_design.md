# UI基盤システム設計

> **目的**: Phase 2完了後の気分転換と新機能追加  
> **対象**: リアルタイム3Dエンジン向けのUI基盤構築  
> **期間**: 1-2週間（段階的実装）  
> **難易度**: 中-高（新領域）

## 1. システム全体アーキテクチャ

### 1.1 アーキテクチャ概要

```
                    UI Manager
                        │
           ┌─────────────┼─────────────┐
           │             │             │
    UI Renderer ←─→ Event System ←─→ Layout Engine
           │             │             │
    ┌──────┴──────┐     │      ┌──────┴──────┐
    │  UI Layer   │     │      │  Component  │
    │  Pipeline   │     │      │   System    │
    └─────────────┘     │      └─────────────┘
           │             │             │
    Graphics Engine ←────┴────→ Input System
```

### 1.2 既存システムとの統合

- **Graphics Engine**: 2レイヤーレンダリング（3D + UI）
- **Input System**: マウス・キーボードイベントの UI への転送
- **Scene System**: UI による 3D シーン制御
- **Resource Manager**: UI テクスチャ・フォント・シェーダー管理

## 2. 設計原則

### 2.1 Immediate Mode GUI

```rust
// Immediate Mode GUI (imgui-rs風) - 推奨アプローチ
fn render_ui(&mut self, ui: &mut UIContext) {
    ui.window("Debug Panel", || {
        ui.text("FPS: {}", self.fps);
        if ui.button("Reset Camera") {
            self.reset_camera();
        }
        ui.slider("Move Speed", &mut self.move_speed, 0.1..10.0);
    });
}

// メリット:
// - シンプルな実装
// - 状態管理不要
// - デバッグ用途に最適
```

### 2.2 座標系統一

```rust
// UI座標系: スクリーン空間 (0,0=左上、Y軸下向き)
// 3D座標系: ワールド空間 (0,0,0=中央、Y軸上向き)

pub struct UICoordinate {
    pub x: f32, // 0.0 ~ screen_width
    pub y: f32, // 0.0 ~ screen_height
}

pub struct UITransform {
    pub position: UICoordinate,
    pub size: UISize,
    pub anchor: AnchorType, // TopLeft, Center, BottomRight等
}

// 座標変換
impl UICoordinate {
    pub fn to_ndc(&self, screen_size: (f32, f32)) -> (f32, f32) {
        let x_ndc = (self.x / screen_size.0) * 2.0 - 1.0;
        let y_ndc = -((self.y / screen_size.1) * 2.0 - 1.0); // Y軸反転
        (x_ndc, y_ndc)
    }
}
```

### 2.3 モジュラー設計

各 UI システムは独立したモジュールとして設計し、段階的実装を可能にします。

## 3. モジュール構成

```rust
src/ui/
├── mod.rs              // UI統合API
├── manager.rs          // UIManager - UI全体の管理
├── context.rs          // UIContext - 描画コンテキスト  
├── renderer.rs         // UIRenderer - 2D描画エンジン
├── layout/
│   ├── mod.rs         // レイアウトシステム
│   ├── anchor.rs      // アンカー・位置決め
│   └── flex.rs        // Flexbox風レイアウト（将来）
├── components/
│   ├── mod.rs         // UI基本コンポーネント
│   ├── text.rs        // テキスト描画
│   ├── button.rs      // ボタン
│   ├── slider.rs      // スライダー
│   ├── window.rs      // ウィンドウ・パネル
│   └── primitives.rs  // 基本図形(Rectangle, Circle等)
├── events/
│   ├── mod.rs         // UIイベントシステム
│   ├── mouse.rs       // マウスイベント
│   └── keyboard.rs    // キーボードイベント
├── style/
│   ├── mod.rs         // スタイルシステム
│   ├── theme.rs       // テーマ・色管理
│   └── font.rs        // フォント管理
└── integration/
    ├── mod.rs         // GraphicsEngine統合
    └── input_bridge.rs // Input統合
```

## 4. 核となるトレイト設計

### 4.1 UI要素基本トレイト

```rust
// UI要素の基本トレイト
pub trait UIElement {
    fn render(&self, ctx: &mut UIContext, transform: &UITransform);
    fn handle_event(&mut self, event: &UIEvent) -> EventResponse;
    fn get_bounds(&self) -> UIRect;
    fn is_interactive(&self) -> bool { true }
}

// レイアウト可能要素
pub trait UILayout: UIElement {
    fn layout(&mut self, available_space: UIRect) -> UISize;
    fn add_child(&mut self, child: Box<dyn UIElement>);
}

// UI描画コンテキスト
pub trait UIRenderer {
    fn draw_rect(&mut self, rect: UIRect, style: &RectStyle);
    fn draw_text(&mut self, text: &str, pos: UICoordinate, style: &TextStyle);
    fn draw_circle(&mut self, center: UICoordinate, radius: f32, style: &CircleStyle);
    fn draw_line(&mut self, start: UICoordinate, end: UICoordinate, style: &LineStyle);
}
```

### 4.2 UI描画コンテキスト

```rust
pub struct UIContext {
    renderer: Box<dyn UIRenderer>,
    event_system: UIEventSystem,
    style_stack: Vec<UIStyle>,
    layout_stack: Vec<UIRect>,
    screen_size: (f32, f32),
    current_frame: u64,
}

impl UIContext {
    // 基本描画API
    pub fn text(&mut self, text: &str, pos: UICoordinate) {
        let style = self.current_text_style();
        self.renderer.draw_text(text, pos, &style);
    }
    
    pub fn button(&mut self, label: &str, bounds: UIRect) -> bool {
        let style = self.current_button_style();
        let is_hovered = self.is_mouse_over(bounds);
        let is_clicked = is_hovered && self.is_mouse_pressed();
        
        // ボタン描画
        self.renderer.draw_rect(bounds, &style.background(is_hovered, is_clicked));
        self.text(label, bounds.center());
        
        is_clicked
    }
    
    // レイアウトAPI
    pub fn window<F>(&mut self, title: &str, content: F) 
    where F: FnOnce(&mut Self) {
        // ウィンドウ背景描画
        let window_bounds = self.allocate_window_space(title);
        self.draw_window_background(window_bounds, title);
        
        // 子要素描画領域設定
        self.push_layout_bounds(window_bounds.content_area());
        content(self);
        self.pop_layout_bounds();
    }
}
```

## 5. レンダリングパイプライン統合

### 5.1 2レイヤーレンダリング

```rust
impl GraphicsEngine {
    pub fn render(&mut self, dt: f32, input: &InputState) -> EngineResult<()> {
        self.metrics.update(dt, self.scene.get_render_objects().len());
        self.metrics.check_performance();

        // シーン更新
        self.scene.update(dt, input);
        self.scene.update_camera_uniform();

        // UIイベント処理
        self.ui_manager.handle_input(input);
        
        let surface_frame = self.surface_manager.acquire_frame()?;
        
        // レイヤー1: 3Dシーンレンダリング
        let scene_command_buffer = self.renderer.render_scene(
            &surface_frame.view,
            self.scene.as_ref(),
            self.scene.get_resource_manager(),
        )?;
        
        // レイヤー2: UIオーバーレイレンダリング
        let ui_command_buffer = self.ui_renderer.render_ui(
            &surface_frame.view,
            &mut self.ui_manager,
        )?;
        
        // コマンドバッファ結合実行
        self.queue.submit([scene_command_buffer, ui_command_buffer]);
        surface_frame.present();
        Ok(())
    }
}
```

### 5.2 UI専用シェーダー

```wgsl
// assets/shaders/ui/ui_basic.wgsl

struct UIVertex {
    @location(0) position: vec2<f32>,  // スクリーン座標
    @location(1) uv: vec2<f32>,        // テクスチャ座標
    @location(2) color: vec4<f32>,     // 頂点カラー
}

struct UIUniform {
    screen_size: vec2<f32>,  // スクリーン解像度
    ui_scale: f32,           // UIスケール
    time: f32,               // アニメーション用
}

@group(0) @binding(0)
var<uniform> ui_uniform: UIUniform;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(input: UIVertex) -> VertexOutput {
    // スクリーン座標をNDCに変換
    let ndc_pos = (input.position / ui_uniform.screen_size) * 2.0 - 1.0;
    
    return VertexOutput {
        position: vec4<f32>(ndc_pos.x, -ndc_pos.y, 0.0, 1.0), // Y軸反転
        uv: input.uv,
        color: input.color,
    };
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
```

### 5.3 UI専用頂点・メッシュ

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UIVertex {
    pub position: [f32; 2],  // スクリーン座標
    pub uv: [f32; 2],        // テクスチャ座標  
    pub color: [f32; 4],     // RGBA
}

impl UIVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<UIVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2, // position
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2, // uv
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4, // color
                },
            ],
        }
    }
}
```

## 6. イベントシステム

### 6.1 UIイベント定義

```rust
#[derive(Debug, Clone)]
pub enum UIEvent {
    MouseMove { pos: UICoordinate, delta: UICoordinate },
    MouseDown { pos: UICoordinate, button: MouseButton },
    MouseUp { pos: UICoordinate, button: MouseButton },
    MouseWheel { pos: UICoordinate, delta: f32 },
    KeyPressed { key: KeyCode, modifiers: KeyModifiers },
    KeyReleased { key: KeyCode, modifiers: KeyModifiers },
    WindowResize { width: u32, height: u32 },
    Focus { element_id: UIElementId },
    Unfocus { element_id: UIElementId },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventResponse {
    Handled,      // イベント処理済み、伝播停止
    Propagate,    // 親要素に伝播
    Ignored,      // イベント無視
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UIElementId(u32);
```

### 6.2 イベントシステム実装

```rust
pub struct UIEventSystem {
    event_queue: Vec<UIEvent>,
    focus_stack: Vec<UIElementId>,
    hover_element: Option<UIElementId>,
    mouse_position: UICoordinate,
    pressed_keys: HashSet<KeyCode>,
}

impl UIEventSystem {
    pub fn handle_winit_event(&mut self, event: &winit::event::WindowEvent) {
        match event {
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let ui_pos = UICoordinate::new(position.x as f32, position.y as f32);
                self.event_queue.push(UIEvent::MouseMove { 
                    pos: ui_pos, 
                    delta: ui_pos - self.mouse_position 
                });
                self.mouse_position = ui_pos;
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                let event = match state {
                    winit::event::ElementState::Pressed => UIEvent::MouseDown {
                        pos: self.mouse_position,
                        button: convert_mouse_button(*button),
                    },
                    winit::event::ElementState::Released => UIEvent::MouseUp {
                        pos: self.mouse_position,
                        button: convert_mouse_button(*button),
                    },
                };
                self.event_queue.push(event);
            }
            // 他のイベント処理...
            _ => {}
        }
    }
    
    pub fn process_events(&mut self, ui_elements: &mut [Box<dyn UIElement>]) {
        for event in self.event_queue.drain(..) {
            self.dispatch_event(&event, ui_elements);
        }
    }
}
```

## 7. スタイルシステム

### 7.1 スタイル定義

```rust
// CSS-like スタイルシステム
#[derive(Debug, Clone)]
pub struct UIStyle {
    // 背景・境界
    pub background_color: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub border_radius: f32,
    
    // テキスト
    pub font_size: f32,
    pub text_color: Color,
    pub text_align: TextAlign,
    pub font_weight: FontWeight,
    
    // レイアウト
    pub margin: Spacing,
    pub padding: Spacing,
    pub width: SizeConstraint,   // Fixed(100.0), Percentage(50.0), Auto
    pub height: SizeConstraint,
    
    // インタラクション
    pub hover_color: Color,
    pub pressed_color: Color,
    pub disabled_color: Color,
    
    // アニメーション
    pub transition: Option<Transition>,
}

#[derive(Debug, Clone)]
pub enum SizeConstraint {
    Fixed(f32),
    Percentage(f32),
    Auto,
    Fill,
}

#[derive(Debug, Clone)]
pub struct Spacing {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Debug, Clone)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}
```

### 7.2 テーマシステム

```rust
#[derive(Debug, Clone)]
pub struct UITheme {
    // 基本カラー
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub surface: Color,
    pub text: Color,
    pub text_secondary: Color,
    
    // インタラクション
    pub button_hover: Color,
    pub button_pressed: Color,
    pub focus_outline: Color,
    
    // 状態
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    
    // サイズ
    pub font_size_small: f32,
    pub font_size_medium: f32,
    pub font_size_large: f32,
    pub border_radius: f32,
    pub padding_small: f32,
    pub padding_medium: f32,
    pub padding_large: f32,
}

impl Default for UITheme {
    fn default() -> Self {
        Self {
            primary: Color::new(0.2, 0.6, 1.0, 1.0),
            secondary: Color::new(0.6, 0.6, 0.6, 1.0),
            background: Color::new(0.1, 0.1, 0.1, 0.9),
            surface: Color::new(0.15, 0.15, 0.15, 0.95),
            text: Color::new(1.0, 1.0, 1.0, 1.0),
            text_secondary: Color::new(0.8, 0.8, 0.8, 1.0),
            // ... 他の色定義
        }
    }
}
```

## 8. 基本コンポーネント設計

### 8.1 ボタンコンポーネント

```rust
#[derive(Debug)]
pub struct Button {
    pub text: String,
    pub bounds: UIRect,
    pub style: ButtonStyle,
    pub enabled: bool,
    pub id: UIElementId,
    
    // 状態
    is_hovered: bool,
    is_pressed: bool,
}

impl Button {
    pub fn new(text: String, bounds: UIRect) -> Self {
        Self {
            text,
            bounds,
            style: ButtonStyle::default(),
            enabled: true,
            id: UIElementId::generate(),
            is_hovered: false,
            is_pressed: false,
        }
    }
}

impl UIElement for Button {
    fn render(&self, ctx: &mut UIContext, transform: &UITransform) {
        let actual_bounds = self.bounds.transform(transform);
        
        // 背景描画
        let bg_color = if !self.enabled {
            self.style.disabled_color
        } else if self.is_pressed {
            self.style.pressed_color
        } else if self.is_hovered {
            self.style.hover_color
        } else {
            self.style.background_color
        };
        
        ctx.draw_rect(actual_bounds, &RectStyle {
            fill_color: bg_color,
            border_color: self.style.border_color,
            border_width: self.style.border_width,
            border_radius: self.style.border_radius,
        });
        
        // テキスト描画
        let text_pos = actual_bounds.center();
        ctx.draw_text(&self.text, text_pos, &TextStyle {
            color: self.style.text_color,
            size: self.style.font_size,
            align: TextAlign::Center,
        });
    }
    
    fn handle_event(&mut self, event: &UIEvent) -> EventResponse {
        if !self.enabled {
            return EventResponse::Ignored;
        }
        
        match event {
            UIEvent::MouseMove { pos, .. } => {
                let was_hovered = self.is_hovered;
                self.is_hovered = self.bounds.contains(*pos);
                
                if was_hovered != self.is_hovered {
                    EventResponse::Handled
                } else {
                    EventResponse::Ignored
                }
            }
            UIEvent::MouseDown { pos, button: MouseButton::Left } => {
                if self.bounds.contains(*pos) {
                    self.is_pressed = true;
                    EventResponse::Handled
                } else {
                    EventResponse::Ignored
                }
            }
            UIEvent::MouseUp { pos, button: MouseButton::Left } => {
                let was_pressed = self.is_pressed;
                self.is_pressed = false;
                
                if was_pressed && self.bounds.contains(*pos) {
                    // ボタンクリック処理
                    EventResponse::Handled
                } else {
                    EventResponse::Ignored
                }
            }
            _ => EventResponse::Ignored,
        }
    }
    
    fn get_bounds(&self) -> UIRect {
        self.bounds
    }
}
```

### 8.2 スライダーコンポーネント

```rust
#[derive(Debug)]
pub struct Slider {
    pub min: f32,
    pub max: f32,
    pub value: f32,
    pub bounds: UIRect,
    pub style: SliderStyle,
    pub enabled: bool,
    pub id: UIElementId,
    
    // 状態
    is_dragging: bool,
    drag_offset: f32,
}

impl Slider {
    pub fn new(min: f32, max: f32, initial_value: f32, bounds: UIRect) -> Self {
        Self {
            min,
            max,
            value: initial_value.clamp(min, max),
            bounds,
            style: SliderStyle::default(),
            enabled: true,
            id: UIElementId::generate(),
            is_dragging: false,
            drag_offset: 0.0,
        }
    }
    
    fn value_to_position(&self, value: f32) -> f32 {
        let normalized = (value - self.min) / (self.max - self.min);
        self.bounds.min.x + normalized * self.bounds.width()
    }
    
    fn position_to_value(&self, pos: f32) -> f32 {
        let normalized = (pos - self.bounds.min.x) / self.bounds.width();
        (self.min + normalized * (self.max - self.min)).clamp(self.min, self.max)
    }
}

impl UIElement for Slider {
    fn render(&self, ctx: &mut UIContext, transform: &UITransform) {
        let actual_bounds = self.bounds.transform(transform);
        
        // トラック描画
        let track_rect = UIRect {
            min: UICoordinate::new(actual_bounds.min.x, actual_bounds.center().y - 2.0),
            max: UICoordinate::new(actual_bounds.max.x, actual_bounds.center().y + 2.0),
        };
        
        ctx.draw_rect(track_rect, &RectStyle {
            fill_color: self.style.track_color,
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
            border_radius: self.style.track_radius,
        });
        
        // ハンドル描画
        let handle_pos = self.value_to_position(self.value);
        let handle_rect = UIRect::centered(
            UICoordinate::new(handle_pos, actual_bounds.center().y),
            self.style.handle_size
        );
        
        let handle_color = if self.is_dragging {
            self.style.handle_pressed_color
        } else {
            self.style.handle_color
        };
        
        ctx.draw_rect(handle_rect, &RectStyle {
            fill_color: handle_color,
            border_color: self.style.handle_border_color,
            border_width: 1.0,
            border_radius: self.style.handle_size / 2.0,
        });
    }
    
    fn handle_event(&mut self, event: &UIEvent) -> EventResponse {
        // ドラッグ処理の実装...
        EventResponse::Ignored
    }
    
    fn get_bounds(&self) -> UIRect {
        self.bounds
    }
}
```

## 9. フォント・テキスト描画

### 9.1 ビットマップフォント（Phase 1推奨）

```rust
pub struct BitmapFont {
    texture: Arc<wgpu::Texture>,
    char_map: HashMap<char, CharInfo>,
    line_height: f32,
    baseline: f32,
}

#[derive(Debug, Clone)]
pub struct CharInfo {
    pub uv_rect: UIRect,     // テクスチャ内UV座標
    pub size: UISize,        // 文字サイズ
    pub offset: UICoordinate, // ベースラインからのオフセット
    pub advance: f32,        // 次の文字までの距離
}

impl BitmapFont {
    pub fn new(texture: Arc<wgpu::Texture>, font_data: FontData) -> Self {
        // ビットマップフォントファイル（.fnt等）の読み込み
        Self {
            texture,
            char_map: font_data.char_map,
            line_height: font_data.line_height,
            baseline: font_data.baseline,
        }
    }
    
    pub fn measure_text(&self, text: &str, scale: f32) -> UISize {
        let mut width = 0.0;
        let mut height = self.line_height * scale;
        
        for ch in text.chars() {
            if let Some(char_info) = self.char_map.get(&ch) {
                width += char_info.advance * scale;
            }
        }
        
        UISize::new(width, height)
    }
    
    pub fn create_text_mesh(&self, text: &str, pos: UICoordinate, scale: f32, color: Color) -> Vec<UIVertex> {
        let mut vertices = Vec::new();
        let mut cursor_x = pos.x;
        
        for ch in text.chars() {
            if let Some(char_info) = self.char_map.get(&ch) {
                let char_pos = UICoordinate::new(
                    cursor_x + char_info.offset.x * scale,
                    pos.y + char_info.offset.y * scale
                );
                
                let char_size = UISize::new(
                    char_info.size.width * scale,
                    char_info.size.height * scale
                );
                
                // 4頂点を生成（左上、右上、左下、右下）
                let quad_vertices = create_text_quad(char_pos, char_size, char_info.uv_rect, color);
                vertices.extend_from_slice(&quad_vertices);
                
                cursor_x += char_info.advance * scale;
            }
        }
        
        vertices
    }
}

fn create_text_quad(pos: UICoordinate, size: UISize, uv_rect: UIRect, color: Color) -> [UIVertex; 6] {
    let color_array = color.to_array();
    
    // 2つの三角形で四角形を構成
    [
        // 上三角形
        UIVertex { position: [pos.x, pos.y], uv: [uv_rect.min.x, uv_rect.min.y], color: color_array },
        UIVertex { position: [pos.x + size.width, pos.y], uv: [uv_rect.max.x, uv_rect.min.y], color: color_array },
        UIVertex { position: [pos.x, pos.y + size.height], uv: [uv_rect.min.x, uv_rect.max.y], color: color_array },
        
        // 下三角形  
        UIVertex { position: [pos.x, pos.y + size.height], uv: [uv_rect.min.x, uv_rect.max.y], color: color_array },
        UIVertex { position: [pos.x + size.width, pos.y], uv: [uv_rect.max.x, uv_rect.min.y], color: color_array },
        UIVertex { position: [pos.x + size.width, pos.y + size.height], uv: [uv_rect.max.x, uv_rect.max.y], color: color_array },
    ]
}
```

## 10. 実装段階とマイルストーン

### Phase UI.1: 基礎システム (1-2日)

**目標**: 最小限のUI表示

```rust
// 実装項目:
1. UIContext基本構造
2. UIRenderer trait + 基本実装
3. 基本図形描画（Rectangle, Circle）
4. 基本テキスト描画（ビットマップフォント）
5. GraphicsEngine統合（2レイヤーレンダリング）
6. 基本InputSystem統合

// デモ:
ui.text("Hello UI!", UICoordinate::new(10.0, 10.0));
ui.rect(UIRect::new(100.0, 100.0, 200.0, 150.0), Color::RED);
```

### Phase UI.2: 基本コンポーネント (2-3日)

**目標**: インタラクティブ要素

```rust
// 実装項目:
7. Button実装（ホバー・クリック処理）
8. UIEvent システム
9. 基本スタイルシステム
10. デバッグパネル実装

// デモ:
if ui.button("Reset Camera", button_rect) {
    engine.reset_camera();
}
ui.text(format!("FPS: {:.1}", engine.get_fps()), fps_pos);
```

### Phase UI.3: 高度なコンポーネント (3-4日)

**目標**: 実用的なUI

```rust
// 実装項目:
11. Slider実装
12. Window/Panel実装
13. レイアウトシステム（基本）
14. テーマシステム

// デモ:
ui.window("Engine Controls", || {
    ui.slider("Camera Speed", &mut camera_speed, 0.1..10.0);
    ui.slider("FOV", &mut fov, 30.0..120.0);
    if ui.button("Add Sphere") {
        scene.add_object(ObjectType::Sphere, random_position());
    }
});
```

### Phase UI.4: エンジン連携 (2-3日)

**目標**: エンジン制御UI

```rust
// 実装項目:
15. カメラ制御UI
16. オブジェクト操作UI
17. 設定パネル
18. パフォーマンス監視UI

// デモ:
ui.window("Scene Control", || {
    ui.text("Camera Position:");
    ui.text(format!("  X: {:.2}", camera.position.x));
    ui.text(format!("  Y: {:.2}", camera.position.y));
    ui.text(format!("  Z: {:.2}", camera.position.z));
    
    if ui.button("Reset Position") {
        camera.reset();
    }
    
    ui.separator();
    
    ui.text("Performance:");
    ui.text(format!("FPS: {:.1}", metrics.get_fps()));
    ui.text(format!("Objects: {}", scene.object_count()));
    ui.text(format!("Frame Time: {:.2}ms", metrics.get_frame_time_ms()));
});
```

## 11. 技術的考慮事項

### 11.1 パフォーマンス最適化

```rust
// バッチング: 同一テクスチャ・スタイルの要素をまとめて描画
pub struct UIBatch {
    vertices: Vec<UIVertex>,
    indices: Vec<u16>,
    texture: Option<Arc<wgpu::Texture>>,
    style_hash: u64,
}

// インスタンシング: 大量の同一UI要素の効率描画
pub struct UIInstanceData {
    transform: [[f32; 4]; 4],  // 変換行列
    color: [f32; 4],           // カラー
    uv_offset: [f32; 2],       // テクスチャオフセット
}

// Dirty Rectangle: 変更領域のみ再描画
pub struct UIDirtyRegion {
    regions: Vec<UIRect>,
    is_full_redraw: bool,
}
```

### 11.2 メモリ管理

```rust
// UI要素のプール化
pub struct UIElementPool<T> {
    pool: Vec<T>,
    active: Vec<usize>,
    free: Vec<usize>,
}

// 文字列インターニング
pub struct UIStringCache {
    strings: HashMap<String, Arc<str>>,
    frame_usage: HashMap<Arc<str>, u64>,
}

// メッシュキャッシュ
pub struct UIMeshCache {
    text_meshes: HashMap<(String, f32, Color), Vec<UIVertex>>,
    primitive_meshes: HashMap<(PrimitiveType, UISize), Vec<UIVertex>>,
}
```

### 11.3 将来拡張

```rust
// アニメーション システム
pub struct UIAnimation {
    duration: f32,
    elapsed: f32,
    easing: EasingFunction,
    property: AnimatableProperty,
    start_value: f32,
    end_value: f32,
}

// レイアウト システム（Flexbox風）
pub struct FlexLayout {
    direction: FlexDirection,     // Row, Column
    justify_content: JustifyContent, // Start, Center, End, SpaceBetween
    align_items: AlignItems,      // Start, Center, End, Stretch
    wrap: FlexWrap,               // NoWrap, Wrap
}

// データバインディング
pub trait UIDataBinding<T> {
    fn bind(&mut self, data: &T);
    fn update(&mut self, data: &T) -> bool; // データ変更時にtrueを返す
}
```

## 12. 推奨実装順序

### **Phase UI.1から開始** (最小限デモ)

```rust
// 1日目: 基本UIテキスト表示
let mut ui_context = UIContext::new(screen_size);
ui_context.text(format!("FPS: {:.1}", engine_fps), UICoordinate::new(10.0, 10.0));

// 2日目: デバッグパネル
ui_context.panel("Debug Info", UIRect::new(10.0, 10.0, 250.0, 150.0), || {
    ui_context.text(format!("Objects: {}", object_count));
    ui_context.text(format!("Camera: {:?}", camera_pos));
    ui_context.text(format!("Frame Time: {:.2}ms", frame_time));
});
```

これにより、リファクタで培った堅牢な基盤の上に、視覚的にインパクトのあるUI機能を段階的に構築できます。

## 13. 期待される効果

### 開発効果
1. **デバッグ効率向上**: リアルタイム情報表示
2. **エンジン制御**: UIによる直感的な操作
3. **プロトタイピング**: 新機能の迅速な検証
4. **ユーザビリティ**: エンドユーザー向けインターフェース

### 技術的効果  
1. **2Dレンダリング経験**: WGPUの理解拡大
2. **イベントシステム**: 入力処理の高度化
3. **状態管理**: UIとエンジンの連携パターン
4. **モジュラー設計**: 拡張可能なアーキテクチャ

このUI基盤により、Demo Engineは本格的な開発ツールとしての機能を獲得します。