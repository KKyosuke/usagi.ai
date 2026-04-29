# usagi.ai ドキュメント

このディレクトリ（`doc/`）には `usagi.ai` の仕様、コマンドリファレンス、内部構造に関するドキュメントがまとめられています。

> [!TIP]
> ドキュメントを新規作成・編集する際は、必ず [ドキュメント執筆ガイドライン](./writing_guide.md) を一読し、ルールに沿って記述してください。

## 📁 ディレクトリ構成と目次

ドキュメントは大きく分けて、ターミナルから直接実行する **CLI** と、`usagi hop` 起動後に操作する **APP（TUI）** の2つの大カテゴリに分かれています。

### 💻 `cli/` - CLI コマンドリファレンス
ターミナルから直接実行する `usagi` コマンドに関するドキュメントです。

- [`usagi init`](./cli/init.md): リポジトリの初期化
- [`usagi hop`](./cli/hop.md): プロジェクト内ターミナル（TUI）の起動
- [`usagi doctor`](./cli/doctor.md): 依存関係のチェック

---

### 🐰 `app/` - APP 内部仕様（TUI）
`usagi hop` で起動する TUI アプリケーションの内部構造や操作に関するドキュメントです。

#### `app/tui/` - TUI 内部コマンドリファレンス
- [TUI コマンド一覧](./app/tui/index.md): TUI内で使える全コマンドの概要
- [`session`](./app/tui/session.md): セッション（ブランチ＋worktree）の管理
- [`space`](./app/tui/space.md): ワークスペース（worktree）の切り替え
- [`ai`](./app/tui/ai.md): AIへの指示・対話
- [`terminal`](./app/tui/terminal.md): 対話型ターミナルの起動
- [`history`](./app/tui/history.md): コマンド実行履歴の表示
- [`doctor`](./app/tui/doctor.md): 依存関係のチェック（TUI版）
- [`man`](./app/tui/man.md): コマンドのヘルプ表示

#### `app/architecture/` - 内部設計・アーキテクチャ
- [ソースコードの構造](./app/architecture/src.md): クリーンアーキテクチャによる各層の責務とディレクトリ構成
- [グローバルDB（共通リポジトリ管理）](./app/architecture/global.md): システム全体でのリポジトリ追跡の仕組み

#### `app/ui/` - UIデザイン・画面遷移
- [UI（ユーザーインターフェース）の構成と名称](./app/ui/layout.md): 画面各部の名称と役割
- [モードの種類と切り替え](./app/ui/mode.md): TUIの各モード（SideMenu, Commandなど）の定義
- [画面遷移](./app/ui/transition.md): 各画面間の遷移フロー

#### `app/project/` - プロジェクト構成・仕様
- [初期化後のディレクトリ構造](./app/project/directory.md): `usagi init` 後に作成されるファイル群の説明
- [プロジェクト設定（usagi.config）](./app/project/config.md): 設定ファイルの仕様

## 🧭 初学者向けガイド
初めて `usagi.ai` のコードや仕様に触れる方は、以下の順序でドキュメントを読むことをおすすめします。

1. **[ソースコードの構造](./app/architecture/src.md)** - 全体的なアーキテクチャと処理フローを把握する
2. **[初期化後のディレクトリ構造](./app/project/directory.md)** - アプリがプロジェクトをどのように管理しているかを知る
3. **[`usagi hop` コマンド](./cli/hop.md)** および **[UIの構成と名称](./app/ui/layout.md)** - メイン機能であるTUIの動作と画面構成を理解する
