//! Merge conflict detection, parsing, and resolution helpers.

use super::runner::run_git;
use anyhow::{bail, Result};
use std::fs;
use std::path::Path;

// ─── Types ─────────────────────────────────────────────────────

/// Describes which kind of merge operation is currently in progress.
#[derive(Debug, Clone, PartialEq)]
pub enum MergeType {
    Merge,
    Rebase,
    CherryPick,
}

impl std::fmt::Display for MergeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MergeType::Merge => write!(f, "merge"),
            MergeType::Rebase => write!(f, "rebase"),
            MergeType::CherryPick => write!(f, "cherry-pick"),
        }
    }
}

/// The current merge state of the repository (if any).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MergeState {
    pub merge_type: MergeType,
    pub head_name: String,
    pub merge_head: Option<String>,
}

/// A single conflict region within a file (between conflict markers).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConflictRegion {
    /// Line number (1-based) where the conflict starts (<<<<<<< line).
    pub start_line: usize,
    /// Line number (1-based) where the conflict ends (>>>>>>> line).
    pub end_line: usize,
    /// Content from the current branch (HEAD / ours).
    pub current: Vec<String>,
    /// Content from the incoming branch (theirs).
    pub incoming: Vec<String>,
    /// Optional ancestor content (from diff3 style), if available.
    pub ancestor: Option<Vec<String>>,
    /// The marker label for current (e.g., "HEAD").
    pub current_label: String,
    /// The marker label for incoming (e.g., "feature-branch").
    pub incoming_label: String,
}

/// Parsed conflict data for a single file.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConflictFile {
    pub path: String,
    /// Raw file content with conflict markers.
    pub raw_content: String,
    /// Parsed conflict regions.
    pub regions: Vec<ConflictRegion>,
    /// Non-conflicting lines (for context).
    pub total_lines: usize,
}

// ─── Merge State Detection ─────────────────────────────────────

/// Detect if a merge, rebase, or cherry-pick is currently in progress.
/// Returns None if no merge operation is active.
pub fn get_merge_state() -> Option<MergeState> {
    // Find the .git directory (handles worktrees and submodules)
    let git_dir = run_git(&["rev-parse", "--git-dir"]).ok()?;
    let git_dir = git_dir.trim();

    let head_name = run_git(&["rev-parse", "--abbrev-ref", "HEAD"])
        .unwrap_or_else(|_| "HEAD".to_string())
        .trim()
        .to_string();

    // Check for merge in progress (.git/MERGE_HEAD exists)
    let merge_head_path = Path::new(git_dir).join("MERGE_HEAD");
    if merge_head_path.exists() {
        let merge_head = fs::read_to_string(&merge_head_path)
            .ok()
            .map(|s| s.trim().to_string());
        return Some(MergeState {
            merge_type: MergeType::Merge,
            head_name,
            merge_head,
        });
    }

    // Check for rebase in progress
    let rebase_merge = Path::new(git_dir).join("rebase-merge");
    let rebase_apply = Path::new(git_dir).join("rebase-apply");
    if rebase_merge.exists() || rebase_apply.exists() {
        return Some(MergeState {
            merge_type: MergeType::Rebase,
            head_name,
            merge_head: None,
        });
    }

    // Check for cherry-pick in progress
    let cherry_pick_head = Path::new(git_dir).join("CHERRY_PICK_HEAD");
    if cherry_pick_head.exists() {
        let cp_head = fs::read_to_string(&cherry_pick_head)
            .ok()
            .map(|s| s.trim().to_string());
        return Some(MergeState {
            merge_type: MergeType::CherryPick,
            head_name,
            merge_head: cp_head,
        });
    }

    None
}

// ─── Conflict Parsing ──────────────────────────────────────────

/// Read a conflicted file and parse its conflict markers into structured regions.
pub fn get_conflict_file(file_path: &str) -> Result<ConflictFile> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| anyhow::anyhow!("Cannot read conflicted file '{}': {}", file_path, e))?;

    let regions = parse_conflict_markers(&content);
    let total_lines = content.lines().count();

    Ok(ConflictFile {
        path: file_path.to_string(),
        raw_content: content,
        regions,
        total_lines,
    })
}

/// Parse conflict markers in file content into structured ConflictRegion entries.
///
/// Handles both standard and diff3 conflict styles:
///   Standard: <<<<<<< HEAD / ======= / >>>>>>> branch
///   Diff3:    <<<<<<< HEAD / ||||||| base / ======= / >>>>>>> branch
pub fn parse_conflict_markers(content: &str) -> Vec<ConflictRegion> {
    let mut regions = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        if lines[i].starts_with("<<<<<<<") {
            let start_line = i + 1; // 1-based
            let current_label = lines[i]
                .strip_prefix("<<<<<<< ")
                .unwrap_or("HEAD")
                .to_string();

            let mut current = Vec::new();
            let mut incoming = Vec::new();
            let mut ancestor: Option<Vec<String>> = None;
            let mut in_section = "current"; // current, ancestor, incoming
            let incoming_label;

            i += 1;
            while i < lines.len() {
                if lines[i].starts_with("|||||||") {
                    // diff3 ancestor marker
                    ancestor = Some(Vec::new());
                    in_section = "ancestor";
                    i += 1;
                    continue;
                }
                if lines[i].starts_with("=======") {
                    in_section = "incoming";
                    i += 1;
                    continue;
                }
                if lines[i].starts_with(">>>>>>>") {
                    incoming_label = lines[i]
                        .strip_prefix(">>>>>>> ")
                        .unwrap_or("incoming")
                        .to_string();
                    let end_line = i + 1; // 1-based

                    regions.push(ConflictRegion {
                        start_line,
                        end_line,
                        current,
                        incoming,
                        ancestor,
                        current_label: current_label.clone(),
                        incoming_label,
                    });
                    break;
                }

                match in_section {
                    "current" => current.push(lines[i].to_string()),
                    "ancestor" => {
                        if let Some(ref mut anc) = ancestor {
                            anc.push(lines[i].to_string());
                        }
                    }
                    "incoming" => incoming.push(lines[i].to_string()),
                    _ => {}
                }
                i += 1;
            }
        }
        i += 1;
    }

    regions
}

// ─── Conflict Resolution ───────────────────────────────────────

/// Resolve a conflict in a file by replacing its content and staging it.
pub fn resolve_file(file_path: &str, resolved_content: &str) -> Result<()> {
    fs::write(file_path, resolved_content)
        .map_err(|e| anyhow::anyhow!("Failed to write resolved file '{}': {}", file_path, e))?;

    run_git(&["add", file_path])?;
    Ok(())
}

/// Resolve a specific conflict region by choosing a side.
/// `choice` is "current", "incoming", or the full resolved text for "merge_both".
pub fn resolve_region(file_path: &str, region: &ConflictRegion, choice: &str) -> Result<String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| anyhow::anyhow!("Cannot read file '{}': {}", file_path, e))?;

    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();

    let start = region.start_line - 1; // Convert to 0-based (this is the <<<<<<< line)
    let end = region.end_line - 1; // The >>>>>>> line (0-based)

    // Copy lines before the conflict
    for line in &lines[..start] {
        result.push(*line);
    }

    // Insert the resolution
    match choice {
        "current" => {
            for line in &region.current {
                result.push(line);
            }
        }
        "incoming" => {
            for line in &region.incoming {
                result.push(line);
            }
        }
        resolved => {
            // Custom merged content — split by newlines
            for line in resolved.lines() {
                result.push(line);
            }
        }
    }

    // Copy lines after the conflict
    if end + 1 < lines.len() {
        for line in &lines[end + 1..] {
            result.push(*line);
        }
    }

    let new_content = result.join("\n");
    // Add trailing newline if the original had one
    let new_content = if content.ends_with('\n') {
        format!("{}\n", new_content)
    } else {
        new_content
    };

    Ok(new_content)
}

// ─── Merge Operations ──────────────────────────────────────────

/// Abort the current merge operation.
pub fn abort_merge() -> Result<()> {
    match get_merge_state() {
        Some(state) => match state.merge_type {
            MergeType::Merge => {
                run_git(&["merge", "--abort"])?;
                Ok(())
            }
            MergeType::Rebase => {
                run_git(&["rebase", "--abort"])?;
                Ok(())
            }
            MergeType::CherryPick => {
                run_git(&["cherry-pick", "--abort"])?;
                Ok(())
            }
        },
        None => bail!("No merge operation in progress"),
    }
}

/// Continue the current merge operation (after resolving all conflicts).
pub fn continue_merge() -> Result<()> {
    match get_merge_state() {
        Some(state) => match state.merge_type {
            MergeType::Merge => {
                // For merge, just commit (all conflicts must be resolved and staged)
                run_git(&["commit", "--no-edit"])?;
                Ok(())
            }
            MergeType::Rebase => {
                run_git(&["rebase", "--continue"])?;
                Ok(())
            }
            MergeType::CherryPick => {
                run_git(&["cherry-pick", "--continue"])?;
                Ok(())
            }
        },
        None => bail!("No merge operation in progress"),
    }
}

/// Get the merge base between HEAD and another branch/ref.
#[allow(dead_code)]
pub fn get_merge_base(other_ref: &str) -> Result<String> {
    let output = run_git(&["merge-base", "HEAD", other_ref])?;
    Ok(output.trim().to_string())
}

/// Get the diff between two refs to preview what a merge would bring in.
#[allow(dead_code)]
pub fn get_merge_preview_diff(other_ref: &str) -> Result<String> {
    let base = get_merge_base(other_ref)?;
    let output = run_git(&["diff", "--stat", &base, other_ref])?;
    Ok(output)
}

/// Check if a merge with another ref would have conflicts (dry run).
/// Returns the list of conflicting file paths, or empty if clean merge.
///
/// Uses `git merge-tree` (git 2.38+) which operates on tree objects without
/// touching the working tree — safe even if interrupted.
/// Falls back to the old approach on older git versions.
#[allow(dead_code)]
pub fn check_merge_conflicts(other_ref: &str) -> Result<Vec<String>> {
    // Try git merge-tree first (safe: no worktree modification)
    let merge_tree_result = run_git(&["merge-tree", "--write-tree", "--no-messages", "HEAD", other_ref]);

    match merge_tree_result {
        Ok(output) => {
            // Exit code 0 = clean merge, exit code non-zero with output = conflicts
            // If merge-tree succeeds with just a tree hash, it's a clean merge
            if output.trim().len() == 40 || output.trim().len() == 64 {
                // Clean merge — just a tree hash, no conflicts
                return Ok(Vec::new());
            }
            // Parse conflict file paths from merge-tree output
            let conflicts: Vec<String> = output
                .lines()
                .filter(|line| {
                    // merge-tree outputs conflict info lines
                    line.contains("CONFLICT") || line.starts_with("Auto-merging")
                })
                .filter_map(|line| {
                    // Extract file path from lines like "CONFLICT (content): Merge conflict in file.rs"
                    line.rsplit(" in ").next().map(|s| s.trim().to_string())
                })
                .collect();
            Ok(conflicts)
        }
        Err(e) => {
            let err_str = e.to_string();
            // merge-tree on older git or with conflicts reports via stderr
            if err_str.contains("CONFLICT") || err_str.contains("Automatic merge failed") {
                let conflicts: Vec<String> = err_str
                    .lines()
                    .filter(|line| line.contains("CONFLICT"))
                    .filter_map(|line| {
                        line.rsplit(" in ").next().map(|s| s.trim().to_string())
                    })
                    .collect();
                Ok(conflicts)
            } else if err_str.contains("not a git command") || err_str.contains("merge-tree") {
                // Fallback for git < 2.38: use merge --no-commit --no-ff
                log::warn!("git merge-tree not available, falling back to merge --no-commit");
                check_merge_conflicts_legacy(other_ref)
            } else {
                Err(e)
            }
        }
    }
}

/// Legacy fallback for git < 2.38: check conflicts by doing a real merge then aborting.
/// This modifies the worktree temporarily and is NOT safe if interrupted.
fn check_merge_conflicts_legacy(other_ref: &str) -> Result<Vec<String>> {
    let result = run_git(&["merge", "--no-commit", "--no-ff", other_ref]);
    match result {
        Ok(_) => {
            let _ = run_git(&["merge", "--abort"]);
            Ok(Vec::new())
        }
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("CONFLICT") || err_str.contains("Automatic merge failed") {
                let status = super::status::get_status().unwrap_or_default();
                let conflicts: Vec<String> =
                    status.conflicts.iter().map(|f| f.path.clone()).collect();
                let _ = run_git(&["merge", "--abort"]);
                Ok(conflicts)
            } else {
                let _ = run_git(&["merge", "--abort"]);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_conflict_markers_standard() {
        let content = "\
before conflict
<<<<<<< HEAD
current line 1
current line 2
=======
incoming line 1
>>>>>>> feature-branch
after conflict
";
        let regions = parse_conflict_markers(content);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].current, vec!["current line 1", "current line 2"]);
        assert_eq!(regions[0].incoming, vec!["incoming line 1"]);
        assert_eq!(regions[0].current_label, "HEAD");
        assert_eq!(regions[0].incoming_label, "feature-branch");
        assert!(regions[0].ancestor.is_none());
    }

    #[test]
    fn test_parse_conflict_markers_diff3() {
        let content = "\
<<<<<<< HEAD
current
||||||| base
original
=======
incoming
>>>>>>> other
";
        let regions = parse_conflict_markers(content);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].current, vec!["current"]);
        assert_eq!(regions[0].incoming, vec!["incoming"]);
        assert!(regions[0].ancestor.is_some());
        assert_eq!(regions[0].ancestor.as_ref().unwrap(), &vec!["original"]);
    }

    #[test]
    fn test_parse_conflict_markers_multiple() {
        let content = "\
<<<<<<< HEAD
a
=======
b
>>>>>>> branch1
middle
<<<<<<< HEAD
c
=======
d
>>>>>>> branch2
";
        let regions = parse_conflict_markers(content);
        assert_eq!(regions.len(), 2);
        assert_eq!(regions[0].current, vec!["a"]);
        assert_eq!(regions[0].incoming, vec!["b"]);
        assert_eq!(regions[1].current, vec!["c"]);
        assert_eq!(regions[1].incoming, vec!["d"]);
    }

    #[test]
    fn test_parse_no_conflicts() {
        let content = "normal file content\nno conflicts here\n";
        let regions = parse_conflict_markers(content);
        assert!(regions.is_empty());
    }
}
