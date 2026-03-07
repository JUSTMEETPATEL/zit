//! Git bisect — binary search for the commit that introduced a bug.

use super::runner::run_git;
use anyhow::Result;
use std::path::Path;

/// Current phase of a bisect session.
#[derive(Debug, Clone, PartialEq)]
pub enum BisectPhase {
    /// No bisect session active.
    Inactive,
    /// Bisect is in progress — user is marking commits good/bad.
    InProgress {
        /// Approximate number of steps remaining.
        steps_remaining: usize,
        /// Number of revisions left to test.
        revisions_left: usize,
        /// The commit currently being tested.
        current_commit: String,
    },
    /// Bisect has found the offending commit.
    Found {
        /// The first bad commit hash.
        commit_hash: String,
        /// One-line summary of the commit.
        summary: String,
    },
}

/// A single entry from bisect log.
#[derive(Debug, Clone)]
pub struct BisectLogEntry {
    pub hash: String,
    pub verdict: String, // "good", "bad", "skip"
    pub message: String,
}

/// Check whether a bisect session is currently active.
pub fn is_bisecting() -> bool {
    // Git stores bisect state in .git/BISECT_LOG
    let output = run_git(&["rev-parse", "--git-dir"]);
    if let Ok(git_dir) = output {
        let bisect_log = Path::new(git_dir.trim()).join("BISECT_LOG");
        bisect_log.exists()
    } else {
        false
    }
}

/// Start a bisect session.
/// `bad` is the known-bad commit (e.g. "HEAD"), `good` is the known-good commit.
pub fn bisect_start(bad: &str, good: &str) -> Result<String> {
    run_git(&["bisect", "start", bad, good])
}

/// Mark the current commit as good.
pub fn bisect_good() -> Result<String> {
    run_git(&["bisect", "good"])
}

/// Mark the current commit as bad.
pub fn bisect_bad() -> Result<String> {
    run_git(&["bisect", "bad"])
}

/// Skip the current commit (cannot test it).
pub fn bisect_skip() -> Result<String> {
    run_git(&["bisect", "skip"])
}

/// Reset / end the bisect session and return to the original branch.
pub fn bisect_reset() -> Result<String> {
    run_git(&["bisect", "reset"])
}

/// Get the bisect log as raw text.
pub fn bisect_log() -> Result<String> {
    run_git(&["bisect", "log"])
}

/// Parse the current bisect status from git output.
pub fn bisect_status() -> BisectPhase {
    if !is_bisecting() {
        return BisectPhase::Inactive;
    }

    // Check if bisect already found the bad commit
    if let Ok(log) = bisect_log() {
        // If log contains "first bad commit", bisect is done
        for line in log.lines() {
            if line.contains("is the first bad commit") {
                let hash = line
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_start_matches('#')
                    .to_string();
                let summary = get_commit_oneline(&hash).unwrap_or_default();
                return BisectPhase::Found {
                    commit_hash: hash,
                    summary,
                };
            }
        }
    }

    // Bisect is in progress — try to get remaining steps
    let current_commit = run_git(&["rev-parse", "--short", "HEAD"])
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    // `git bisect visualize --oneline` can tell us roughly how many are left
    let (steps, revs) = estimate_remaining();

    BisectPhase::InProgress {
        steps_remaining: steps,
        revisions_left: revs,
        current_commit,
    }
}

/// Get a one-line description of a commit.
fn get_commit_oneline(hash: &str) -> Result<String> {
    let output = run_git(&["log", "-1", "--format=%h %s", hash])?;
    Ok(output.trim().to_string())
}

/// Estimate remaining bisect steps.
fn estimate_remaining() -> (usize, usize) {
    // `git bisect visualize --oneline` lists remaining commits
    if let Ok(output) = run_git(&["bisect", "visualize", "--oneline"]) {
        let count = output.lines().count();
        if count == 0 {
            return (0, 0);
        }
        // steps ≈ log2(count)
        let steps = (count as f64).log2().ceil() as usize;
        return (steps, count);
    }
    (0, 0)
}

/// Parse bisect log into structured entries.
pub fn parse_bisect_log() -> Vec<BisectLogEntry> {
    let log = match bisect_log() {
        Ok(l) => l,
        Err(_) => return Vec::new(),
    };

    let mut entries = Vec::new();
    for line in log.lines() {
        let line = line.trim();
        // Lines look like: "# good: [abc1234] commit message"
        // or "git bisect good abc1234..."
        if line.starts_with("# good:") || line.starts_with("# bad:") || line.starts_with("# skip:")
        {
            let verdict = if line.starts_with("# good:") {
                "good"
            } else if line.starts_with("# bad:") {
                "bad"
            } else {
                "skip"
            };
            // Extract hash from [hash] or after the colon
            let rest = line.split_once(':').map(|x| x.1).unwrap_or("").trim();
            let (hash, message) = if rest.starts_with('[') {
                let end = rest.find(']').unwrap_or(rest.len());
                (rest[1..end].to_string(), rest[end + 1..].trim().to_string())
            } else {
                let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                (
                    parts.first().unwrap_or(&"").to_string(),
                    parts.get(1).unwrap_or(&"").to_string(),
                )
            };
            entries.push(BisectLogEntry {
                hash,
                verdict: verdict.to_string(),
                message,
            });
        }
    }
    entries
}

/// Get a list of recent commit hashes for the start-bisect picker.
pub fn recent_commits_for_picker(count: usize) -> Result<Vec<(String, String)>> {
    let format_str = "--format=%H %s";
    let count_str = format!("-{}", count);
    let output = run_git(&["log", &count_str, format_str])?;
    let mut commits = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (hash, msg) = line.split_once(' ').unwrap_or((line, ""));
        commits.push((hash.to_string(), msg.to_string()));
    }
    Ok(commits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bisect_phase_inactive_default() {
        // Outside a git repo, is_bisecting should return false
        let phase = bisect_status();
        assert_eq!(phase, BisectPhase::Inactive);
    }

    #[test]
    fn test_parse_empty_log() {
        let entries = parse_bisect_log();
        // Should gracefully return empty when no bisect is running
        assert!(entries.is_empty());
    }
}
