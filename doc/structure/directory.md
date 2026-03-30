# Project Directory Structure

When a project is initialized using `usagi init`, the following directory structure is created in the project root:

## Overview

```text
.
├── .usagi/             # Internal management directory (used by usagi.ai)
│   └── state.json      # Internal state file (initialization flag, worktrees list, etc.)
├── main/               # The primary working directory where the repository is cloned
├── usagi.config        # Project configuration file (e.g., repository URL)
└── .gitignore          # Updated to ignore the .usagi/ management directory
```

## Component Details

### `.usagi/`
A hidden directory that stores internal state and data for the `usagi.ai` tool. This directory is not meant to be modified manually by users.

- **`state.json`**: This file tracks the project's current state, including whether it has been initialized (`initialized: true`) and a list of worktrees that have been created within the project.

### `main/`
The main working directory of the project. When `usagi init` is executed, the target Git repository is cloned into this directory. This serves as the primary location for your AI-assisted development tasks.

### `usagi.config`
A configuration file for the project. It stores project-specific settings, such as the `repository_url`. This file can be viewed and edited by users to configure the project's behavior.

### `.gitignore`
The project's `.gitignore` file is automatically updated during initialization to include `.usagi/`. This ensures that the tool's internal management files are not tracked by Git. If a `.gitignore` file already exists, the entry is appended; otherwise, a new file is created.
