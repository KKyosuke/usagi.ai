# UI（ユーザーインターフェース）の構成と名称

`usagi.ai` で使用される画面と各パーツの名称を定義し、共通の認識を持てるようにします。

## 画面一覧 (Screen List)

1.  [Home 画面 (Home Screen)](#1-home-画面-home-screen)
2.  [プロジェクト選択画面 (Project Selection Screen)](#2-プロジェクト選択画面-project-selection-screen)
3.  [Workspace 画面 (Workspace Screen) / プロジェクト画面](#3-workspace-画面-workspace-screen--プロジェクト画面)

---

## 1. Home 画面 (Home Screen)

`usagi hop` を実行した直後に表示されるメインメニュー画面です。

- **役割**: アプリケーションのロゴ表示とメインアクションの選択。
- **実装**: `src/presentation/tui/home.rs`。

### サンプル (ASCII Art)
```text
  (\(\
 (='-')       USAGI AI
 o(_(")(")

  ●  Open       o
  ●  New        e
  ●  Config     c
  ●  Quit       q
```

## 2. プロジェクト選択画面 (Project Selection Screen)

Home 画面で `Open` を選択した際に表示される、登録済みのプロジェクト（リポジトリ）を選択するための画面です。

- **役割**: 管理下にあるリポジトリの一覧表示と選択。
- **実装**: `src/presentation/tui/project.rs`。

### サンプル (Layout)
```text
     ● > repository-name-1      modified: 2026/04/10 07:23
         repository-name-2      modified: 2026/04/10 07:00

         v0.1.0 ⚡ plugins 4/55 in 23.885ms
```

## 3. Workspace画面 (Workspace Screen) / プロジェクト画面

`usagi hop` によって起動する、特定のプロジェクトを操作するためのメインのTUI画面です。
以下の3つの主要なセクションで構成されています。

### サンプル (Layout Structure)
```text
[Header] --------------------------------------------------------------
MODE: SideMenu
-----------------------------------------------------------------------
[セッション一覧]        | [コンテンツ画面]
workspace            | Welcome to usagi terminal! (Dir: main)
> ●  main            | 
      modified: ...  | ai hello
                     | hello! how can I help you?
                     | 
                     | 
-----------------------------------------------------------------------
[コマンド入力]          | 
COMMAND              | ai 
                     | (ai) > _
-----------------------------------------------------------------------
```

### 各パーツの詳細

#### ヘッダー (Header)
画面上部に表示されるエリアです。現在の `MODE`（SideMenu, Command, Interaction, Execution）が表示されます。

#### セッション一覧 (Session List / Side Menu)
画面左側のカラムです。
- **内容**: ワークスペース（worktree/セッション）の一覧が表示されます。現在選択中のセッションには `●` が表示されます。
- **操作**: 矢印キー（上下）でセッションを選択できます。

#### コンテンツ画面 (Content Screen / Terminal View)
画面右側の広範なエリアです。
- **内容**: 選択中のセッションへのウェルカムメッセージや、実行されたコマンドの履歴、AIの回答、およびターミナルコマンドの実行結果が表示されます。
- **ターミナル機能**: `terminal` コマンドを実行することで、対話型ターミナルを起動できます。

#### コマンド入力 (Command Input / Command Section)
画面下部の入力エリアです。
- **内容**: `COMMAND |` というプロンプトに続いてコマンドを入力できます。入力中はオートコンプリートのポップアップが表示されます。

---

## 関連ドキュメント

- [画面遷移](./transition.md)
- [モードの種類と切り替え](./mode.md)
- [ソースコードの構造](./src.md)
