# エディター風UIシステム設計（タブ・ドッキング）

> **目的**: 将来的な高度なUI機能実装  
> **対象**: Visual Studio Code、Unity、Blender等と同等のエディターUI  
> **期間**: 2-3週間（段階的実装）  
> **難易度**: 高（複雑なUI状態管理）  
> **前提条件**: 基本UIシステム完了後

## 1. システム概要

### 1.1 目標機能

- **タブシステム**: 複数コンテンツの効率管理
- **ドラッグ&ドロップ**: タブ移動・結合・分離
- **分割レイアウト**: 自由な画面分割
- **フローティングウィンドウ**: 独立ウィンドウ化
- **レイアウト永続化**: 設定保存・復元

### 1.2 参考システム

- **Visual Studio Code**: パネル・タブ・分割
- **Unity Editor**: Scene/Game/Inspector/Hierarchy
- **Blender**: エリア分割・タブ結合
- **JetBrains IDE**: ツールウィンドウ・ドッキング

## 2. アーキテクチャ設計

### 2.1 核となる構造体

```rust
// ドッキングシステムの中核
pub struct DockSpace {
    pub root: DockNode,
    pub floating_windows: Vec<FloatingWindow>,
    pub active_tab: Option<TabId>,
    pub drag_state: DragState,
    pub layout_id: String,
}

// ノードベースの階層構造
pub enum DockNode {
    // リーフノード：実際のタブを含む
    Leaf {
        id: NodeId,
        tabs: Vec<Tab>,
        active_tab: usize,
        bounds: UIRect,
    },
    // 分割ノード：子ノードを含む
    Split {
        id: NodeId,
        direction: SplitDirection,
        ratio: f32,  // 0.0-1.0の分割比率
        left: Box<DockNode>,
        right: Box<DockNode>,
        bounds: UIRect,
        splitter_bounds: UIRect, // リサイズ用のスプリッター
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SplitDirection {
    Horizontal, // 左右分割
    Vertical,   // 上下分割
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(u32);
```

### 2.2 タブシステム

```rust
// タブの定義
pub struct Tab {
    pub id: TabId,
    pub title: String,
    pub content: Box<dyn TabContent>,
    pub closable: bool,
    pub icon: Option<Icon>,
    pub dirty: bool, // 未保存変更があるか
    pub pinned: bool, // ピン留め（閉じられない）
    pub context_menu: Option<Vec<ContextMenuItem>>,
}

// タブの内容を定義するトレイト
pub trait TabContent: Send + Sync {
    fn title(&self) -> &str;
    fn render(&mut self, ui: &mut UIContext, bounds: UIRect);
    fn handle_event(&mut self, event: &UIEvent) -> EventResponse;
    fn is_dirty(&self) -> bool { false }
    fn can_close(&self) -> bool { true }
    fn on_close(&mut self) -> bool { true } // falseで閉じるのを拒否
    fn on_focus(&mut self) {} // タブがアクティブになった時
    fn on_blur(&mut self) {} // タブが非アクティブになった時
    fn get_icon(&self) -> Option<Icon> { None }
    fn get_context_menu(&self) -> Vec<ContextMenuItem> { Vec::new() }
}

// タブに対するアクション
#[derive(Debug, Clone)]
pub enum TabAction {
    Activate(usize),
    Close(TabId),
    Pin(TabId),
    Unpin(TabId),
    New(Box<dyn TabContent>),
    Duplicate(TabId),
    MoveToNewWindow(TabId),
    ShowContextMenu(TabId, UICoordinate),
}
```

### 2.3 ドラッグ&ドロップシステム

```rust
#[derive(Debug, Clone)]
pub struct DragState {
    pub dragging_tab: Option<DraggedTab>,
    pub drag_position: UICoordinate,
    pub drop_target: Option<DropTarget>,
    pub preview_rect: Option<UIRect>, // ドロップ位置のプレビュー
    pub drag_offset: UICoordinate,
}

#[derive(Debug, Clone)]
pub struct DraggedTab {
    pub tab_id: TabId,
    pub source_node: NodeId,
    pub tab_index: usize,
    pub tab_bounds: UIRect,
    pub content_snapshot: Option<TextureId>, // ドラッグ中のプレビュー
}

#[derive(Debug, Clone)]
pub enum DropTarget {
    // 既存タブグループに追加
    TabGroup { 
        node_id: NodeId, 
        insert_index: usize,
        preview_rect: UIRect,
    },
    // 新しい分割を作成
    Split { 
        target_node: NodeId, 
        direction: SplitDirection, 
        ratio: f32,
        preview_rect: UIRect,
    },
    // フローティングウィンドウ化
    FloatingWindow { 
        position: UICoordinate,
        size: UISize,
    },
    // 既存のフローティングウィンドウに追加
    FloatingTabGroup {
        window_id: WindowId,
        insert_index: usize,
    },
}

impl DockSpace {
    pub fn handle_tab_drag(&mut self, event: &UIEvent) -> EventResponse {
        match event {
            UIEvent::MouseDown { pos, button: MouseButton::Left } => {
                if let Some((tab_id, node_id, tab_index)) = self.find_tab_at(*pos) {
                    let tab_bounds = self.get_tab_bounds(node_id, tab_index);
                    
                    self.drag_state.dragging_tab = Some(DraggedTab {
                        tab_id,
                        source_node: node_id,
                        tab_index,
                        tab_bounds,
                        content_snapshot: self.capture_tab_preview(tab_id),
                    });
                    
                    self.drag_state.drag_offset = *pos - tab_bounds.min;
                    EventResponse::Handled
                } else {
                    EventResponse::Ignored
                }
            }
            
            UIEvent::MouseMove { pos, .. } => {
                if let Some(ref drag) = self.drag_state.dragging_tab {
                    self.drag_state.drag_position = *pos;
                    self.drag_state.drop_target = self.find_drop_target(*pos, drag);
                    EventResponse::Handled
                } else {
                    EventResponse::Ignored
                }
            }
            
            UIEvent::MouseUp { pos, button: MouseButton::Left } => {
                if let Some(drag) = self.drag_state.dragging_tab.take() {
                    self.complete_tab_drag(drag, *pos);
                    self.drag_state.drop_target = None;
                    EventResponse::Handled
                } else {
                    EventResponse::Ignored
                }
            }
            
            _ => EventResponse::Ignored,
        }
    }
    
    fn complete_tab_drag(&mut self, drag: DraggedTab, drop_pos: UICoordinate) {
        if let Some(target) = &self.drag_state.drop_target {
            match target {
                DropTarget::TabGroup { node_id, insert_index, .. } => {
                    // タブを既存グループに移動
                    let tab = self.remove_tab(drag.source_node, drag.tab_index);
                    self.insert_tab(*node_id, *insert_index, tab);
                }
                
                DropTarget::Split { target_node, direction, ratio, .. } => {
                    // 新しい分割を作成
                    let tab = self.remove_tab(drag.source_node, drag.tab_index);
                    let new_leaf = DockNode::Leaf {
                        id: NodeId::generate(),
                        tabs: vec![tab],
                        active_tab: 0,
                        bounds: UIRect::ZERO, // レイアウト時に計算
                    };
                    
                    self.split_node(*target_node, *direction, *ratio, new_leaf);
                }
                
                DropTarget::FloatingWindow { position, size } => {
                    // フローティングウィンドウを作成
                    let tab = self.remove_tab(drag.source_node, drag.tab_index);
                    let window = FloatingWindow::new(*position, *size, vec![tab]);
                    self.floating_windows.push(window);
                }
                
                DropTarget::FloatingTabGroup { window_id, insert_index } => {
                    // 既存フローティングウィンドウに追加
                    let tab = self.remove_tab(drag.source_node, drag.tab_index);
                    if let Some(window) = self.floating_windows.iter_mut()
                        .find(|w| w.id == *window_id) {
                        window.insert_tab(*insert_index, tab);
                    }
                }
            }
            
            // レイアウト再計算
            self.recalculate_layout();
        }
    }
}
```

### 2.4 ドロップゾーン可視化

```rust
impl DockSpace {
    pub fn render_drop_zones(&mut self, ui: &mut UIContext) {
        if self.drag_state.dragging_tab.is_none() {
            return;
        }
        
        // ノードごとのドロップゾーン
        self.render_node_drop_zones(ui, &self.root);
        
        // フローティングウィンドウのドロップゾーン
        for window in &self.floating_windows {
            self.render_window_drop_zones(ui, window);
        }
        
        // 新しいフローティングウィンドウゾーン
        self.render_floating_zone(ui);
    }
    
    fn render_node_drop_zones(&mut self, ui: &mut UIContext, node: &DockNode) {
        match node {
            DockNode::Leaf { bounds, .. } => {
                let drop_zones = [
                    (SplitDirection::Vertical, bounds.top_strip(30.0), "Top"),
                    (SplitDirection::Vertical, bounds.bottom_strip(30.0), "Bottom"), 
                    (SplitDirection::Horizontal, bounds.left_strip(30.0), "Left"),
                    (SplitDirection::Horizontal, bounds.right_strip(30.0), "Right"),
                    (SplitDirection::Horizontal, bounds.center_rect(0.6), "Center"),
                ];
                
                for (direction, zone_rect, label) in drop_zones {
                    let distance = self.drag_state.drag_position.distance_to_rect(zone_rect);
                    
                    if distance < 50.0 { // 近接時のみ表示
                        let alpha = (1.0 - distance / 50.0).max(0.2);
                        
                        ui.draw_rect(zone_rect, &RectStyle {
                            fill_color: Color::rgba(0.0, 0.5, 1.0, alpha * 0.3),
                            border_color: Color::rgba(0.0, 0.5, 1.0, alpha),
                            border_width: 2.0,
                            border_radius: 4.0,
                        });
                        
                        // ドロップアイコン
                        let icon_pos = zone_rect.center();
                        ui.draw_icon(self.get_drop_icon(direction), icon_pos, 24.0, 
                                   Color::rgba(1.0, 1.0, 1.0, alpha));
                        
                        if alpha > 0.7 {
                            ui.text(label, icon_pos + UICoordinate::new(0.0, 20.0));
                        }
                    }
                }
            }
            
            DockNode::Split { left, right, .. } => {
                self.render_node_drop_zones(ui, left);
                self.render_node_drop_zones(ui, right);
            }
        }
    }
    
    fn get_drop_icon(&self, direction: SplitDirection) -> Icon {
        match direction {
            SplitDirection::Vertical => Icon::SplitVertical,
            SplitDirection::Horizontal => Icon::SplitHorizontal,
        }
    }
}
```

## 3. タブバー実装

### 3.1 基本タブバー

```rust
impl DockNode {
    pub fn render_tab_bar(&mut self, ui: &mut UIContext) -> Option<TabAction> {
        if let DockNode::Leaf { tabs, active_tab, bounds, .. } = self {
            let tab_bar_height = 32.0;
            let tab_bar_rect = UIRect::new(
                bounds.min.x, bounds.min.y,
                bounds.max.x, bounds.min.y + tab_bar_height
            );
            
            // タブバー背景
            ui.draw_rect(tab_bar_rect, &RectStyle::filled(ui.theme().surface));
            
            let available_width = tab_bar_rect.width() - 40.0; // 新規ボタン分を除く
            let tab_width = if tabs.is_empty() { 
                0.0 
            } else { 
                (available_width / tabs.len() as f32).min(150.0).max(80.0) 
            };
            
            let mut action = None;
            let mut total_width = 0.0;
            
            // スクロール可能タブバー
            let scroll_needed = tabs.len() as f32 * tab_width > available_width;
            let mut scroll_offset = if scroll_needed { 
                self.calculate_tab_scroll(*active_tab, tab_width, available_width) 
            } else { 
                0.0 
            };
            
            // スクロールボタン
            if scroll_needed {
                let scroll_left_rect = UIRect::new(
                    tab_bar_rect.min.x, tab_bar_rect.min.y,
                    tab_bar_rect.min.x + 20.0, tab_bar_rect.max.y
                );
                let scroll_right_rect = UIRect::new(
                    tab_bar_rect.max.x - 60.0, tab_bar_rect.min.y,
                    tab_bar_rect.max.x - 40.0, tab_bar_rect.max.y
                );
                
                if ui.button_icon("◀", scroll_left_rect) {
                    self.tab_scroll_offset = (self.tab_scroll_offset - tab_width).max(0.0);
                }
                
                if ui.button_icon("▶", scroll_right_rect) {
                    let max_offset = (tabs.len() as f32 * tab_width - available_width).max(0.0);
                    self.tab_scroll_offset = (self.tab_scroll_offset + tab_width).min(max_offset);
                }
            }
            
            // タブ描画
            for (i, tab) in tabs.iter().enumerate() {
                let tab_x = tab_bar_rect.min.x + i as f32 * tab_width - scroll_offset;
                let tab_rect = UIRect::new(
                    tab_x, tab_bar_rect.min.y,
                    tab_x + tab_width, tab_bar_rect.max.y
                );
                
                // 表示範囲外のタブをスキップ
                if tab_rect.max.x < tab_bar_rect.min.x || tab_rect.min.x > tab_bar_rect.max.x {
                    continue;
                }
                
                let is_active = i == *active_tab;
                let is_hovered = ui.is_hovered(tab_rect);
                let is_middle_clicked = ui.is_middle_clicked(tab_rect);
                
                // タブの描画
                self.render_single_tab(ui, tab, tab_rect, is_active, is_hovered);
                
                // タブインタラクション
                if is_middle_clicked && tab.closable {
                    action = Some(TabAction::Close(tab.id));
                } else if ui.is_clicked(tab_rect) {
                    action = Some(TabAction::Activate(i));
                } else if ui.is_right_clicked(tab_rect) {
                    action = Some(TabAction::ShowContextMenu(tab.id, ui.mouse_position()));
                }
                
                total_width += tab_width;
            }
            
            // 新しいタブボタン
            let new_tab_rect = UIRect::new(
                tab_bar_rect.max.x - 30.0, tab_bar_rect.min.y + 4.0,
                tab_bar_rect.max.x - 6.0, tab_bar_rect.max.y - 4.0
            );
            
            if ui.button_icon("+", new_tab_rect) {
                action = Some(TabAction::New(Box::new(EmptyTab::new())));
            }
            
            return action;
        }
        
        None
    }
    
    fn render_single_tab(&self, ui: &mut UIContext, tab: &Tab, rect: UIRect, 
                        is_active: bool, is_hovered: bool) {
        // タブ背景
        let bg_color = if is_active {
            ui.theme().background
        } else if is_hovered {
            ui.theme().surface.lighten(0.1)
        } else {
            ui.theme().surface
        };
        
        ui.draw_rect(rect, &RectStyle::filled(bg_color));
        
        // アクティブタブの上部ライン
        if is_active {
            let accent_line = UIRect::new(
                rect.min.x, rect.min.y,
                rect.max.x, rect.min.y + 2.0
            );
            ui.draw_rect(accent_line, &RectStyle::filled(ui.theme().primary));
        }
        
        // タブアイコン
        let mut content_x = rect.min.x + 8.0;
        if let Some(icon) = &tab.icon {
            ui.draw_icon(*icon, UICoordinate::new(content_x, rect.center().y), 16.0, 
                        ui.theme().text);
            content_x += 20.0;
        }
        
        // タブタイトル
        let title_rect = UIRect::new(
            content_x, rect.min.y,
            rect.max.x - if tab.closable { 25.0 } else { 8.0 }, rect.max.y
        );
        
        let mut display_title = tab.title.clone();
        if tab.dirty {
            display_title = format!("● {}", display_title);
        }
        
        ui.text_clipped(&display_title, title_rect);
        
        // ピン留めインジケーター
        if tab.pinned {
            let pin_pos = UICoordinate::new(rect.max.x - 35.0, rect.center().y);
            ui.draw_icon(Icon::Pin, pin_pos, 12.0, ui.theme().text_secondary);
        }
        
        // 閉じるボタン
        if tab.closable && !tab.pinned {
            let close_rect = UIRect::new(
                rect.max.x - 20.0, rect.min.y + 6.0,
                rect.max.x - 4.0, rect.max.y - 6.0
            );
            
            let close_color = if ui.is_hovered(close_rect) {
                Color::RED
            } else {
                ui.theme().text_secondary
            };
            
            ui.draw_icon(Icon::Close, close_rect.center(), 10.0, close_color);
        }
    }
}
```

### 3.2 タブコンテキストメニュー

```rust
#[derive(Debug, Clone)]
pub struct ContextMenuItem {
    pub label: String,
    pub action: ContextMenuAction,
    pub enabled: bool,
    pub separator_after: bool,
}

#[derive(Debug, Clone)]
pub enum ContextMenuAction {
    CloseTab,
    CloseOthers,
    CloseAll,
    CloseToRight,
    Pin,
    Unpin,
    Duplicate,
    MoveToNewWindow,
    Custom(String), // カスタムアクション
}

impl Tab {
    pub fn get_default_context_menu(&self) -> Vec<ContextMenuItem> {
        vec![
            ContextMenuItem {
                label: "Close".to_string(),
                action: ContextMenuAction::CloseTab,
                enabled: self.closable,
                separator_after: false,
            },
            ContextMenuItem {
                label: "Close Others".to_string(),
                action: ContextMenuAction::CloseOthers,
                enabled: true,
                separator_after: false,
            },
            ContextMenuItem {
                label: "Close All".to_string(),
                action: ContextMenuAction::CloseAll,
                enabled: true,
                separator_after: true,
            },
            ContextMenuItem {
                label: if self.pinned { "Unpin" } else { "Pin" }.to_string(),
                action: if self.pinned { ContextMenuAction::Unpin } else { ContextMenuAction::Pin },
                enabled: true,
                separator_after: false,
            },
            ContextMenuItem {
                label: "Duplicate".to_string(),
                action: ContextMenuAction::Duplicate,
                enabled: true,
                separator_after: false,
            },
            ContextMenuItem {
                label: "Move to New Window".to_string(),
                action: ContextMenuAction::MoveToNewWindow,
                enabled: true,
                separator_after: false,
            },
        ]
    }
}
```

## 4. 実用的なタブ実装例

### 4.1 3Dシーンビュータブ

```rust
pub struct SceneViewTab {
    camera_controls: CameraControls,
    render_settings: RenderSettings,
    gizmo_mode: GizmoMode,
    show_grid: bool,
    show_wireframe: bool,
    selected_objects: HashSet<ObjectId>,
}

impl TabContent for SceneViewTab {
    fn title(&self) -> &str { "Scene View" }
    
    fn render(&mut self, ui: &mut UIContext, bounds: UIRect) {
        // ツールバー
        let toolbar_height = 40.0;
        let toolbar_rect = UIRect::new(
            bounds.min.x, bounds.min.y,
            bounds.max.x, bounds.min.y + toolbar_height
        );
        
        self.render_scene_toolbar(ui, toolbar_rect);
        
        // 3Dビューポート
        let viewport_rect = UIRect::new(
            bounds.min.x, bounds.min.y + toolbar_height,
            bounds.max.x, bounds.max.y
        );
        
        ui.custom_3d_viewport(viewport_rect, |viewport| {
            // 3Dシーンの描画
            viewport.render_scene(&self.camera_controls);
            
            // グリッド描画
            if self.show_grid {
                viewport.render_grid();
            }
            
            // ギズモ描画
            if !self.selected_objects.is_empty() {
                viewport.render_gizmo(&self.selected_objects, self.gizmo_mode);
            }
        });
        
        // 統計情報オーバーレイ
        self.render_stats_overlay(ui, viewport_rect);
    }
    
    fn handle_event(&mut self, event: &UIEvent) -> EventResponse {
        // カメラ制御
        if let response = self.camera_controls.handle_event(event) {
            if response == EventResponse::Handled {
                return response;
            }
        }
        
        // オブジェクト選択
        match event {
            UIEvent::MouseDown { pos, button: MouseButton::Left } => {
                if let Some(object_id) = self.pick_object_at(*pos) {
                    if !ui.is_key_pressed(KeyCode::ControlLeft) {
                        self.selected_objects.clear();
                    }
                    self.selected_objects.insert(object_id);
                    EventResponse::Handled
                } else {
                    self.selected_objects.clear();
                    EventResponse::Handled
                }
            }
            _ => EventResponse::Ignored,
        }
    }
    
    fn get_icon(&self) -> Option<Icon> {
        Some(Icon::Scene)
    }
    
    fn get_context_menu(&self) -> Vec<ContextMenuItem> {
        let mut menu = self.get_default_context_menu();
        
        // シーン特有のメニュー項目
        menu.extend(vec![
            ContextMenuItem {
                label: "Reset Camera".to_string(),
                action: ContextMenuAction::Custom("reset_camera".to_string()),
                enabled: true,
                separator_after: false,
            },
            ContextMenuItem {
                label: "Frame Selection".to_string(),
                action: ContextMenuAction::Custom("frame_selection".to_string()),
                enabled: !self.selected_objects.is_empty(),
                separator_after: true,
            },
        ]);
        
        menu
    }
}

impl SceneViewTab {
    fn render_scene_toolbar(&mut self, ui: &mut UIContext, rect: UIRect) {
        ui.horizontal_layout(rect, |ui| {
            // ギズモモード選択
            if ui.toggle_button("Move", self.gizmo_mode == GizmoMode::Translate) {
                self.gizmo_mode = GizmoMode::Translate;
            }
            if ui.toggle_button("Rotate", self.gizmo_mode == GizmoMode::Rotate) {
                self.gizmo_mode = GizmoMode::Rotate;
            }
            if ui.toggle_button("Scale", self.gizmo_mode == GizmoMode::Scale) {
                self.gizmo_mode = GizmoMode::Scale;
            }
            
            ui.separator_vertical();
            
            // 表示オプション
            ui.checkbox("Grid", &mut self.show_grid);
            ui.checkbox("Wireframe", &mut self.show_wireframe);
            
            ui.separator_vertical();
            
            // カメラプリセット
            if ui.button("Front") {
                self.camera_controls.set_view(CameraView::Front);
            }
            if ui.button("Top") {
                self.camera_controls.set_view(CameraView::Top);
            }
            if ui.button("Right") {
                self.camera_controls.set_view(CameraView::Right);
            }
        });
    }
    
    fn render_stats_overlay(&self, ui: &mut UIContext, rect: UIRect) {
        let overlay_rect = UIRect::new(
            rect.min.x + 10.0, rect.max.y - 80.0,
            rect.min.x + 200.0, rect.max.y - 10.0
        );
        
        ui.with_style(overlay_style(), || {
            ui.panel_transparent("Stats", overlay_rect, || {
                ui.text(format!("Objects: {}", self.get_object_count()));
                ui.text(format!("Triangles: {}", self.get_triangle_count()));
                ui.text(format!("FPS: {:.1}", ui.get_fps()));
            });
        });
    }
}
```

### 4.2 インスペクタータブ

```rust
pub struct InspectorTab {
    selected_objects: Vec<ObjectId>,
    property_editors: HashMap<String, Box<dyn PropertyEditor>>,
    expanded_sections: HashSet<String>,
}

impl TabContent for InspectorTab {
    fn title(&self) -> &str { "Inspector" }
    
    fn render(&mut self, ui: &mut UIContext, bounds: UIRect) {
        if self.selected_objects.is_empty() {
            ui.centered_text("No object selected", bounds);
            return;
        }
        
        ui.scrollable_area(bounds, |ui| {
            if self.selected_objects.len() == 1 {
                // 単一オブジェクトの詳細
                self.render_single_object_inspector(ui, self.selected_objects[0]);
            } else {
                // 複数オブジェクトの共通プロパティ
                self.render_multi_object_inspector(ui);
            }
        });
    }
    
    fn handle_event(&mut self, event: &UIEvent) -> EventResponse {
        // 選択変更の監視
        if let UIEvent::SelectionChanged { objects } = event {
            self.selected_objects = objects.clone();
            self.update_property_editors();
            return EventResponse::Handled;
        }
        
        EventResponse::Ignored
    }
    
    fn get_icon(&self) -> Option<Icon> {
        Some(Icon::Inspector)
    }
}

impl InspectorTab {
    fn render_single_object_inspector(&mut self, ui: &mut UIContext, object_id: ObjectId) {
        let object = ui.get_object(object_id).unwrap();
        
        // オブジェクト名
        ui.section("Object", true, || {
            ui.text_input(&mut object.name, "Name");
            ui.checkbox("Visible", &mut object.visible);
            ui.enum_combo("Layer", &mut object.layer);
        });
        
        // Transform
        if ui.section_header("Transform", self.is_expanded("transform")) {
            ui.indent(|| {
                ui.vec3_input("Position", &mut object.transform.position);
                ui.vec3_input("Rotation", &mut object.transform.rotation_euler());
                ui.vec3_input("Scale", &mut object.transform.scale);
                
                if ui.button("Reset Transform") {
                    object.transform = Transform::identity();
                }
            });
        }
        
        // コンポーネント
        for (component_name, editor) in &mut self.property_editors {
            if ui.section_header(component_name, self.is_expanded(component_name)) {
                ui.indent(|| {
                    editor.render(ui, object_id);
                });
            }
        }
        
        // コンポーネント追加
        ui.separator();
        if ui.button("Add Component") {
            ui.show_component_menu(object_id);
        }
    }
    
    fn render_multi_object_inspector(&mut self, ui: &mut UIContext) {
        ui.text(format!("{} objects selected", self.selected_objects.len()));
        ui.separator();
        
        // 共通プロパティのみ表示
        ui.section("Common Properties", true, || {
            // 全選択オブジェクトに共通する変更可能プロパティ
            let mut common_visible = self.get_common_visible();
            if ui.checkbox("Visible", &mut common_visible) {
                self.set_all_visible(common_visible);
            }
            
            let mut common_layer = self.get_common_layer();
            if let Some(ref mut layer) = common_layer {
                if ui.enum_combo("Layer", layer) {
                    self.set_all_layer(*layer);
                }
            }
        });
    }
    
    fn is_expanded(&self, section: &str) -> bool {
        self.expanded_sections.contains(section)
    }
}
```

### 4.3 コンソールタブ

```rust
pub struct ConsoleTab {
    messages: Vec<LogMessage>,
    input_buffer: String,
    auto_scroll: bool,
    filter_level: LogLevel,
    search_term: String,
    command_history: Vec<String>,
    history_index: usize,
}

impl TabContent for ConsoleTab {
    fn title(&self) -> &str { 
        if self.has_errors() { "Console (!)" } else { "Console" }
    }
    
    fn render(&mut self, ui: &mut UIContext, bounds: UIRect) {
        let toolbar_height = 30.0;
        let input_height = 30.0;
        
        // ツールバー
        let toolbar_rect = UIRect::new(
            bounds.min.x, bounds.min.y,
            bounds.max.x, bounds.min.y + toolbar_height
        );
        self.render_console_toolbar(ui, toolbar_rect);
        
        // ログ表示エリア
        let log_bounds = UIRect::new(
            bounds.min.x, bounds.min.y + toolbar_height,
            bounds.max.x, bounds.max.y - input_height
        );
        
        ui.scrollable_area(log_bounds, |ui| {
            let filtered_messages = self.get_filtered_messages();
            
            for message in filtered_messages {
                let color = self.get_message_color(message.level);
                let timestamp = message.timestamp.format("%H:%M:%S");
                
                ui.text_colored(
                    &format!("[{}] {}", timestamp, message.text), 
                    color
                );
            }
            
            if self.auto_scroll {
                ui.scroll_to_bottom();
            }
        });
        
        // コマンド入力
        let input_bounds = UIRect::new(
            bounds.min.x, bounds.max.y - input_height,
            bounds.max.x, bounds.max.y
        );
        
        if ui.text_input(&mut self.input_buffer, input_bounds, "Enter command...") {
            self.execute_command();
        }
    }
    
    fn handle_event(&mut self, event: &UIEvent) -> EventResponse {
        match event {
            UIEvent::KeyPressed { key: KeyCode::Enter, .. } => {
                self.execute_command();
                EventResponse::Handled
            }
            UIEvent::KeyPressed { key: KeyCode::ArrowUp, .. } => {
                self.navigate_history(-1);
                EventResponse::Handled
            }
            UIEvent::KeyPressed { key: KeyCode::ArrowDown, .. } => {
                self.navigate_history(1);
                EventResponse::Handled
            }
            _ => EventResponse::Ignored,
        }
    }
    
    fn is_dirty(&self) -> bool {
        self.has_unread_errors()
    }
    
    fn get_icon(&self) -> Option<Icon> {
        if self.has_errors() {
            Some(Icon::ConsoleError)
        } else {
            Some(Icon::Console)
        }
    }
}

impl ConsoleTab {
    fn render_console_toolbar(&mut self, ui: &mut UIContext, rect: UIRect) {
        ui.horizontal_layout(rect, |ui| {
            // 消去ボタン
            if ui.button("Clear") {
                self.messages.clear();
            }
            
            ui.separator_vertical();
            
            // フィルターレベル
            ui.label("Filter:");
            ui.enum_combo("", &mut self.filter_level);
            
            ui.separator_vertical();
            
            // 自動スクロール
            ui.checkbox("Auto-scroll", &mut self.auto_scroll);
            
            ui.separator_vertical();
            
            // 検索
            ui.text_input(&mut self.search_term, "Search...");
        });
    }
    
    fn execute_command(&mut self) {
        if self.input_buffer.trim().is_empty() {
            return;
        }
        
        // コマンド履歴に追加
        self.command_history.push(self.input_buffer.clone());
        self.history_index = self.command_history.len();
        
        // コマンド実行（簡単な例）
        let result = match self.input_buffer.trim() {
            "clear" => {
                self.messages.clear();
                "Console cleared.".to_string()
            }
            cmd if cmd.starts_with("echo ") => {
                cmd[5..].to_string()
            }
            cmd if cmd.starts_with("fps") => {
                format!("Current FPS: {:.1}", ui.get_fps())
            }
            cmd => {
                format!("Unknown command: {}", cmd)
            }
        };
        
        // 結果をログに追加
        self.messages.push(LogMessage {
            text: format!("> {}", self.input_buffer),
            level: LogLevel::Info,
            timestamp: Instant::now(),
        });
        
        if !result.is_empty() {
            self.messages.push(LogMessage {
                text: result,
                level: LogLevel::Info,
                timestamp: Instant::now(),
            });
        }
        
        self.input_buffer.clear();
    }
}
```

## 5. フローティングウィンドウ

### 5.1 フローティングウィンドウの実装

```rust
#[derive(Debug, Clone)]
pub struct FloatingWindow {
    pub id: WindowId,
    pub title: String,
    pub position: UICoordinate,
    pub size: UISize,
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub resizable: bool,
    pub minimized: bool,
    pub always_on_top: bool,
    pub drag_state: WindowDragState,
}

#[derive(Debug, Clone)]
pub enum WindowDragState {
    None,
    Moving { offset: UICoordinate },
    Resizing { edge: ResizeEdge, start_pos: UICoordinate, start_size: UISize },
}

#[derive(Debug, Clone, Copy)]
pub enum ResizeEdge {
    Top, Bottom, Left, Right,
    TopLeft, TopRight, BottomLeft, BottomRight,
}

impl FloatingWindow {
    pub fn new(position: UICoordinate, size: UISize, tabs: Vec<Tab>) -> Self {
        Self {
            id: WindowId::generate(),
            title: if tabs.len() == 1 { 
                tabs[0].title.clone() 
            } else { 
                format!("{} tabs", tabs.len()) 
            },
            position,
            size,
            tabs,
            active_tab: 0,
            resizable: true,
            minimized: false,
            always_on_top: false,
            drag_state: WindowDragState::None,
        }
    }
    
    pub fn render(&mut self, ui: &mut UIContext) -> Option<WindowAction> {
        if self.minimized {
            return self.render_minimized(ui);
        }
        
        let window_rect = UIRect::new(
            self.position.x, self.position.y,
            self.position.x + self.size.width,
            self.position.y + self.size.height
        );
        
        // ウィンドウの影
        let shadow_rect = window_rect.offset(UICoordinate::new(2.0, 2.0));
        ui.draw_rect(shadow_rect, &RectStyle::filled(Color::rgba(0.0, 0.0, 0.0, 0.3)));
        
        // ウィンドウ背景
        ui.draw_rect(window_rect, &RectStyle {
            fill_color: ui.theme().surface,
            border_color: ui.theme().border,
            border_width: 1.0,
            border_radius: 4.0,
        });
        
        // タイトルバー
        let title_bar_height = 30.0;
        let title_bar_rect = UIRect::new(
            window_rect.min.x, window_rect.min.y,
            window_rect.max.x, window_rect.min.y + title_bar_height
        );
        
        let title_action = self.render_title_bar(ui, title_bar_rect);
        if title_action.is_some() {
            return title_action;
        }
        
        // タブバー（複数タブの場合）
        let tab_bar_height = if self.tabs.len() > 1 { 32.0 } else { 0.0 };
        let content_y = window_rect.min.y + title_bar_height + tab_bar_height;
        
        if self.tabs.len() > 1 {
            let tab_bar_rect = UIRect::new(
                window_rect.min.x, window_rect.min.y + title_bar_height,
                window_rect.max.x, content_y
            );
            
            if let Some(tab_action) = self.render_tab_bar(ui, tab_bar_rect) {
                return Some(WindowAction::TabAction(tab_action));
            }
        }
        
        // コンテンツエリア
        let content_rect = UIRect::new(
            window_rect.min.x + 1.0, content_y,
            window_rect.max.x - 1.0, window_rect.max.y - 1.0
        );
        
        if let Some(active_tab) = self.tabs.get_mut(self.active_tab) {
            active_tab.content.render(ui, content_rect);
        }
        
        // リサイズハンドル
        if self.resizable {
            self.render_resize_handles(ui, window_rect);
        }
        
        None
    }
    
    fn render_title_bar(&mut self, ui: &mut UIContext, rect: UIRect) -> Option<WindowAction> {
        // タイトルバー背景
        ui.draw_rect(rect, &RectStyle::filled(ui.theme().primary.darken(0.3)));
        
        // タイトル
        let title_rect = UIRect::new(
            rect.min.x + 8.0, rect.min.y,
            rect.max.x - 60.0, rect.max.y
        );
        ui.text_clipped(&self.title, title_rect);
        
        // ウィンドウコントロールボタン
        let button_size = 20.0;
        let button_y = rect.min.y + (rect.height() - button_size) / 2.0;
        
        // 最小化ボタン
        let minimize_rect = UIRect::new(
            rect.max.x - 60.0, button_y,
            rect.max.x - 40.0, button_y + button_size
        );
        if ui.button_icon("−", minimize_rect) {
            return Some(WindowAction::Minimize);
        }
        
        // 最大化/復元ボタン
        let maximize_rect = UIRect::new(
            rect.max.x - 40.0, button_y,
            rect.max.x - 20.0, button_y + button_size
        );
        if ui.button_icon("□", maximize_rect) {
            return Some(WindowAction::ToggleMaximize);
        }
        
        // 閉じるボタン
        let close_rect = UIRect::new(
            rect.max.x - 20.0, button_y,
            rect.max.x, button_y + button_size
        );
        if ui.button_icon("×", close_rect) {
            return Some(WindowAction::Close);
        }
        
        // タイトルバードラッグ
        if ui.is_dragging(rect) {
            return Some(WindowAction::StartDrag);
        }
        
        None
    }
    
    fn render_resize_handles(&mut self, ui: &mut UIContext, rect: UIRect) {
        let handle_size = 8.0;
        
        let handles = [
            (ResizeEdge::TopLeft, UIRect::new(
                rect.min.x, rect.min.y,
                rect.min.x + handle_size, rect.min.y + handle_size
            )),
            (ResizeEdge::TopRight, UIRect::new(
                rect.max.x - handle_size, rect.min.y,
                rect.max.x, rect.min.y + handle_size
            )),
            (ResizeEdge::BottomLeft, UIRect::new(
                rect.min.x, rect.max.y - handle_size,
                rect.min.x + handle_size, rect.max.y
            )),
            (ResizeEdge::BottomRight, UIRect::new(
                rect.max.x - handle_size, rect.max.y - handle_size,
                rect.max.x, rect.max.y
            )),
        ];
        
        for (edge, handle_rect) in handles {
            if ui.is_hovered(handle_rect) {
                ui.set_cursor(self.get_resize_cursor(edge));
                
                // ハンドル可視化
                ui.draw_rect(handle_rect, &RectStyle::filled(
                    ui.theme().primary.with_alpha(0.5)
                ));
            }
        }
    }
    
    fn get_resize_cursor(&self, edge: ResizeEdge) -> CursorIcon {
        match edge {
            ResizeEdge::Top | ResizeEdge::Bottom => CursorIcon::ResizeVertical,
            ResizeEdge::Left | ResizeEdge::Right => CursorIcon::ResizeHorizontal,
            ResizeEdge::TopLeft | ResizeEdge::BottomRight => CursorIcon::ResizeDiagonal1,
            ResizeEdge::TopRight | ResizeEdge::BottomLeft => CursorIcon::ResizeDiagonal2,
        }
    }
}

#[derive(Debug, Clone)]
pub enum WindowAction {
    Close,
    Minimize,
    ToggleMaximize,
    StartDrag,
    StartResize(ResizeEdge),
    TabAction(TabAction),
}
```

## 6. レイアウト永続化

### 6.1 シリアライゼーション

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct LayoutConfig {
    pub version: u32,
    pub root: SerializableNode,
    pub floating_windows: Vec<SerializableWindow>,
    pub active_layout: String,
    pub saved_layouts: HashMap<String, SavedLayout>,
}

#[derive(Serialize, Deserialize)]
pub enum SerializableNode {
    Leaf {
        tabs: Vec<TabConfig>,
        active_tab: usize,
        bounds: SerializableRect,
    },
    Split {
        direction: SplitDirection,
        ratio: f32,
        left: Box<SerializableNode>,
        right: Box<SerializableNode>,
        bounds: SerializableRect,
    },
}

#[derive(Serialize, Deserialize)]
pub struct TabConfig {
    pub tab_type: String, // "SceneView", "Inspector", etc.
    pub title: String,
    pub closable: bool,
    pub pinned: bool,
    pub custom_data: HashMap<String, serde_json::Value>, // タブ固有の設定
}

#[derive(Serialize, Deserialize)]
pub struct SerializableWindow {
    pub position: SerializableCoordinate,
    pub size: SerializableSize,
    pub tabs: Vec<TabConfig>,
    pub active_tab: usize,
    pub minimized: bool,
    pub always_on_top: bool,
}

#[derive(Serialize, Deserialize)]
pub struct SavedLayout {
    pub name: String,
    pub description: String,
    pub root: SerializableNode,
    pub floating_windows: Vec<SerializableWindow>,
    pub created_at: String,
}

impl DockSpace {
    pub fn save_layout(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let config = LayoutConfig {
            version: 1,
            root: self.root.to_serializable(),
            floating_windows: self.floating_windows.iter()
                .map(|w| w.to_serializable())
                .collect(),
            active_layout: "default".to_string(),
            saved_layouts: HashMap::new(),
        };
        
        let json = serde_json::to_string_pretty(&config)?;
        std::fs::write(path, json)?;
        
        log::info!("Layout saved to {}", path);
        Ok(())
    }
    
    pub fn load_layout(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let json = std::fs::read_to_string(path)?;
        let config: LayoutConfig = serde_json::from_str(&json)?;
        
        // バージョンチェック
        if config.version > 1 {
            return Err("Unsupported layout version".into());
        }
        
        // ノード復元
        self.root = DockNode::from_serializable(
            config.root, 
            &self.tab_factory
        )?;
        
        // フローティングウィンドウ復元
        self.floating_windows = config.floating_windows.into_iter()
            .map(|w| FloatingWindow::from_serializable(w, &self.tab_factory))
            .collect::<Result<Vec<_>, _>>()?;
        
        // レイアウト再計算
        self.recalculate_layout();
        
        log::info!("Layout loaded from {}", path);
        Ok(())
    }
    
    pub fn save_layout_preset(&mut self, name: String, description: String) {
        let preset = SavedLayout {
            name: name.clone(),
            description,
            root: self.root.to_serializable(),
            floating_windows: self.floating_windows.iter()
                .map(|w| w.to_serializable())
                .collect(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        self.saved_layouts.insert(name, preset);
    }
    
    pub fn load_layout_preset(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
        if let Some(preset) = self.saved_layouts.get(name) {
            self.root = DockNode::from_serializable(
                preset.root.clone(),
                &self.tab_factory
            )?;
            
            self.floating_windows = preset.floating_windows.iter()
                .map(|w| FloatingWindow::from_serializable(w.clone(), &self.tab_factory))
                .collect::<Result<Vec<_>, _>>()?;
            
            self.recalculate_layout();
            Ok(())
        } else {
            Err(format!("Layout preset '{}' not found", name).into())
        }
    }
}
```

### 6.2 タブファクトリーシステム

```rust
// タブの動的生成
pub struct TabFactory {
    creators: HashMap<String, Box<dyn Fn(TabConfig) -> Box<dyn TabContent>>>,
}

impl TabFactory {
    pub fn new() -> Self {
        let mut factory = Self {
            creators: HashMap::new(),
        };
        
        // 標準タブタイプの登録
        factory.register("SceneView", |config| {
            Box::new(SceneViewTab::from_config(config))
        });
        
        factory.register("Inspector", |config| {
            Box::new(InspectorTab::from_config(config))
        });
        
        factory.register("Console", |config| {
            Box::new(ConsoleTab::from_config(config))
        });
        
        factory.register("Hierarchy", |config| {
            Box::new(HierarchyTab::from_config(config))
        });
        
        factory.register("Assets", |config| {
            Box::new(AssetsTab::from_config(config))
        });
        
        factory
    }
    
    pub fn register<F>(&mut self, tab_type: &str, creator: F) 
    where F: Fn(TabConfig) -> Box<dyn TabContent> + 'static {
        self.creators.insert(tab_type.to_string(), Box::new(creator));
    }
    
    pub fn create_tab(&self, config: TabConfig) -> Result<Tab, Box<dyn Error>> {
        if let Some(creator) = self.creators.get(&config.tab_type) {
            let content = creator(config.clone());
            
            Ok(Tab {
                id: TabId::generate(),
                title: config.title,
                content,
                closable: config.closable,
                icon: None,
                dirty: false,
                pinned: config.pinned,
                context_menu: None,
            })
        } else {
            Err(format!("Unknown tab type: {}", config.tab_type).into())
        }
    }
}

// タブコンテンツの設定復元
pub trait ConfigurableTab {
    fn from_config(config: TabConfig) -> Self where Self: Sized;
    fn to_config(&self) -> TabConfig;
}

impl ConfigurableTab for SceneViewTab {
    fn from_config(config: TabConfig) -> Self {
        let mut tab = SceneViewTab::new();
        
        // カスタムデータから設定復元
        if let Some(show_grid) = config.custom_data.get("show_grid") {
            tab.show_grid = show_grid.as_bool().unwrap_or(true);
        }
        
        if let Some(gizmo_mode) = config.custom_data.get("gizmo_mode") {
            if let Some(mode_str) = gizmo_mode.as_str() {
                tab.gizmo_mode = match mode_str {
                    "translate" => GizmoMode::Translate,
                    "rotate" => GizmoMode::Rotate,
                    "scale" => GizmoMode::Scale,
                    _ => GizmoMode::Translate,
                };
            }
        }
        
        tab
    }
    
    fn to_config(&self) -> TabConfig {
        let mut custom_data = HashMap::new();
        custom_data.insert("show_grid".to_string(), 
                          serde_json::Value::Bool(self.show_grid));
        custom_data.insert("gizmo_mode".to_string(), 
                          serde_json::Value::String(match self.gizmo_mode {
                              GizmoMode::Translate => "translate".to_string(),
                              GizmoMode::Rotate => "rotate".to_string(),
                              GizmoMode::Scale => "scale".to_string(),
                          }));
        
        TabConfig {
            tab_type: "SceneView".to_string(),
            title: self.title().to_string(),
            closable: true,
            pinned: false,
            custom_data,
        }
    }
}
```

## 7. 実装段階

### Phase Editor.1: 基本タブシステム (3-4日)
```rust
1. Tab trait + TabContent trait実装
2. 基本的なタブバーレンダリング
3. タブ切り替え・閉じる機能
4. 3-4種類の基本タブ実装（SceneView, Inspector, Console）

目標: 単純なタブ切り替えができるUI
```

### Phase Editor.2: ドッキングシステム (4-5日)
```rust
5. DockNode階層構造実装
6. 基本的な分割レイアウト（固定比率）
7. タブドラッグ&ドロップ基本機能
8. ドロップゾーン可視化

目標: タブの移動・結合ができるシステム
```

### Phase Editor.3: 高度なドッキング (3-4日)
```rust
9. 分割比率のリサイズ機能
10. フローティングウィンドウ
11. ドロップゾーン詳細化（上下左右中央）
12. タブピン留め・コンテキストメニュー

目標: プロレベルのドッキングシステム
```

### Phase Editor.4: 永続化・最適化 (2-3日)
```rust
13. レイアウト保存・読み込み
14. タブファクトリーシステム
15. パフォーマンス最適化
16. レイアウトプリセット機能

目標: 実用的なエディターUI完成
```

## 8. 期待される効果

### 開発効果
1. **ワークフロー革命**: Visual Studio Code級の開発環境
2. **生産性向上**: 情報の同時表示・効率的な作業空間
3. **カスタマイズ性**: ユーザー個別の最適レイアウト
4. **プロフェッショナル**: 商用エディター級のUX

### 技術的効果
1. **UI状態管理**: 複雑な状態を扱う経験
2. **イベント処理**: 高度なインタラクション実装
3. **データ永続化**: 設定・レイアウトの保存技術
4. **モジュラー設計**: 拡張可能なアーキテクチャ

### エンジン価値向上
1. **開発ツール**: プロレベルのエディター機能
2. **差別化**: 他の3Dエンジンとの明確な差別化
3. **エコシステム**: プラグイン・カスタマイズ基盤
4. **商用レベル**: 実際のプロダクション使用可能

## 9. 結論

このエディター風UIシステムにより、Demo Engineは**単なる学習プロジェクトから、実用的な3D開発環境**へと進化します。

- **Visual Studio Code級**: プロ仕様のタブ・ドッキング
- **Unity Editor級**: 3D開発に特化した専門UI
- **Blender級**: 柔軟なワークスペース構成
- **完全カスタマイズ**: ユーザーごとの最適化

将来的な実装として、このシステムによりDemo Engineの価値と実用性が大幅に向上することが期待されます。