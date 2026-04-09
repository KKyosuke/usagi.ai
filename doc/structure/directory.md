# 初期化後のディレクトリ構造

`usagi init` を実行すると、以下のディレクトリ構造が作成されます。

## ディレクトリ構造

```text
<project-root>/
├── .usagi/                  # usagi.ai の内部管理ディレクトリ（手動編集不要）
│   └── state.json           # プロジェクト状態（初期化フラグ・worktree一覧・履歴など）
├── main/                    # クローンされたリポジトリ（メイン worktree）
├── usagi.config             # プロジェクト設定ファイル（リポジトリURLなど）
└── .gitignore               # .usagi/ を無視する設定が追記される
```

`session start` でセッションを追加すると、`main/` と並列に worktree ディレクトリが追加されます：

```text
<project-root>/
├── .usagi/
│   └── state.json
├── main/                    # メイン worktree
├── my-feature/              # session start my-feature で作成された worktree
├── hotfix/                  # session start hotfix で作成された worktree
├── usagi.config
└── .gitignore
```

## ファイル・ディレクトリの説明

### `.usagi/`

`usagi.ai` がプロジェクトの管理状態を保持するための隠しディレクトリです。
手動での編集は不要です。`.gitignore` によって Git の管理対象から除外されます。

#### `.usagi/state.json`

プロジェクトの状態を管理する JSON ファイルです。

```json
{
  "initialized": true,
  "worktrees": [
    {
      "branch": "main",
      "directory": "main",
      "default": true,
      "modifiedAt": "2026-04-09 20:01 UTC"
    },
    {
      "branch": "my-feature",
      "directory": "my-feature",
      "default": false,
      "modifiedAt": "2026-04-09 20:01 UTC"
    }
  ],
  "current_worktree": "my-feature",
  "history": [
    "session start my-feature",
    "space my-feature"
  ]
}
```

| フィールド | 型 | 説明 |
|---|---|---|
| `initialized` | `bool` | プロジェクトが初期化済みかどうか |
| `worktrees` | `Vec<Worktree>` | 作成された worktree オブジェクトの一覧 |
| `current_worktree` | `String | null` | 現在アクティブな branch 名（`main` の場合は `null`） |
| `history` | `Vec<String>` | TUI コマンドの実行履歴 |

### `main/`

指定したリポジトリがクローンされるメインの作業ディレクトリです。
ディレクトリ名はデフォルトブランチ名から生成されます（`/` は `-` に変換）。

### `usagi.config`

ユーザーが参照・編集できる設定ファイルです。
初期状態では `usagi init` 時に指定したリポジトリの URL が記録されます。

### `.gitignore`

`.usagi/` ディレクトリを Git の管理対象から除外するための設定が追記されます。
既存の `.gitignore` がある場合は追記され、ない場合は新規作成されます。

## 関連ドキュメント

- [`usagi init` コマンド](../cli/init.md)
- [`session` コマンド](../tui/session.md)
- [グローバルDB（共通リポジトリ管理）](./global.md)
