# UI（ユーザーインターフェース）の構成と名称

`usagi.ai` で使用される画面と各パーツの名称を定義し、共通の認識を持てるようにします。

## 画面一覧 (Screen List)

1.  [うさぎ画面 (Rabbit Screen)](#1-うさぎ画面-rabbit-screen)
2.  [プロジェクト選択画面 (Project Selection Screen)](#2-プロジェクト選択画面-project-selection-screen)
3.  [プロジェクト画面 (Project Screen)](#3-プロジェクト画面-project-screen)

---

## 1. うさぎ画面 (Rabbit Screen)

`usagi` コマンドを実行した際や、初期化が必要な際などに表示される、マスコットキャラクターのうさぎが表示される画面です。

- **役割**: アプリケーションのロゴ表示、ウェルカムメッセージ。
- **実装**: `src/presentation/tui/layout.rs` の `show_rabbit` 関数。

### サンプル (ASCII Art)
```text
  (\(\ 
 (='-') 
 o(_(")(")

  USAGI AI
```

## 2. プロジェクト選択画面 (Project Selection Screen)

`usagi open` を実行した際に表示される、登録済みのプロジェクト（リポジトリ）を選択するための画面です。

- **役割**: 管理下にあるリポジトリの一覧表示と選択。
- **実装**: `src/presentation/tui/open.rs`。

### サンプル (Layout)
```text
     ● > repository-name-1      modified: 2026/04/10 07:23
         repository-name-2      modified: 2026/04/10 07:00

         v0.1.0 ⚡ plugins 4/55 in 23.885ms
```

## 3. プロジェクト画面 (Project Screen)

`usagi hop` によって起動する、特定のプロジェクトを操作するためのメインのTUI画面です。
以下の4つの主要なセクションで構成されています。

### サンプル (Layout Structure)
```text
[Header] --------------------------------------------------------------
----- USAGI TERMINAL -----
MODE: SideMenu
-----------------------------------------------------------------------
[Side Menu]          | [Terminal View]
workspace            | Welcome to usagi terminal! (Workspace: main)
> ●  main            | 
     modified: ...   | ai hello
                     | hello! how can I help you?
                     | 
                     | 
-----------------------------------------------------------------------
[Command Section]    | 
COMMAND              | ai 
                     | Enter: execute, Escape: cancel...
-----------------------------------------------------------------------
```

### 各パーツの詳細

#### ヘッダー (Header)
画面上部に表示されるエリアです。
- **内容**: `----- USAGI TERMINAL -----` というタイトル、および現在の `MODE`（Global, SideMenu, Command, Execution）が表示されます。

#### サイドメニュー (Side Menu)
画面左側のカラムです。
- **内容**: ワークスペース（worktree）の一覧が表示されます。
- **操作**: 矢印キー（上下）でワークスペースを選択できます。

#### ターミナルビュー (Terminal View)
画面右側の広範なエリアです。
- **内容**: 選択中のワークスペースへのウェルカムメッセージや、実行されたコマンドの履歴、AIの回答、およびターミナルコマンドの実行結果が表示されます。
- **ターミナル機能**: `terminal <command>` と入力することで、選択中のワークスペースのディレクトリでシェルコマンドを実行できます。また、組み込みコマンドに該当しない入力も、自動的にターミナルコマンドとして実行が試行されます。
  - `terminal` を実行した場合には、このビューのタイトルが `TERMINAL` に切り替わります。

#### コマンドセクション (Command Section)
画面下部の入力エリアです。
- **内容**: `COMMAND |` というプロンプトに続いてコマンドを入力できます。入力中はオートコンプリートのポップアップが表示されます。

---

## 関連ドキュメント

- [モードの種類と切り替え](./mode.md)
- [ソースコードの構造](./src.md)
