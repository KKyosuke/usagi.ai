# `usagi doctor` — システム依存関係の確認

## 概要

`usagi doctor` は、`usagi.ai` が正常に動作するために必要な外部ツール（`git` など）や言語ランタイムがシステムにインストールされているかを確認するコマンドです。

## 使い方

```bash
usagi doctor
```

## チェック項目

このコマンドは以下のツールを確認します。

| ツール | 重要度 | 説明 |
|---|---|---|
| `git` | **必須 (Essential)** | リポジトリのクローンやワークツリーの作成に使用します。 |
| `bash` / `cmd.exe` | **必須 (Essential)** | ターミナルで使用するシェルです。 |
| `node` / `npm` | 任意 (Optional) | AI エージェントの実行や開発環境で必要になる場合があります。 |
| `python` / `python3` | 任意 (Optional) | AI エージェントの実行やスクリプト実行で必要になる場合があります。 |

## 出力例

```
Checking dependencies...

✅ git        (Essential) git version 2.39.3
✅ bash       (Essential) GNU bash, version 3.2.57(1)-release (arm64-apple-darwin23)
✅ node       (Optional)  v24.13.0
✅ npm        (Optional)  11.6.2
✅ python3    (Optional)  Python 3.13.5
⚠️ python     (Optional)  Not found

All essential commands are available.
```

- ✅: インストール済み。
- ❌: 必須ツールが見つかりません。
- ⚠️: 任意ツールが見つかりません。

## 関連ドキュメント

- [TUI 版 `doctor`](../tui/doctor.md)
- [`usagi init`](./init.md)
