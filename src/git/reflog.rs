use anyhow::Result;
use super::runner::run_git;

#[derive(Debug, Clone)]
pub struct ReflogEntry {
    pub index: usize,
    pub hash: String,
    pub short_hash: String,
    pub operation: String,   // commit, reset, checkout, merge, rebase, etc.
    pub message: String,
    pub date: String,
}

const REFLOG_FORMAT: &str = "%H\x1f%h\x1f%gs\x1f%ar";

/// Fetch reflog entries.
pub fn get_reflog(count: usize) -> Result<Vec<ReflogEntry>> {
    let count_str = format!("-{}", count);
    let format_str = format!("--format={}", REFLOG_FORMAT);

    let output = run_git(&["reflog", &count_str, &format_str])?;
    let mut entries = Vec::new();

    for (i, line) in output.lines().enumerate() {
        let parts: Vec<&str> = line.split('\x1f').collect();
        if parts.len() < 4 {
            continue;
        }

        // Extract operation type from the reflog subject (e.g., "commit: message" -> "commit")
        let gs = parts[2];
        let (operation, message) = if let Some(idx) = gs.find(": ") {
            (gs[..idx].to_string(), gs[idx + 2..].to_string())
        } else {
            (gs.to_string(), String::new())
        };

        entries.push(ReflogEntry {
            index: i,
            hash: parts[0].to_string(),
            short_hash: parts[1].to_string(),
            operation,
            message,
            date: parts[3].to_string(),
        });
    }

    Ok(entries)
}

/// Filter reflog entries by operation type.
pub fn filter_reflog(entries: &[ReflogEntry], operation: &str) -> Vec<ReflogEntry> {
    entries
        .iter()
        .filter(|e| e.operation.to_lowercase().contains(&operation.to_lowercase()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_reflog() {
        let sample = "abc123def456abc123def456abc123def456abc123\x1fabc123d\x1fcommit: initial commit\x1f2 hours ago\n\
                       def456abc123def456abc123def456abc123def456\x1fdef456a\x1freset: moving to HEAD~1\x1f3 hours ago\n";

        let mut entries = Vec::new();
        for (i, line) in sample.lines().enumerate() {
            let parts: Vec<&str> = line.split('\x1f').collect();
            if parts.len() < 4 {
                continue;
            }
            let gs = parts[2];
            let (operation, message) = if let Some(idx) = gs.find(": ") {
                (gs[..idx].to_string(), gs[idx + 2..].to_string())
            } else {
                (gs.to_string(), String::new())
            };

            entries.push(ReflogEntry {
                index: i,
                hash: parts[0].to_string(),
                short_hash: parts[1].to_string(),
                operation,
                message,
                date: parts[3].to_string(),
            });
        }

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].operation, "commit");
        assert_eq!(entries[0].message, "initial commit");
        assert_eq!(entries[1].operation, "reset");
        assert_eq!(entries[1].message, "moving to HEAD~1");
    }

    #[test]
    fn test_filter_reflog() {
        let entries = vec![
            ReflogEntry {
                index: 0,
                hash: "abc".to_string(),
                short_hash: "abc".to_string(),
                operation: "commit".to_string(),
                message: "test".to_string(),
                date: "now".to_string(),
            },
            ReflogEntry {
                index: 1,
                hash: "def".to_string(),
                short_hash: "def".to_string(),
                operation: "reset".to_string(),
                message: "test".to_string(),
                date: "now".to_string(),
            },
        ];

        let filtered = filter_reflog(&entries, "commit");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].operation, "commit");
    }
}
