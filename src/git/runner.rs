use anyhow::{Context, Result, bail};
use std::process::Command;

/// Execute a git command with the given arguments and return stdout.
/// Fails with a descriptive error if the command exits non-zero.
pub fn run_git(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .context("Failed to execute git command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout)
}

/// Check if the current directory is inside a git repository.
pub fn is_git_repo() -> bool {
    run_git(&["rev-parse", "--is-inside-work-tree"]).is_ok()
}

/// Get the repository root path.
pub fn repo_root() -> Result<String> {
    let out = run_git(&["rev-parse", "--show-toplevel"])?;
    Ok(out.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_git_version() {
        let result = run_git(&["--version"]);
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with("git version"));
    }
}
