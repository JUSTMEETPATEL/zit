use anyhow::Result;
use super::runner::run_git;

#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Ignored,
    Conflicted,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub status: FileStatus,
    pub path: String,
    pub original_path: Option<String>, // For renames
}

#[derive(Debug, Clone)]
pub struct RepoStatus {
    pub branch: String,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub staged: Vec<FileEntry>,
    pub unstaged: Vec<FileEntry>,
    pub untracked: Vec<FileEntry>,
    pub conflicts: Vec<FileEntry>,
    pub stash_count: u32,
}

impl RepoStatus {
    pub fn is_clean(&self) -> bool {
        self.staged.is_empty()
            && self.unstaged.is_empty()
            && self.untracked.is_empty()
            && self.conflicts.is_empty()
    }
}

/// Fetch the full repository status by parsing `git status --porcelain=v2 --branch`.
pub fn get_status() -> Result<RepoStatus> {
    let output = run_git(&["status", "--porcelain=v2", "--branch"])?;

    let mut branch = String::from("(unknown)");
    let mut upstream = None;
    let mut ahead: u32 = 0;
    let mut behind: u32 = 0;
    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();
    let mut conflicts = Vec::new();

    for line in output.lines() {
        if line.starts_with("# branch.head ") {
            branch = line.strip_prefix("# branch.head ").unwrap_or("(unknown)").to_string();
        } else if line.starts_with("# branch.upstream ") {
            upstream = Some(
                line.strip_prefix("# branch.upstream ")
                    .unwrap_or("")
                    .to_string(),
            );
        } else if line.starts_with("# branch.ab ") {
            // Format: # branch.ab +N -M
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                ahead = parts[2]
                    .trim_start_matches('+')
                    .parse()
                    .unwrap_or(0);
                behind = parts[3]
                    .trim_start_matches('-')
                    .parse()
                    .unwrap_or(0);
            }
        } else if line.starts_with("1 ") {
            // Ordinary changed entry: 1 XY sub mH mI mW hH hI path
            parse_ordinary_entry(line, &mut staged, &mut unstaged, &mut conflicts);
        } else if line.starts_with("2 ") {
            // Rename/copy entry: 2 XY sub mH mI mW hH hI Xscore path\torigPath
            parse_rename_entry(line, &mut staged, &mut unstaged);
        } else if line.starts_with("u ") {
            // Unmerged entry
            let parts: Vec<&str> = line.splitn(11, ' ').collect();
            if let Some(path) = parts.last() {
                conflicts.push(FileEntry {
                    status: FileStatus::Conflicted,
                    path: path.to_string(),
                    original_path: None,
                });
            }
        } else if line.starts_with("? ") {
            // Untracked
            let path = line.strip_prefix("? ").unwrap_or("").to_string();
            untracked.push(FileEntry {
                status: FileStatus::Untracked,
                path,
                original_path: None,
            });
        }
    }

    // Get stash count
    let stash_count = run_git(&["stash", "list"])
        .map(|s| s.lines().count() as u32)
        .unwrap_or(0);

    Ok(RepoStatus {
        branch,
        upstream,
        ahead,
        behind,
        staged,
        unstaged,
        untracked,
        conflicts,
        stash_count,
    })
}

fn parse_ordinary_entry(
    line: &str,
    staged: &mut Vec<FileEntry>,
    unstaged: &mut Vec<FileEntry>,
    conflicts: &mut Vec<FileEntry>,
) {
    // Format: 1 XY sub mH mI mW hH hI path
    let parts: Vec<&str> = line.splitn(9, ' ').collect();
    if parts.len() < 9 {
        return;
    }
    let xy = parts[1];
    let path = parts[8].to_string();
    let x = xy.chars().next().unwrap_or('.');
    let y = xy.chars().nth(1).unwrap_or('.');

    // X = index status (staged)
    if let Some(status) = char_to_status(x) {
        if status == FileStatus::Conflicted {
            conflicts.push(FileEntry {
                status,
                path: path.clone(),
                original_path: None,
            });
        } else {
            staged.push(FileEntry {
                status,
                path: path.clone(),
                original_path: None,
            });
        }
    }

    // Y = worktree status (unstaged)
    if let Some(status) = char_to_status(y) {
        unstaged.push(FileEntry {
            status,
            path,
            original_path: None,
        });
    }
}

fn parse_rename_entry(
    line: &str,
    staged: &mut Vec<FileEntry>,
    unstaged: &mut Vec<FileEntry>,
) {
    // Format: 2 XY sub mH mI mW hH hI Xscore path\torigPath
    let parts: Vec<&str> = line.splitn(10, ' ').collect();
    if parts.len() < 10 {
        return;
    }
    let xy = parts[1];
    let path_part = parts[9];
    let paths: Vec<&str> = path_part.split('\t').collect();
    let path = paths.first().unwrap_or(&"").to_string();
    let orig = paths.get(1).map(|s| s.to_string());

    let x = xy.chars().next().unwrap_or('.');

    if x == 'R' || x == 'C' {
        let status = if x == 'R' {
            FileStatus::Renamed
        } else {
            FileStatus::Copied
        };
        staged.push(FileEntry {
            status,
            path,
            original_path: orig,
        });
    }

    let y = xy.chars().nth(1).unwrap_or('.');
    if let Some(status) = char_to_status(y) {
        unstaged.push(FileEntry {
            status,
            path: paths.first().unwrap_or(&"").to_string(),
            original_path: None,
        });
    }
}

fn char_to_status(c: char) -> Option<FileStatus> {
    match c {
        'M' => Some(FileStatus::Modified),
        'A' => Some(FileStatus::Added),
        'D' => Some(FileStatus::Deleted),
        'R' => Some(FileStatus::Renamed),
        'C' => Some(FileStatus::Copied),
        'U' => Some(FileStatus::Conflicted),
        '.' | ' ' => None,
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_porcelain_v2() {
        let sample = "\
# branch.head main
# branch.upstream origin/main
# branch.ab +2 -1
1 M. N... 100644 100644 100644 abc123 def456 src/main.rs
1 .M N... 100644 100644 100644 abc123 def456 src/lib.rs
? untracked_file.txt
";
        let mut branch = String::from("(unknown)");
        let mut upstream = None;
        let mut ahead: u32 = 0;
        let mut behind: u32 = 0;
        let mut staged = Vec::new();
        let mut unstaged = Vec::new();
        let mut untracked = Vec::new();
        let mut conflicts = Vec::new();

        for line in sample.lines() {
            if line.starts_with("# branch.head ") {
                branch = line.strip_prefix("# branch.head ").unwrap().to_string();
            } else if line.starts_with("# branch.upstream ") {
                upstream = Some(line.strip_prefix("# branch.upstream ").unwrap().to_string());
            } else if line.starts_with("# branch.ab ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                ahead = parts[2].trim_start_matches('+').parse().unwrap_or(0);
                behind = parts[3].trim_start_matches('-').parse().unwrap_or(0);
            } else if line.starts_with("1 ") {
                parse_ordinary_entry(line, &mut staged, &mut unstaged, &mut conflicts);
            } else if line.starts_with("? ") {
                let path = line.strip_prefix("? ").unwrap().to_string();
                untracked.push(FileEntry {
                    status: FileStatus::Untracked,
                    path,
                    original_path: None,
                });
            }
        }

        assert_eq!(branch, "main");
        assert_eq!(upstream, Some("origin/main".to_string()));
        assert_eq!(ahead, 2);
        assert_eq!(behind, 1);
        assert_eq!(staged.len(), 1);
        assert_eq!(staged[0].path, "src/main.rs");
        assert_eq!(unstaged.len(), 1);
        assert_eq!(unstaged[0].path, "src/lib.rs");
        assert_eq!(untracked.len(), 1);
        assert_eq!(untracked[0].path, "untracked_file.txt");
    }
}
