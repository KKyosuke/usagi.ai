# グローバルなプロジェクト情報管理

`register_project` 関数によって保存される情報は、システム全体で `usagi` プロジェクトを追跡するために使用されます。

## 保存場所

OSごとのユーザーデータディレクトリに保存されます。

- **macOS**: `~/Library/Application Support/usagi`
- **Linux**: `~/.local/share/usagi` (または `$XDG_DATA_HOME/usagi`)
- **Windows**: `C:\Users\<User>\AppData\Roaming\usagi\data`

## 保存ファイル

- **ファイル名**: `repositories.json`
- **フォーマット**: JSON

## データ構造

保存されるデータは、`Repositories` 構造体として定義されています。

```rust
struct Repositories {
    repositories: Vec<PathBuf>,
}
```

実際の JSON 形式は以下のようになります。

```json
{
  "repositories": [
    "/Users/username/projects/project1",
    "/Users/username/projects/project2"
  ]
}
```

## 保存される内容の詳細

- **repositories**: 
  - `usagi init` が実行されたプロジェクトのルートディレクトリの絶対パスのリスト。
  - プロジェクトが新しく初期化されるたびに、そのディレクトリが既になければ追加されます。
  - このリストにより、ツールはシステム上のどこに `usagi` プロジェクトが存在するかを把握できます。
