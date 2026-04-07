use anyhow::Result;
use std::path::PathBuf;
use crate::usecase::initialize;

/// Handles the `usagi init` CLI command by delegating to the initialize use case.
pub fn run(
    repository_url: &str,
    directory: Option<PathBuf>,
    branch: Option<String>,
) -> Result<()> {
    initialize::run(repository_url, directory, branch)
}
