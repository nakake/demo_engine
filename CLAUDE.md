# CLAUDE.md

このファイルは、Claude Code がこのリポジトリでコードを扱う際の指針を提供します。

## プロジェクト概要

これはRust + WGPU + Winitで構築された3Dグラフィックスエンジンデモです。リアルタイムカメラ制御、Scene trait抽象化、適切なエラーハンドリングを持つモジュラーアーキテクチャを実装しています。

**現在の状態**: Phase 1 完了（2025-08-10）- 警告ゼロ + ドキュメント + テスト基盤

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
App → GraphicsEngine → Scene → Resources
 ↓         ↓            ↓         ↓
Input → Rendering → Camera → WGPU
```

### 🏗️ モジュール構成

- **`app/`** - アプリケーションライフサイクル（Winit ApplicationHandler）
- **`core/`** - エラーハンドリング（EngineError、unwrap()ゼロ）
- **`graphics/`** - WGPUレンダリングエンジン（現在はGod Object状態）
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
- **✅ モジュラーアーキテクチャ**: 6つの責任分離されたモジュール
- **✅ リアルタイム3D**: WASD移動、矢印キー回転の滑らかな操作
- **✅ 適切なレンダリング**: WGPU最適化、ユニフォームバッファ統合

## ✅ Phase 1 完了事項（2025-08-10）

- **✅ コンパイル警告ゼロ** - 19個の警告を全て解消
- **✅ ドキュメント整備** - 主要4コンポーネントの詳細説明
- **✅ テスト基盤確立** - 12個の単体テスト（ResourceId + Camera）
- **✅ バグ修正** - キー入力でオブジェクト消失問題を解決

## ⚠️ Phase 2 計画事項

- **GraphicsEngine God Object** - 責任分離
- **設定のハードコーディング** - constants.rs + 外部化
- **ログシステム** - println! 置換
- **基本メトリクス** - パフォーマンス監視

## 🎯 次のアクション

**Phase 1 完了済み**: [完了記録](./docs/claude/refactoring/phase_1_immediate.md)

**Phase 2 開始準備完了**: [Phase 2 短期改善](./docs/claude/refactoring/phase_2_short_term.md)

1. GraphicsEngine責任分離（Renderer + Pipeline分離）
2. constants.rs作成（マジックナンバー解消）
3. ログシステム構築（println! 置換）
4. 基本メトリクス実装

**推定時間**: 1-2週間でアーキテクチャ改善を実現