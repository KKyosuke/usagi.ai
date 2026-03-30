# Initialization (`init`)

The `init` command is used to set up the working directory for a project, including cloning the target repository and generating the necessary configuration files.

## Usage

```bash
usagi init <repository-url>
```

Replace `<repository-url>` with the URL of the Git repository you want to use with your AI agent.

## How it Works

When you run the `init` command, the following steps are performed:

### 1. Repository Cloning

The specified repository is cloned into a `main/` directory within the project root.

- **Directory Name:** The directory name is derived from the **default branch name**.
- **Transformation:** Any forward slashes (`/`) in the branch name are automatically converted to hyphens (`-`) to ensure compatibility with all file systems.

Example: If the default branch is `feature/main-task`, the repository will be cloned into `main/feature-main-task`.

### 2. Configuration Generation

An `usagi.config` file is generated in the root directory. This file holds the configuration settings required for efficient CLI usage with AI agents.

## Directory Structure Example

After running `usagi init`, your directory will look like this:

```text
.
├── .usagi/             # 管理用ディレクトリ（ツール内部で使用）
│   └── state.json      # 内部状態（初期化フラグ、ワークツリー一覧など）
├── main/               # クローンされたリポジトリ（現在はディレクトリ作成のみ）
├── usagi.config        # プロジェクトの設定ファイル（リポジトリURLなど）
└── .gitignore          # .usagi/ ディレクトリを無視するための設定
```

### File/Directory Details

- **`.usagi/`**: `usagi` CLI がプロジェクトの管理状態を保持するための隠しディレクトリです。
    - **`state.json`**: プロジェクトが初期化済みかどうか (`initialized: true`) や、今後作成されるワークツリーのリストなどを管理する JSON ファイルです。
- **`main/`**: 指定したリポジトリがクローンされるメインの作業ディレクトリです。
- **`usagi.config`**: ユーザーが参照・編集可能な設定ファイルです。初期状態では、`init` 時に指定したリポジトリの URL が書き込まれます。
- **`.gitignore`**: Git で管理する場合に、ツール内部の状態を保持する `.usagi/` ディレクトリを無視するように設定が追記されます。既存のファイルがある場合は追記され、ない場合は新規作成されます。
