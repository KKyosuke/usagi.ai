# usagi.ai

A tool designed for efficient use of AI Agent CLIs.

## Overview

`usagi.ai` helps developers manage repositories and configurations when working with AI agents, providing a streamlined workflow for initialization and project management.

## Prerequisites

`usagi.ai` requires the following tools to be installed on your system:

- **Git** (Essential): Used for cloning repositories and managing worktrees.
- **Bash** (macOS/Linux) or **cmd.exe** (Windows) (Essential): Used for the interactive terminal.
- **Node.js / npm** (Optional): Often required by AI agents and development environments.
- **Python** (Optional): Often required by AI agents and scripts.

You can check if these are installed by running:

```bash
usagi doctor
```

## Installation

### From Source (using Cargo)
```bash
cargo install --git https://github.com/KKyosuke/usagi.ai
```

### Using cargo-binstall

If you have [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) installed, you can install the binary directly from GitHub:

```bash
cargo binstall --git https://github.com/KKyosuke/usagi.ai usagi
```

### From GitHub Releases

You can install `usagi` with a single command (macOS and Linux):

```bash
curl -fsSL https://raw.githubusercontent.com/KKyosuke/usagi.ai/main/scripts/install.sh | bash
```

Alternatively, you can download the binary for your platform and install it with the following commands:

#### macOS (Apple Silicon)
```bash
curl -L https://github.com/KKyosuke/usagi.ai/releases/latest/download/usagi-macos-arm64.tar.gz | tar -xz && ./install.sh && rm install.sh
```

#### macOS (Intel)
```bash
curl -L https://github.com/KKyosuke/usagi.ai/releases/latest/download/usagi-macos-amd64.tar.gz | tar -xz && ./install.sh && rm install.sh
```

#### Linux (AMD64)
```bash
curl -L https://github.com/KKyosuke/usagi.ai/releases/latest/download/usagi-linux-amd64.tar.gz | tar -xz && ./install.sh && rm install.sh
```

#### Windows (AMD64)
Download the latest `usagi-windows-amd64.zip` from the [Releases](https://github.com/KKyosuke/usagi.ai/releases) page, extract it, and run `install.sh` (using Git Bash) or manually add the binary to your PATH.

## Quick Start

### 1. Check Dependencies

Ensure your system has the required tools:

```bash
usagi doctor
```

For more details on dependencies, see [doc/cli/doctor.md](doc/cli/doctor.md).

### 2. Initialize a Repository

Initialize a repository with:

```bash
usagi init <repository-url>
```

For more details on initialization, see [doc/cli/init.md](doc/cli/init.md).


## Project Structure

When you run `usagi init`, the following structure is created:

- `root/`
  - `main/`: The repository is cloned here. The directory name is based on the default branch name (with `/` converted to `-`).
  - `usagi.config`: A configuration file is automatically generated.
