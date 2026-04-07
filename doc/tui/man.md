# `man` — コマンドのヘルプ表示

## 概要

`man` コマンドは、利用可能なコマンドの一覧、または特定のコマンドの詳細なヘルプを表示します。

## 使い方

```
man [COMMAND]
```

| 引数 | 必須 | 説明 |
|---|---|---|
| `[COMMAND]` | — | ヘルプを表示したいコマンド名（省略時はコマンド一覧を表示） |

## 例

### コマンド一覧を表示

```
man
```

出力:

```
Available commands:
  ai         Call the AI
  close      Close the session
  history    Show command history
  man        Show manual
  session    Manage sessions
  space      Switch workspace

You can show detailed help with 'man <command>'.
```

### 特定のコマンドの詳細ヘルプを表示

```
man session
```

出力:

```
Command: session
Description: Manage sessions
Help:
Manages sessions (new working branches and worktrees).
Usage: session start <branch_name> [--base <base_branch>]
Creates a new branch and sets up a corresponding Git worktree.
```

## 関連ドキュメント

- [TUI コマンド一覧](./index.md)
