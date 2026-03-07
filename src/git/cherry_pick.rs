//! Git cherry-pick — apply specific commits from one branch to another.

use super::log::CommitEntry;
use super::runner::run_git;
use anyhow::Result;
use std::path::Path;

/// Cherry-pick a single commit onto the current branch.
pub fn cherry_pick(commit_hash: &str) -> Result<String> {
    run_git(&["cherry-pick", commit_hash])
}

/// Cherry-pick multiple commits (in order).
pub fn cherry_pick_multiple(commit_hashes: &[&str]) -> Result<String> {
    let mut args = vec!["cherry-pick"];
    args.extend_from_slice(commit_hashes);
    run_git(&args)
}

/// Abort an in-progress cherry-pick.
pub fn cherry_pick_abort() -> Result<String> {
    run_git(&["cherry-pick", "--abort"])
}

/// Continue a cherry-pick after resolving conflicts.
pub fn cherry_pick_continue() -> Result<String> {
    run_git(&["cherry-pick", "--continue"])
}

/// Check if a cherry-pick is currently in progress.
pub fn is_cherry_picking() -> bool {
    if let Ok(git_dir) = run_git(&["rev-parse", "--git-dir"]) {
        let cp_head = Path::new(git_dir.trim()).join("CHERRY_PICK_HEAD");
        cp_head.exists()
    } else {
        false
    }
}

/// Get local branch names (excluding the current one) for source selection.
pub fn list_source_branches() -> Result<Vec<String>> {
    let output = run_git(&["branch", "--format=%(refname:short)"])?;
    let current = current_branch().unwrap_or_default();
    let branches: Vec<String> = output
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|b| !b.is_empty() && *b != current)
        .collect();
    Ok(branches)
}

/// Get the current branch name.
fn current_branch() -> Result<String> {
    let output = run_git(&["rev-parse", "--abbrev-ref", "HEAD"])?;
    Ok(output.trim().to_string())
}

/// Get the current branch name (public).
pub fn get_current_branch() -> String {
    current_branch().unwrap_or_else(|_| "HEAD".to_string())
}

/// Get commits unique to `source_branch` that are not on the current branch.
/// These are the commits eligible for cherry-picking.
pub fn get_cherry_candidates(source_branch: &str, count: usize) -> Result<Vec<CommitEntry>> {
    let range = format!("HEAD..{}", source_branch);
    let count_str = format!("-{}", count);
    let format_str = "--format=%H\x1f%h\x1f%s\x1f%an\x1f%ar";
    let output = run_git(&["log", &count_str, format_str, &range])?;

    let mut entries = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\x1f').collect();
        if parts.len() >= 5 {
            entries.push(CommitEntry {
                hash: parts[0].to_string(),
                short_hash: parts[1].to_string(),
                message: parts[2].to_string(),
                author: parts[3].to_string(),
                date: parts[4].to_string(),
                date_iso: String::new(),
                parents: Vec::new(),
                refs: String::new(),
                graph: String::new(),
            });
        }
    }
    Ok(entries)
}

/// Show the diff for a specific commit (for preview).
pub fn commit_diff(commit_hash: &str) -> Result<String> {
    run_git(&["show", "--stat", "--patch", commit_hash])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_not_cherry_picking_outside_repo() {
        // Should not panic, just return false
        let result = is_cherry_picking();
        assert!(!result);
    }

    #[test]
    fn test_list_source_branches_format() {
        // Verifies the function signature compiles and returns Result
        let _result: Result<Vec<String>> = list_source_branches();
    }
}
