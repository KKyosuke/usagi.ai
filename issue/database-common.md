### 共通設定・データの保持場所

ユーザー全体で共有される設定や、管理している全レポジトリの情報は、OS の標準的なディレクトリ（現代的な標準）に保持します。

*   **macOS / Linux (XDG Base Directory Specification に準拠):**
    *   設定ファイル: `~/.config/usagi/`
    *   データファイル: `~/.local/share/usagi/`
*   **Windows:**
    *   設定・データ: `C:\Users\<ユーザー名>\AppData\Roaming\usagi\`

#### 保持する情報の例
*   `repositories.json`: 管理下にあるプロジェクトのパス一覧
*   `global.config`: ユーザー共通設定 (APIキーなど)
