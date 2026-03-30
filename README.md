# usagi.ai

A tool designed for efficient use of AI Agent CLIs.

## Overview

`usagi.ai` helps developers manage repositories and configurations when working with AI agents, providing a streamlined workflow for initialization and project management.

## Installation

### From Source (using Cargo)
```bash
cargo install --git https://github.com/KKyosuke/usagi.ai
```

### From GitHub Releases
Download the latest binary for your OS from the [Releases](https://github.com/KKyosuke/usagi.ai/releases) page.
Unzip (if needed) and move the binary to a directory in your PATH.

## Quick Start

Initialize a repository with:

```bash
usagi init <repository-url>
```

For more details on initialization, see [doc/init.md](doc/command/init.md).


## Project Structure

When you run `usagi init`, the following structure is created:

- `root/`
  - `main/`: The repository is cloned here. The directory name is based on the default branch name (with `/` converted to `-`).
  - `usagi.config`: A configuration file is automatically generated.
