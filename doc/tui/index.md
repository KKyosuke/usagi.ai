# TUI コマンド一覧

`usagi hop` のターミナル TUI 内でコマンドモードから実行できるコマンドの一覧です。

コマンドモードへの入り方: サイドメニューモードで `Enter` キーを押す

## コマンド一覧

| コマンド | 使い方 | 説明 |
|---|---|---|
| [`ai`](./ai.md) | `ai <MESSAGE>` | AI にメッセージを送信する |
| [`close`](./close.md) | `close` | セッションを閉じてプロジェクト選択画面へ戻る |
| [`history`](./history.md) | `history` | コマンド実行履歴を表示する |
| [`man`](./man.md) | `man [COMMAND]` | コマンドのヘルプを表示する |
| [`session`](./session.md) | `session start <BRANCH>` | 新しいセッション（ブランチ＋worktree）を作成する |
| [`space`](./space.md) | `space <WORKTREE>` | ワークスペース（worktree）を切り替える |

## オートコンプリートの使い方

コマンドモードで文字を入力すると、コマンド名やサブコマンドのサジェストが自動表示されます。
`Tab` キーでサジェストを補完できます。

## 関連ドキュメント

- [`usagi hop` コマンド](../cli/hop.md)
- [モードの種類と切り替え](../structure/mode.md)
