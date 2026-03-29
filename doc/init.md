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
├── main/             # Cloned repository (named after branch)
│   └── (repo contents)
└── usagi.config      # Automatically generated configuration
```
