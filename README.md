# usagi.ai

A tool designed for efficient use of AI Agent CLIs.

## Overview

`usagi.ai` helps developers manage repositories and configurations when working with AI agents, providing a streamlined workflow for initialization and project management.

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
