# Phase 3: 長期改善

> **目標**: 本格的な 3D エンジンへの発展  
> **期間**: 1-3ヶ月  
> **リスク**: 高  
> **前提条件**: Phase 1, 2 完了

## 1. アーキテクチャの完全再設計

### 1.1 Entity Component System (ECS) 導入

**優先度**: 🔴 高

**現在の問題**:
- オブジェクト階層の欠如
- スケーラビリティの限界
- コンポーネントの再利用性が低い

**ECS 設計**:
```rust
// src/ecs/mod.rs (新規作成)
pub mod entity;
pub mod component;
pub mod system;
pub mod world;

// src/ecs/entity.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(u32);

pub struct EntityManager {
    next_id: u32,
    free_entities: Vec<Entity>,
    generations: Vec<u32>,
}

// src/ecs/component.rs
pub trait Component: 'static + Send + Sync {}

#[derive(Component)]
pub struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

#[derive(Component)]
pub struct MeshRenderer {
    pub mesh_id: ResourceId,
    pub material_id: ResourceId,
}

#[derive(Component)]  
pub struct Camera {
    pub projection: CameraProjection,
    pub view: CameraView,
}

// src/ecs/system.rs
pub trait System {
    fn update(&mut self, world: &mut World, dt: f32);
}

pub struct RenderSystem {
    graphics_engine: GraphicsEngine,
}

impl System for RenderSystem {
    fn update(&mut self, world: &mut World, dt: f32) {
        // クエリでTransform + MeshRendererを持つEntityを取得
        for (entity, (transform, mesh_renderer)) in world.query::<(Transform, MeshRenderer)>() {
            // レンダリング処理
        }
    }
}

// src/ecs/world.rs
pub struct World {
    entities: EntityManager,
    components: HashMap<TypeId, Box<dyn ComponentStorage>>,
    systems: Vec<Box<dyn System>>,
}
```

**実装ステップ**:
1. 基本的な ECS フレームワーク実装
2. 既存のレンダリングシステムを ECS に移行
3. Transform、MeshRenderer コンポーネント実装
4. System trait とRenderSystem実装
5. 段階的移行とテスト

**期待効果**: 高いスケーラビリティ、コンポーネント再利用性、並列処理対応

### 1.2 マルチスレッドレンダリング

**優先度**: 🟡 中

**実装内容**:
```rust
// src/graphics/render_thread.rs (新規作成)
use std::sync::mpsc;
use std::thread;

pub enum RenderCommand {
    DrawMesh { mesh_id: ResourceId, transform: glam::Mat4 },
    SetCamera { view_proj: glam::Mat4 },
    Clear { color: wgpu::Color },
    Present,
}

pub struct RenderThread {
    command_sender: mpsc::Sender<RenderCommand>,
    thread_handle: thread::JoinHandle<()>,
}

impl RenderThread {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let (tx, rx) = mpsc::channel();
        
        let handle = thread::spawn(move || {
            let mut renderer = ThreadRenderer::new(device, queue);
            
            while let Ok(command) = rx.recv() {
                match command {
                    RenderCommand::DrawMesh { mesh_id, transform } => {
                        renderer.draw_mesh(mesh_id, transform);
                    },
                    RenderCommand::Present => {
                        renderer.present();
                        break;
                    }
                    // ... other commands
                }
            }
        });
        
        Self {
            command_sender: tx,
            thread_handle: handle,
        }
    }
}
```

## 2. マテリアルシステム

### 2.1 PBR (Physically Based Rendering) 実装

**優先度**: 🟡 中

**実装内容**:
```rust  
// src/materials/mod.rs (新規作成)
pub mod pbr;
pub mod material;
pub mod shader_compiler;

// src/materials/material.rs
#[derive(Debug, Clone)]
pub struct Material {
    pub name: String,
    pub shader_id: ResourceId,
    pub textures: HashMap<String, ResourceId>,
    pub uniforms: MaterialUniforms,
}

#[derive(Debug, Clone)]  
pub struct MaterialUniforms {
    pub albedo: glam::Vec3,
    pub metallic: f32,
    pub roughness: f32,
    pub normal_scale: f32,
    pub emission: glam::Vec3,
}

// src/materials/pbr.rs
pub struct PbrMaterial {
    pub albedo_texture: Option<ResourceId>,
    pub normal_texture: Option<ResourceId>,
    pub metallic_roughness_texture: Option<ResourceId>,
    pub emission_texture: Option<ResourceId>,
    pub parameters: PbrParameters,
}

#[derive(Debug, Clone)]
pub struct PbrParameters {
    pub albedo_factor: glam::Vec3,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub normal_scale: f32,
    pub emission_factor: glam::Vec3,
}
```

**シェーダー実装**:
```wgsl
// assets/shaders/pbr/pbr.wgsl
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
}

struct PbrUniforms {
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    normal_scale: f32,
    emission: vec3<f32>,
}

@group(1) @binding(0)
var<uniform> material: PbrUniforms;

@group(1) @binding(1)  
var albedo_texture: texture_2d<f32>;

@group(1) @binding(2)
var normal_texture: texture_2d<f32>;

@group(1) @binding(3)
var metallic_roughness_texture: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // PBR calculations
    let albedo = sample_albedo(in.uv);
    let normal = sample_normal(in.uv, in.normal, in.tangent, in.bitangent);
    let metallic_roughness = sample_metallic_roughness(in.uv);
    
    // Lighting calculations
    let color = calculate_pbr_lighting(
        in.world_pos,
        normal,
        albedo,
        metallic_roughness.x,
        metallic_roughness.y
    );
    
    return vec4<f32>(color, 1.0);
}
```

### 2.2 テクスチャシステム

**優先度**: 🟡 中

**実装内容**:
```rust
// src/resources/texture.rs (新規作成)
pub struct Texture {
    pub texture: Arc<wgpu::Texture>,
    pub view: Arc<wgpu::TextureView>,
    pub sampler: Arc<wgpu::Sampler>,
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
}

pub struct TextureLoader {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
}

impl TextureLoader {
    pub async fn load_from_file(&self, path: &str) -> Result<Texture, TextureError> {
        let image = image::open(path)?.to_rgba8();
        let (width, height) = image.dimensions();
        
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(path),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );
        
        // Create view and sampler...
        Ok(Texture { /* ... */ })
    }
}
```

## 3. ライティングシステム

### 3.1 複数光源サポート

**優先度**: 🟡 中

**実装内容**:
```rust
// src/lighting/mod.rs (新規作成)
pub mod light;
pub mod shadow;

// src/lighting/light.rs
#[derive(Debug, Clone, Component)]
pub enum Light {
    Directional(DirectionalLight),
    Point(PointLight),
    Spot(SpotLight),
}

#[derive(Debug, Clone)]
pub struct DirectionalLight {
    pub direction: glam::Vec3,
    pub color: glam::Vec3,
    pub intensity: f32,
    pub cast_shadows: bool,
}

#[derive(Debug, Clone)]
pub struct PointLight {
    pub position: glam::Vec3,
    pub color: glam::Vec3,
    pub intensity: f32,
    pub range: f32,
    pub attenuation: glam::Vec3, // constant, linear, quadratic
}

// src/lighting/shadow.rs
pub struct ShadowMap {
    pub texture: Arc<wgpu::Texture>,
    pub view: Arc<wgpu::TextureView>,
    pub sampler: Arc<wgpu::Sampler>,
    pub size: u32,
}

pub struct ShadowSystem {
    shadow_maps: HashMap<Entity, ShadowMap>,
    shadow_pipeline: Arc<wgpu::RenderPipeline>,
}
```

### 3.2 IBL (Image Based Lighting)

**優先度**: 🟢 低

**実装内容**:
```rust
// src/lighting/ibl.rs (新規作成)
pub struct EnvironmentMap {
    pub skybox: Arc<wgpu::Texture>,
    pub irradiance: Arc<wgpu::Texture>,
    pub prefiltered: Arc<wgpu::Texture>,
    pub brdf_lut: Arc<wgpu::Texture>,
}

impl EnvironmentMap {
    pub async fn from_hdr(path: &str, device: &wgpu::Device) -> Result<Self, IblError> {
        // HDR画像からキューブマップを生成
        // Irradiance map の事前計算
        // Prefiltered environment map の生成
        // BRDF LUT の生成
    }
}
```

## 4. アセットパイプライン

### 4.1 アセットローディングシステム

**優先度**: 🟡 中

**実装内容**:
```rust
// src/assets/mod.rs (新規作成)
pub mod loader;
pub mod asset_server;
pub mod handle;

// src/assets/asset_server.rs
pub struct AssetServer {
    loaders: HashMap<String, Box<dyn AssetLoader>>,
    assets: HashMap<AssetId, LoadedAsset>,
    loading_queue: VecDeque<LoadRequest>,
}

pub trait AssetLoader: Send + Sync {
    fn can_load(&self, extension: &str) -> bool;
    fn load(&self, data: &[u8]) -> Result<Box<dyn Asset>, LoadError>;
}

// src/assets/handle.rs
pub struct Handle<T: Asset> {
    id: AssetId,
    _phantom: PhantomData<T>,
}

impl<T: Asset> Handle<T> {
    pub fn get(&self, asset_server: &AssetServer) -> Option<&T> {
        asset_server.get_asset(self.id)?.downcast_ref()
    }
}

// glTF サポート
// src/assets/gltf_loader.rs
pub struct GltfLoader;

impl AssetLoader for GltfLoader {
    fn can_load(&self, extension: &str) -> bool {
        extension == "gltf" || extension == "glb"
    }
    
    fn load(&self, data: &[u8]) -> Result<Box<dyn Asset>, LoadError> {
        let gltf = gltf::Gltf::from_slice(data)?;
        // glTF parsing and conversion
    }
}
```

### 4.2 シーンフォーマット

**優先度**: 🟢 低

**実装内容**:
```rust
// src/scene/scene_format.rs (新規作成)
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SceneFile {
    pub entities: Vec<SceneEntity>,
    pub environment: EnvironmentSettings,
    pub cameras: Vec<SceneCamera>,
    pub lights: Vec<SceneLight>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SceneEntity {
    pub name: String,
    pub transform: TransformData,
    pub components: Vec<ComponentData>,
}

// YAML/JSONでのシーン定義サポート
impl SceneFile {
    pub fn load_from_file(path: &str) -> Result<Self, SceneError> {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(&content)?)
    }
}
```

## 5. エディター機能

### 5.1 DebugUI (egui統合)

**優先度**: 🟡 中

**実装内容**:
```rust
// Cargo.toml
[dependencies]
egui = "0.24"
egui-wgpu = "0.24"  
egui-winit = "0.24"

// src/editor/mod.rs (新規作成)
pub mod debug_ui;
pub mod inspector;

// src/editor/debug_ui.rs
pub struct DebugUI {
    egui_ctx: egui::Context,
    egui_winit: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
}

impl DebugUI {
    pub fn draw(&mut self, world: &World, metrics: &EngineMetrics) {
        egui::Window::new("Engine Debug")
            .show(&self.egui_ctx, |ui| {
                ui.label(format!("FPS: {:.1}", metrics.fps));
                ui.label(format!("Frame Time: {:.2}ms", metrics.frame_time * 1000.0));
                ui.label(format!("Entities: {}", world.entity_count()));
                
                ui.separator();
                
                if ui.button("Add Cube") {
                    // エンティティ追加
                }
                
                if ui.button("Clear Scene") {
                    // シーンクリア
                }
            });
    }
}
```

### 5.2 インスペクター

**優先度**: 🟢 低

**実装内容**:
```rust
// src/editor/inspector.rs
pub struct Inspector {
    selected_entity: Option<Entity>,
}

impl Inspector {
    pub fn draw(&mut self, ui: &mut egui::Ui, world: &mut World) {
        if let Some(entity) = self.selected_entity {
            ui.heading("Inspector");
            
            // Transform コンポーネント
            if let Some(transform) = world.get_component_mut::<Transform>(entity) {
                ui.collapsing("Transform", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Position:");
                        ui.add(egui::DragValue::new(&mut transform.position.x));
                        ui.add(egui::DragValue::new(&mut transform.position.y));
                        ui.add(egui::DragValue::new(&mut transform.position.z));
                    });
                });
            }
            
            // MeshRenderer コンポーネント
            if let Some(renderer) = world.get_component::<MeshRenderer>(entity) {
                ui.collapsing("Mesh Renderer", |ui| {
                    ui.label(format!("Mesh: {:?}", renderer.mesh_id));
                    ui.label(format!("Material: {:?}", renderer.material_id));
                });
            }
        }
    }
}
```

## 6. パフォーマンス最適化

### 6.1 フラスタムカリング

**優先度**: 🟡 中

**実装内容**:
```rust
// src/graphics/culling.rs (新規作成)
pub struct Frustum {
    planes: [glam::Vec4; 6], // left, right, bottom, top, near, far
}

impl Frustum {
    pub fn from_view_proj(view_proj: &glam::Mat4) -> Self {
        // View-projection matrix から frustum planes を抽出
        let m = *view_proj;
        
        Self {
            planes: [
                glam::Vec4::new(m.w_axis.x + m.x_axis.x, m.w_axis.y + m.x_axis.y, m.w_axis.z + m.x_axis.z, m.w_axis.w + m.x_axis.w), // left
                glam::Vec4::new(m.w_axis.x - m.x_axis.x, m.w_axis.y - m.x_axis.y, m.w_axis.z - m.x_axis.z, m.w_axis.w - m.x_axis.w), // right
                // ... other planes
            ]
        }
    }
    
    pub fn intersects_aabb(&self, aabb: &AABB) -> bool {
        for plane in &self.planes {
            if aabb.distance_to_plane(*plane) < 0.0 {
                return false;
            }
        }
        true
    }
}

pub struct AABB {
    pub min: glam::Vec3,
    pub max: glam::Vec3,
}
```

### 6.2 LOD (Level of Detail) システム

**優先度**: 🟢 低

**実装内容**:
```rust
// src/graphics/lod.rs (新規作成)
#[derive(Component)]
pub struct LodGroup {
    pub lod_levels: Vec<LodLevel>,
    pub current_lod: usize,
}

pub struct LodLevel {
    pub mesh_id: ResourceId,
    pub distance_threshold: f32,
    pub screen_coverage_threshold: f32,
}

pub struct LodSystem;

impl System for LodSystem {
    fn update(&mut self, world: &mut World, _dt: f32) {
        // カメラ位置を取得
        let camera_pos = world.get_camera_position();
        
        for (entity, (transform, lod_group)) in world.query_mut::<(Transform, LodGroup)>() {
            let distance = (transform.position - camera_pos).length();
            
            // 距離に基づいてLODレベルを選択
            for (i, lod_level) in lod_group.lod_levels.iter().enumerate() {
                if distance <= lod_level.distance_threshold {
                    lod_group.current_lod = i;
                    break;
                }
            }
        }
    }
}
```

## 7. 物理シミュレーション統合

### 7.1 物理エンジン統合 (rapier)

**優先度**: 🟢 低

**実装内容**:
```rust
// Cargo.toml
[dependencies]
rapier3d = "0.18"

// src/physics/mod.rs (新規作成)
pub mod physics_world;
pub mod rigidbody;
pub mod collider;

// src/physics/physics_world.rs
pub struct PhysicsWorld {
    gravity: glam::Vec3,
    integration_parameters: rapier3d::dynamics::IntegrationParameters,
    physics_pipeline: rapier3d::pipeline::PhysicsPipeline,
    island_manager: rapier3d::geometry::IslandManager,
    broad_phase: rapier3d::geometry::BroadPhase,
    narrow_phase: rapier3d::geometry::NarrowPhase,
    rigid_body_set: rapier3d::dynamics::RigidBodySet,
    collider_set: rapier3d::geometry::ColliderSet,
    impulse_joint_set: rapier3d::dynamics::ImpulseJointSet,
    multibody_joint_set: rapier3d::dynamics::MultibodyJointSet,
    ccd_solver: rapier3d::dynamics::CCDSolver,
}

#[derive(Component)]
pub struct RigidBody {
    pub handle: rapier3d::dynamics::RigidBodyHandle,
}

#[derive(Component)]
pub struct Collider {
    pub handle: rapier3d::geometry::ColliderHandle,
}
```

## 実装ロードマップ

### Month 1: 基盤アーキテクチャ
- [ ] ECS フレームワーク実装
- [ ] マルチスレッドレンダリング基盤
- [ ] マテリアルシステム基盤
- [ ] テクスチャローディング

### Month 2: レンダリング機能拡張
- [ ] PBR シェーダー実装
- [ ] ライティングシステム
- [ ] シャドウマッピング
- [ ] フラスタムカリング

### Month 3: ツールと最適化
- [ ] アセットパイプライン
- [ ] デバッグUI/エディター
- [ ] パフォーマンス最適化
- [ ] 物理シミュレーション統合

## 期待される最終成果

1. **本格的な3Dエンジン**: ECS、PBR、マルチライト対応
2. **プロダクションレディ**: アセットパイプライン、エディター機能
3. **高パフォーマンス**: マルチスレッド、カリング、LOD
4. **拡張性**: モジュラー設計、プラグインシステム
5. **開発効率**: ビジュアルエディター、リアルタイムデバッグ

このフェーズの完了により、商用レベルの 3D アプリケーション開発が可能な本格的なエンジンとなります。