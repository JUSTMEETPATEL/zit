//! Integration tests for zit — uses real git repos in temp directories.
//!
//! These tests create actual git repositories using `git init` and run
//! the same parsing/logic functions that the TUI uses.

use std::process::Command;
use tempfile::TempDir;

/// Helper: run git in a specific directory and return stdout.
fn git(dir: &std::path::Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "Test User")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "Test User")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
        .output()
        .expect("failed to run git");
    String::from_utf8(output.stdout).unwrap()
}

/// Helper: create a temp git repo with an initial commit.
fn init_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    git(dir.path(), &["init", "-b", "main"]);
    std::fs::write(dir.path().join("README.md"), "# Test\n").unwrap();
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-m", "initial commit"]);
    dir
}

// ────────────────────────────────────────────────────────────────────────
// Git status tests
// ────────────────────────────────────────────────────────────────────────

#[test]
fn test_clean_repo_status() {
    let dir = init_repo();
    let output = git(dir.path(), &["status", "--porcelain=v2", "--branch"]);
    assert!(output.contains("# branch.head main"));
    // No modified/untracked lines in a clean repo
    let has_changes = output
        .lines()
        .any(|l| l.starts_with("1 ") || l.starts_with("? "));
    assert!(!has_changes, "expected clean repo");
}

#[test]
fn test_untracked_file_shows_in_status() {
    let dir = init_repo();
    std::fs::write(dir.path().join("new_file.txt"), "hello").unwrap();
    let output = git(dir.path(), &["status", "--porcelain=v2", "--branch"]);
    assert!(output.contains("? new_file.txt"));
}

#[test]
fn test_staged_file_shows_in_status() {
    let dir = init_repo();
    std::fs::write(dir.path().join("new.txt"), "content").unwrap();
    git(dir.path(), &["add", "new.txt"]);
    let output = git(dir.path(), &["status", "--porcelain=v2", "--branch"]);
    // Should show as ordinary entry with A (added) in index
    let has_added = output.lines().any(|l| l.starts_with("1 A."));
    assert!(has_added, "expected staged file: {}", output);
}

#[test]
fn test_modified_unstaged_shows_in_status() {
    let dir = init_repo();
    // Modify existing tracked file
    std::fs::write(dir.path().join("README.md"), "# Modified\n").unwrap();
    let output = git(dir.path(), &["status", "--porcelain=v2", "--branch"]);
    let has_modified = output.lines().any(|l| l.starts_with("1 .M"));
    assert!(has_modified, "expected unstaged modification: {}", output);
}

// ────────────────────────────────────────────────────────────────────────
// Git diff tests
// ────────────────────────────────────────────────────────────────────────

#[test]
fn test_diff_shows_changes() {
    let dir = init_repo();
    std::fs::write(dir.path().join("README.md"), "# Changed content\n").unwrap();
    let output = git(dir.path(), &["diff"]);
    assert!(output.contains("diff --git"));
    assert!(output.contains("-# Test"));
    assert!(output.contains("+# Changed content"));
}

#[test]
fn test_staged_diff() {
    let dir = init_repo();
    std::fs::write(dir.path().join("README.md"), "staged change\n").unwrap();
    git(dir.path(), &["add", "README.md"]);
    let output = git(dir.path(), &["diff", "--cached"]);
    assert!(output.contains("diff --git"));
    assert!(output.contains("+staged change"));
}

#[test]
fn test_no_diff_when_clean() {
    let dir = init_repo();
    let output = git(dir.path(), &["diff"]);
    assert!(output.trim().is_empty());
}

// ────────────────────────────────────────────────────────────────────────
// Git log tests
// ────────────────────────────────────────────────────────────────────────

#[test]
fn test_log_shows_initial_commit() {
    let dir = init_repo();
    let output = git(dir.path(), &["log", "--oneline"]);
    assert!(output.contains("initial commit"));
}

#[test]
fn test_log_multiple_commits() {
    let dir = init_repo();
    std::fs::write(dir.path().join("file2.txt"), "data").unwrap();
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-m", "second commit"]);
    let output = git(dir.path(), &["log", "--oneline"]);
    let count = output.lines().count();
    assert_eq!(count, 2);
}

// ────────────────────────────────────────────────────────────────────────
// Git branch tests
// ────────────────────────────────────────────────────────────────────────

#[test]
fn test_branch_list() {
    let dir = init_repo();
    let output = git(dir.path(), &["branch"]);
    assert!(output.contains("main"));
}

#[test]
fn test_create_and_switch_branch() {
    let dir = init_repo();
    git(dir.path(), &["checkout", "-b", "feature"]);
    let output = git(dir.path(), &["rev-parse", "--abbrev-ref", "HEAD"]);
    assert_eq!(output.trim(), "feature");
}

#[test]
fn test_delete_branch() {
    let dir = init_repo();
    git(dir.path(), &["branch", "to-delete"]);
    git(dir.path(), &["branch", "-d", "to-delete"]);
    let output = git(dir.path(), &["branch"]);
    assert!(!output.contains("to-delete"));
}

// ────────────────────────────────────────────────────────────────────────
// Git reflog tests
// ────────────────────────────────────────────────────────────────────────

#[test]
fn test_reflog_has_entries() {
    let dir = init_repo();
    let output = git(dir.path(), &["reflog", "--oneline"]);
    assert!(!output.trim().is_empty());
}

#[test]
fn test_reflog_records_branch_switch() {
    let dir = init_repo();
    git(dir.path(), &["checkout", "-b", "dev"]);
    git(dir.path(), &["checkout", "main"]);
    let output = git(dir.path(), &["reflog", "--oneline"]);
    assert!(output.contains("checkout: moving from dev to main"));
}

// ────────────────────────────────────────────────────────────────────────
// Git stash tests
// ────────────────────────────────────────────────────────────────────────

#[test]
fn test_stash_count_zero_on_clean_repo() {
    let dir = init_repo();
    let output = git(dir.path(), &["stash", "list"]);
    assert_eq!(output.lines().count(), 0);
}

#[test]
fn test_stash_creates_entry() {
    let dir = init_repo();
    std::fs::write(dir.path().join("README.md"), "modified for stash\n").unwrap();
    git(dir.path(), &["stash", "push", "-m", "test stash"]);
    let output = git(dir.path(), &["stash", "list"]);
    assert_eq!(output.lines().count(), 1);
    assert!(output.contains("test stash"));
}

// ────────────────────────────────────────────────────────────────────────
// CLI flag tests (binary invocation)
// ────────────────────────────────────────────────────────────────────────

#[test]
fn test_cli_version_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_zit"))
        .arg("--version")
        .output()
        .expect("failed to run zit");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("zit"));
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
    assert!(output.status.success());
}

#[test]
fn test_cli_help_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_zit"))
        .arg("--help")
        .output()
        .expect("failed to run zit");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("USAGE"));
    assert!(stdout.contains("OPTIONS"));
    assert!(stdout.contains("--verbose"));
    assert!(output.status.success());
}

#[test]
fn test_cli_unknown_flag_errors() {
    let output = Command::new(env!("CARGO_BIN_EXE_zit"))
        .arg("--nonexistent")
        .output()
        .expect("failed to run zit");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Unknown option"));
    assert!(!output.status.success());
}

#[test]
fn test_cli_not_git_repo() {
    let dir = TempDir::new().unwrap(); // NOT a git repo
    let output = Command::new(env!("CARGO_BIN_EXE_zit"))
        .current_dir(dir.path())
        .output()
        .expect("failed to run zit");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Not a git repository"));
    assert!(!output.status.success());
}

// ────────────────────────────────────────────────────────────────────────
// --no-ai flag test
// ────────────────────────────────────────────────────────────────────────

#[test]
fn test_cli_no_ai_flag_accepted() {
    let output = Command::new(env!("CARGO_BIN_EXE_zit"))
        .args(["--no-ai", "--help"])
        .output()
        .expect("failed to run zit");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("USAGE"));
    assert!(output.status.success());
}

#[test]
fn test_cli_help_shows_no_ai() {
    let output = Command::new(env!("CARGO_BIN_EXE_zit"))
        .arg("--help")
        .output()
        .expect("failed to run zit");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--no-ai"));
}

// ────────────────────────────────────────────────────────────────────────
// Stash management tests
// ────────────────────────────────────────────────────────────────────────

#[test]
fn test_stash_pop_restores_changes() {
    let dir = init_repo();
    std::fs::write(dir.path().join("README.md"), "stash me\n").unwrap();
    git(dir.path(), &["stash", "push", "-m", "pop test"]);
    // README should be back to original
    let contents = std::fs::read_to_string(dir.path().join("README.md"))
        .unwrap()
        .replace("\r", "");
    assert_eq!(contents, "# Test\n");
    // Pop it
    git(dir.path(), &["stash", "pop"]);
    let contents = std::fs::read_to_string(dir.path().join("README.md"))
        .unwrap()
        .replace("\r", "");
    assert_eq!(contents, "stash me\n");
}

#[test]
fn test_stash_apply_keeps_entry() {
    let dir = init_repo();
    std::fs::write(dir.path().join("README.md"), "apply me\n").unwrap();
    git(dir.path(), &["stash", "push", "-m", "apply test"]);
    git(dir.path(), &["stash", "apply"]);
    // Stash should still exist
    let list = git(dir.path(), &["stash", "list"]);
    assert_eq!(list.lines().count(), 1);
    // Changes should be restored
    let contents = std::fs::read_to_string(dir.path().join("README.md"))
        .unwrap()
        .replace("\r", "");
    assert_eq!(contents, "apply me\n");
}

#[test]
fn test_stash_drop_removes_entry() {
    let dir = init_repo();
    std::fs::write(dir.path().join("README.md"), "drop me\n").unwrap();
    git(dir.path(), &["stash", "push", "-m", "drop test"]);
    assert_eq!(git(dir.path(), &["stash", "list"]).lines().count(), 1);
    git(dir.path(), &["stash", "drop", "stash@{0}"]);
    assert_eq!(git(dir.path(), &["stash", "list"]).lines().count(), 0);
}

#[test]
fn test_stash_show_has_diff() {
    let dir = init_repo();
    std::fs::write(dir.path().join("README.md"), "show me\n").unwrap();
    git(dir.path(), &["stash", "push", "-m", "show test"]);
    let show = git(dir.path(), &["stash", "show", "-p", "stash@{0}"]);
    assert!(show.contains("show me"));
}

#[test]
fn test_stash_clear_removes_all() {
    let dir = init_repo();
    // Create two stashes
    std::fs::write(dir.path().join("README.md"), "first\n").unwrap();
    git(dir.path(), &["stash", "push", "-m", "stash 1"]);
    std::fs::write(dir.path().join("README.md"), "second\n").unwrap();
    git(dir.path(), &["stash", "push", "-m", "stash 2"]);
    assert_eq!(git(dir.path(), &["stash", "list"]).lines().count(), 2);
    git(dir.path(), &["stash", "clear"]);
    assert_eq!(git(dir.path(), &["stash", "list"]).lines().count(), 0);
}

// ────────────────────────────────────────────────────────────────────────
// Commit workflow tests
// ────────────────────────────────────────────────────────────────────────

#[test]
fn test_stage_and_commit_workflow() {
    let dir = init_repo();
    std::fs::write(dir.path().join("app.txt"), "new feature\n").unwrap();
    git(dir.path(), &["add", "app.txt"]);
    git(dir.path(), &["commit", "-m", "add app"]);
    let log = git(dir.path(), &["log", "--oneline"]);
    assert!(log.contains("add app"));
    assert_eq!(log.lines().count(), 2);
}

#[test]
fn test_amend_commit() {
    let dir = init_repo();
    std::fs::write(dir.path().join("fix.txt"), "fix\n").unwrap();
    git(dir.path(), &["add", "fix.txt"]);
    git(dir.path(), &["commit", "-m", "wrong msg"]);
    git(dir.path(), &["commit", "--amend", "-m", "correct msg"]);
    let log = git(dir.path(), &["log", "--oneline"]);
    assert!(log.contains("correct msg"));
    assert!(!log.contains("wrong msg"));
}

// ────────────────────────────────────────────────────────────────────────
// Branch rename & merge tests
// ────────────────────────────────────────────────────────────────────────

#[test]
fn test_rename_branch() {
    let dir = init_repo();
    git(dir.path(), &["checkout", "-b", "old-name"]);
    git(dir.path(), &["branch", "-m", "new-name"]);
    let branches = git(dir.path(), &["branch"]);
    assert!(branches.contains("new-name"));
    assert!(!branches.contains("old-name"));
}

#[test]
fn test_merge_branch() {
    let dir = init_repo();
    git(dir.path(), &["checkout", "-b", "feature"]);
    std::fs::write(dir.path().join("feature.txt"), "feature work\n").unwrap();
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-m", "feature commit"]);
    git(dir.path(), &["checkout", "main"]);
    git(
        dir.path(),
        &["merge", "feature", "--no-ff", "-m", "merge feature"],
    );
    let log = git(dir.path(), &["log", "--oneline"]);
    assert!(log.contains("merge feature"));
    assert!(log.contains("feature commit"));
}

// ────────────────────────────────────────────────────────────────────────
// Reset tests
// ────────────────────────────────────────────────────────────────────────

#[test]
fn test_soft_reset() {
    let dir = init_repo();
    std::fs::write(dir.path().join("reset.txt"), "data\n").unwrap();
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-m", "to be reset"]);
    let hash = git(dir.path(), &["rev-parse", "HEAD~1"]).trim().to_string();
    git(dir.path(), &["reset", "--soft", &hash]);
    // File should still be staged
    let status = git(dir.path(), &["status", "--porcelain"]);
    assert!(status.contains("reset.txt"));
}

#[test]
fn test_mixed_reset() {
    let dir = init_repo();
    std::fs::write(dir.path().join("mixed.txt"), "data\n").unwrap();
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-m", "to be mixed-reset"]);
    let hash = git(dir.path(), &["rev-parse", "HEAD~1"]).trim().to_string();
    git(dir.path(), &["reset", "--mixed", &hash]);
    // File should be unstaged (untracked)
    let status = git(dir.path(), &["status", "--porcelain"]);
    assert!(status.contains("mixed.txt"));
}

// ────────────────────────────────────────────────────────────────────────
// Reflog recovery test
// ────────────────────────────────────────────────────────────────────────

#[test]
fn test_reflog_preserves_reset_history() {
    let dir = init_repo();
    std::fs::write(dir.path().join("file.txt"), "v1\n").unwrap();
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-m", "v1"]);
    let v1_hash = git(dir.path(), &["rev-parse", "HEAD"]).trim().to_string();
    std::fs::write(dir.path().join("file.txt"), "v2\n").unwrap();
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-m", "v2"]);
    // Reset back to v1
    git(dir.path(), &["reset", "--hard", &v1_hash]);
    // Reflog should still contain v2
    let reflog = git(dir.path(), &["reflog", "--oneline"]);
    assert!(reflog.contains("v2"));
}

// ────────────────────────────────────────────────────────────────────────
// Edge case: empty stash on clean repo
// ────────────────────────────────────────────────────────────────────────

#[test]
fn test_stash_on_clean_repo_fails() {
    let dir = init_repo();
    let output = Command::new("git")
        .args(["stash", "push", "-m", "nothing"])
        .current_dir(dir.path())
        .output()
        .expect("failed to run git");
    // Git stash should indicate no changes to stash
    let stderr = String::from_utf8(output.stderr).unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let combined = format!("{}{}", stdout, stderr);
    assert!(
        combined.contains("No local changes") || combined.contains("no changes"),
        "Expected stash to indicate no changes: {}",
        combined
    );
}
