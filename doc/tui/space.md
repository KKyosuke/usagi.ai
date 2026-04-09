# `space` — ワークスペースの切り替え

## 概要

`space` コマンドは、現在アクティブなワークスペース（Git worktree）を切り替えます。
`main` を指定するとメイン worktree に戻ります。

## 使い方

```
space <WORKTREE>
```

| 引数 | 必須 | 説明 |
|---|---|---|
| `<WORKTREE>` | ✅ | 切り替え先の worktree 名（`main` でメイン worktree に戻る） |

## 例

```
# my-feature ワークスペースに切り替える
space my-feature

# メイン worktree に戻る
space main
```

## 動作

- 指定した worktree が存在しない場合はエラーになります。
- `main` を指定すると `current_worktree` が `null` にリセットされます（メイン worktree への切り替え）。
- 切り替え後は `.usagi/state.json` の `current_worktree` フィールドが更新されます。

## state.json の変化

```json
// space my-feature 実行後
{
  "initialized": true,
  "worktrees": [
    {
      "branch": "main",
      "directory": "main",
      "default": true,
      "modified_at": "2026-04-09 20:01 UTC"
    },
    {
      "branch": "my-feature",
      "directory": "my-feature",
      "default": false,
      "modified_at": "2026-04-09 20:01 UTC"
    }
  ],
  "current_worktree": "my-feature",
  "history": [...]
}

// space main 実行後
{
  "initialized": true,
  "worktrees": [
    {
      "branch": "main",
      "directory": "main",
      "default": true,
      "modified_at": "2026-04-09 20:01 UTC"
    },
    {
      "branch": "my-feature",
      "directory": "my-feature",
      "default": false,
      "modified_at": "2026-04-09 20:01 UTC"
    }
  ],
  "current_worktree": null,
  "history": [...]
}
```

## 関連ドキュメント

- [TUI コマンド一覧](./index.md)
- [`session` コマンド（新しいセッションの作成）](./session.md)
