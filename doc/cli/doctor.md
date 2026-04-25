# `usagi doctor` — システム依存関係の確認

## 概要

`usagi doctor` は、`usagi.ai` が正常に動作するために必要な外部ツール（`git` など）や言語ランタイムがシステムにインストールされているかを確認するコマンドです。

実行すると、可愛いうさぎのアニメーションが画面を走り抜け、システムの状態をチェックしてくれます。🐰💨

## 使い方

```bash
usagi doctor
```

## チェック項目

このコマンドは以下のツールを確認します。

| ツール | 重要度 | 説明 |
|---|---|---|
| `git` | **必須 (Essential)** | リポジトリのクローンやワークツリーの作成に使用します。 |
| デフォルトシェル | **必須 (Essential)** | ターミナルで使用するシェルです。 |
| `aws` | **必須 (Essential)** | AWS SSO ログインなどの AWS 操作に使用します。 |
| `node` / `npm` | 任意 (Optional) | AI エージェントの実行や開発環境で必要になる場合があります。 |
| `python` / `python3` | 任意 (Optional) | AI エージェントの実行やスクリプト実行で必要になる場合があります。 |

## 出力例

```
                                     (\(\
                                    (='-')
                                    o(_(")(")
                                    USAGI AI
🐰 USAGI DOCTOR is checking your system... 🐰
🥕 git        (Essential) git version 2.39.3 (Apple Git-146)
🥕 bash       (Essential) GNU bash, version 3.2.57(1)-release (arm64-apple-darwin23)
🥕 aws        (Essential) aws-cli/2.15.30 Python/3.11.8 Darwin/23.4.0 exe/x86_64 prompt/off
🥕 node       (Optional)  v24.13.0
🥕 npm        (Optional)  11.6.2
🥕 python3    (Optional)  Python 3.13.5
🐾 python     (Optional)  Not found

✨ Everything looks fluffy! Usagi is ready to hop! ✨
```

- 🥕: インストール済み。
- ❌: 必須ツールが見つかりません。
- 🐾: 任意ツールが見つかりません。

## 関連ドキュメント

- [TUI 版 `doctor`](../tui/doctor.md)
- [`usagi init`](./init.md)
