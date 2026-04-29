# `terminal` — 対話型ターミナルの起動

## 概要

`terminal` コマンドは、現在のワークスペース（worktree）内で対話型のシェルを PTY (Pseudo-terminal) 経由で起動します。
TUI 内で直接コマンドを実行したり、AI エージェントと対話したりするために使用します。

## 使い方

```
terminal [command]
```

引数を指定しない場合は、OS のデフォルトシェル（環境変数 `SHELL` または `COMSPEC`）が起動します。

## 例

```bash
terminal /bin/bash
terminal npm install
```

## 動作

- TUI 画面が一時的にターミナル画面に切り替わります。
- `exit` と入力するか、シェルを終了すると TUI 画面に戻ります。
- ターミナル内では通常のシェルと同様に操作が可能です。

## 関連ドキュメント

- [TUI コマンド一覧](./index.md)
