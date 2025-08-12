# CLAUDE.md

このファイルは、Claude Code がこのリポジトリでコードを扱う際の指針を提供します。

## プロジェクト概要

これはRust + WGPU + Winitで構築された3Dグラフィックスエンジンデモです。リアルタイムカメラ制御、Scene trait抽象化、適切なエラーハンドリングを持つモジュラーアーキテクチャを実装しています。

**現在の状態**: Phase 2.1 完了（2025-08-12）- GraphicsEngine分割 + Phase 2.2 進行中

## 📚 詳細ドキュメント

プロジェクトの包括的な情報は `docs/claude/` に整理されています：

- **[メインドキュメント](./docs/claude/README.md)** - 概要とナビゲーション
- **[現状評価](./docs/claude/evaluation/current_state.md)** - 実装状況とメトリクス
- **[アーキテクチャ分析](./docs/claude/architecture/analysis.md)** - 設計評価と改善案
- **[リファクタリング計画](./docs/claude/refactoring/roadmap.md)** - 段階的改善ロードマップ

## 🚀 クイックスタート

```bash
# ビルド & 実行
cargo run

# キー操作
WASD: カメラ移動  |  Q/E: 上下移動  |  矢印キー: カメラ回転  |  ESC: 終了
```

## 🔧 開発コマンド

```bash
cargo build          # ビルド（警告ゼロ ✅）
cargo run --release  # リリースモード実行
cargo test           # テスト実行（12個の単体テスト ✅）
cargo doc --open     # ドキュメント生成
```

## ⚡ 現在のアーキテクチャ

```
App → GraphicsEngine (統制) → Scene → Resources
 ↓         ↓                   ↓         ↓
Input → Renderer (実行) → Camera → WGPU
       ↓
   SurfaceManager (Surface管理)
```

### 🏗️ モジュール構成

- **`app/`** - アプリケーションライフサイクル（Winit ApplicationHandler）
- **`core/`** - エラーハンドリング（EngineError、unwrap()ゼロ）
- **`graphics/`** - WGPUレンダリングシステム（3層分離アーキテクチャ）
  - `engine.rs` - 統制レイヤー（メトリクス・エラー統合）
  - `renderer.rs` - レンダリング実行（CommandBuffer生成）
  - `surface_manager.rs` - Surface管理（フレーム取得・表示）
- **`resources/`** - リソース管理（HashMap + Arc共有）
- **`scene/`** - Scene trait + DemoScene実装（カメラ統合済み）
- **`input/`** - 入力処理（HashSet、リアルタイムカメラ制御）

### 🎯 現在の実装

- **レンダリング**: 4色グラデーションクワッド（インデックスバッファ使用）
- **カメラ制御**: WASD移動、矢印キー回転、QE上下移動
- **シェーダー**: WGSL、カメラユニフォームバッファ統合
- **パフォーマンス**: 60FPS、デルタタイム計算、継続的レンダリングループ

## 🔧 技術スタック

- **WGPU** (26.0.1) - GPU抽象化レイヤー
- **Winit** (0.30.12) - クロスプラットフォームウィンドウ
- **glam** (0.30.5) - 3D数学ライブラリ  
- **bytemuck** - 安全な型変換

## 📁 重要なファイル

```
src/
├── app/mod.rs           # アプリケーション主処理
├── graphics/engine.rs   # レンダリングエンジン（要リファクタリング）
├── scene/demo_scene.rs  # メインシーン実装
├── input/mod.rs         # キーボード・マウス処理
└── resources/manager.rs # リソース管理

assets/shaders/basic/
└── triangle.wgsl        # メインシェーダー（カメラ統合済み）
```

## 🏆 技術的成果

- **✅ メモリ安全**: unwrap()完全排除、Arc共有による安全な設計
- **✅ モジュラーアーキテクチャ**: 8つの責任分離されたモジュール（GraphicsEngine分割完了）
- **✅ リアルタイム3D**: WASD移動、矢印キー回転の滑らかな操作
- **✅ 適切なレンダリング**: WGPU最適化、ユニフォームバッファ統合
- **✅ God Object解決**: GraphicsEngine（253行）→ 3コンポーネント分離

## ✅ Phase 1 完了事項（2025-08-10）

- **✅ コンパイル警告ゼロ** - 19個の警告を全て解消
- **✅ ドキュメント整備** - 主要4コンポーネントの詳細説明
- **✅ テスト基盤確立** - 12個の単体テスト（ResourceId + Camera）
- **✅ バグ修正** - キー入力でオブジェクト消失問題を解決

## ✅ Phase 2.1 完了事項（2025-08-12）

- **✅ GraphicsEngine分割** - Renderer + SurfaceManager + GraphicsEngine（統制層）
- **✅ Renderer実装** - 純粋レンダリングロジック、CommandBuffer返却
- **✅ SurfaceManager実装** - Surface管理・フレーム取得・表示
- **✅ 後方互換性維持** - 既存API（new/render/resize）シグネチャ不変

## ⚠️ Phase 2 残り項目（基盤整備完成）

### 🔴 基盤システム完成（最優先）
- **統合設定システム** - constants.rs + config.toml統合（マジックナンバー解消 + 外部設定）
- **ログシステム導入** - println! → log::debug! 置換
- **基本メトリクス実装** - FPS/フレーム時間監視

## 🚀 Phase 3 計画事項（エンジン機能拡張）

### 🎯 エンジン機能設計・実装
- **Scene管理システム** - SceneManager設計、複数シーン、遷移システム
- **入力システム設計** - InputBinding、InputAction、カスタマイズ可能な入力
- **リソース管理拡張** - 動的ロード、メモリ管理最適化
- **パフォーマンス最適化** - GPU最適化、並列処理

## 🎯 次のアクション

**Phase 1 完了済み**: [完了記録](./docs/claude/refactoring/phase_1_immediate.md)

**Phase 2.1 完了済み**: GraphicsEngine分割（2025-08-12）

**Phase 2.2 進行中**: [Phase 2 基盤整備](./docs/claude/refactoring/phase_2_short_term.md)

1. ~~GraphicsEngine責任分離（Renderer + SurfaceManager分離）~~ ✅
2. constants.rs作成（マジックナンバー解消）
3. ログシステム構築（println! 置換）
4. 基本メトリクス実装

**Phase 3 準備完了**: エンジン機能拡張フェーズ

- Scene管理システム設計・実装
- 入力システム設計・実装  
- リソース管理拡張
- パフォーマンス最適化

**推定時間**: Phase 2完了まで3-5日、Phase 3は本格的なエンジン仕様策定のため2-3週間