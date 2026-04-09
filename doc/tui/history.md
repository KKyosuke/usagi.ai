# `history` — コマンド履歴の表示

## 概要

`history` コマンドは、現在のプロジェクトで実行されたコマンドの履歴を表示します。

## 使い方

```
history
```

引数はありません。

## 出力例

```
   1 session start my-feature
   2 space my-feature
   3 ai このコードのバグを修正して
   4 space main
```

左端の番号を入力することで、そのコマンドを再実行できます。

## 履歴の保存先

コマンド履歴はプロジェクトの `.usagi/state.json` 内の `history` フィールドに保存されます。

```json
{
  "initialized": true,
  "worktrees": [
    {
      "branch": "main",
      "directory": "main",
      "default": true,
      "modifiedAt": "2026-04-09 20:01 UTC"
    }
  ],
  "current_worktree": "main",
  "history": [
    "session start my-feature",
    "space my-feature"
  ]
}
```

## 関連ドキュメント

- [TUI コマンド一覧](./index.md)
- [初期化後のディレクトリ構造](../structure/directory.md)
