# Develop

## 開発環境でのテスト方法 (playground)

`playground` ディレクトリ内で `usagi` コマンドをテストする手順です。

### 1. バイナリのビルド
プロジェクトのルートディレクトリでバイナリをビルドします。

```bash
cargo build
```

### 2. `playground` での実行
`playground` ディレクトリに移動し、ビルドされたバイナリを相対パスで指定して実行します。

```bash
cd playground
../target/debug/usagi init https://github.com/KKyosuke/sqlalchemy-test
```

### 補足
- `init` コマンドを実行すると、実行したディレクトリに `.usagi/`、`main/`、`usagi.config`、`.gitignore` が作成されます。
- `playground` ディレクトリが空であることを確認してから実行することをお勧めします。

