# Creating a Workspace (`start`)

The `start` command is used to create a new workspace by using a Git worktree. This allows you to work on multiple branches simultaneously in separate directories.

## Usage

```bash
usagi start <new-branch-name> [origin-branch-name]
```

Replace `<new-branch-name>` with the name of the new branch you want to create. The optional `[origin-branch-name]` specifies the source branch; if omitted, the remote's default branch will be used.

## How it Works

When you run the `start` command, the following steps are performed:

### 1. New Workspace Creation

The command uses `git worktree` to create a new workspace. This means that a new directory is created where the repository is checked out with the specified branch.

### 2. Branch Check-out

The `start` command behaves similarly to `git checkout -b <new-branch-name> <origin-branch-name>`. It creates a new branch named `<new-branch-name>` based on `<origin-branch-name>` (or the default branch) and switches to it within the new workspace.

### 3. Upstream Configuration

The command internally uses a `-force` flag to ensure that the new branch does not automatically maintain or follow the upstream tracking of the original branch.

## Directory Structure Example

After running `usagi start`, your project structure will expand to include the new workspace:

```text
.
├── main/             # Initial cloned repository
├── <new-branch>/     # New workspace created for the branch
└── usagi.config      # Project configuration
```

Note: Forward slashes (`/`) in the branch name are automatically converted to hyphens (`-`) for the directory name to ensure compatibility with all file systems.
