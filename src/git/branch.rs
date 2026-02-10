use anyhow::Result;
use super::runner::run_git;

#[derive(Debug, Clone)]
pub struct BranchEntry {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub last_commit_msg: String,
    pub last_commit_date: String,
    pub last_commit_author: String,
    pub upstream: String,
}

pub struct BranchOps;

impl BranchOps {
    /// List all branches (local + remote).
    pub fn list() -> Result<Vec<BranchEntry>> {
        let format = "%(if)%(HEAD)%(then)*%(else) %(end)\x1f%(refname:short)\x1f%(upstream:short)\x1f%(subject)\x1f%(authorname)\x1f%(committerdate:relative)";
        let output = run_git(&["branch", "-a", "--format", format])?;

        let mut branches = Vec::new();
        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split('\x1f').collect();
            if parts.len() < 6 {
                continue;
            }

            let is_current = parts[0].trim() == "*";
            let name = parts[1].trim().to_string();
            let is_remote = name.starts_with("remotes/") || name.contains('/');

            branches.push(BranchEntry {
                name,
                is_current,
                is_remote,
                upstream: parts[2].trim().to_string(),
                last_commit_msg: parts[3].trim().to_string(),
                last_commit_author: parts[4].trim().to_string(),
                last_commit_date: parts[5].trim().to_string(),
            });
        }

        Ok(branches)
    }

    /// Create a new branch from HEAD or a specific commit.
    pub fn create(name: &str, from: Option<&str>) -> Result<()> {
        let mut args = vec!["branch", name];
        if let Some(base) = from {
            args.push(base);
        }
        run_git(&args)?;
        Ok(())
    }

    /// Switch to a branch.
    pub fn switch(name: &str) -> Result<()> {
        run_git(&["switch", name])?;
        Ok(())
    }

    /// Delete a local branch with safety check.
    pub fn delete(name: &str, force: bool) -> Result<()> {
        let flag = if force { "-D" } else { "-d" };
        run_git(&["branch", flag, name])?;
        Ok(())
    }

    /// Rename the current branch.
    pub fn rename(new_name: &str) -> Result<()> {
        run_git(&["branch", "-m", new_name])?;
        Ok(())
    }

    /// Get the current branch name.
    pub fn current() -> Result<String> {
        let output = run_git(&["branch", "--show-current"])?;
        Ok(output.trim().to_string())
    }

    /// Check if there are uncommitted changes.
    pub fn has_uncommitted_changes() -> Result<bool> {
        let output = run_git(&["status", "--porcelain"])?;
        Ok(!output.trim().is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_branch() {
        // This test only works inside a git repo
        if run_git(&["rev-parse", "--is-inside-work-tree"]).is_ok() {
            let result = BranchOps::current();
            assert!(result.is_ok());
        }
    }
}
