# usagi.ai

A tool designed for efficient use of AI Agent CLIs.

## Overview

`usagi.ai` helps developers manage repositories and configurations when working with AI agents, providing a streamlined workflow for initialization and project management.

## Installation

(Add installation instructions here when available)

## Quick Start

Initialize a repository with:

```bash
usagi init <repository-url>
```

For more details on initialization, see [doc/init.md](doc/init.md).

### Start a new workspace

Create a new workspace for a branch with:

```bash
usagi start <new-branch-name> [origin-branch-name]
```

For more details on creating workspaces, see [doc/start.md](doc/start.md).

## Project Structure

When you run `usagi init`, the following structure is created:

- `root/`
  - `main/`: The repository is cloned here. The directory name is based on the default branch name (with `/` converted to `-`).
  - `usagi.config`: A configuration file is automatically generated.
