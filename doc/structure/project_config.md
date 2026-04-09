# プロジェクト設定（usagi.config）

`usagi.config` は、プロジェクトのルートディレクトリに配置される設定ファイルです。
`usagi init` 時に自動生成されますが、必要に応じて手動で編集することが可能です。

## ファイル形式

TOML 形式に近い key-value 形式を採用しています（現時点では単純な key = value）。

## 設定項目

| キー | 型 | 説明 |
|---|---|---|
| `repository_url` | `String` | クローン元の Git リポジトリ URL。`usagi init` 時に指定した値が記録されます。 |

## 設定例

```toml
# usagi project configuration
repository_url = "https://github.com/KKyosuke/sqlalchemy-test.git"
```

## 役割

この設定ファイルは、プロジェクト固有の設定を保持するために使用されます。
グローバルな `repositories.json` が「どこにプロジェクトがあるか」を管理するのに対し、`usagi.config` は「そのプロジェクトがどのような設定を持っているか」を管理します。

## 関連ドキュメント

- [初期化後のディレクトリ構造](./directory.md)
- [グローバルDB（共通リポジトリ管理）](./global.md)
- [`usagi init` コマンド](../cli/init.md)
