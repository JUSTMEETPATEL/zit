use anyhow::Result;
use regex::Regex;
use super::runner::run_git;

#[derive(Debug, Clone)]
pub struct CommitEntry {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub date: String,       // relative date like "2 hours ago"
    pub date_iso: String,   // ISO format for sorting
    pub parents: Vec<String>,
    pub refs: String,       // decorated refs (HEAD -> main, origin/main, tag: v1.0)
    pub graph: String,      // graph characters for this line
}

const LOG_FORMAT: &str = "%H\x1f%h\x1f%s\x1f%an\x1f%ar\x1f%aI\x1f%P\x1f%D";
const SEPARATOR: char = '\x1f';

/// Fetch commit log entries with optional pagination.
pub fn get_log(count: usize, skip: usize, branch: Option<&str>) -> Result<Vec<CommitEntry>> {
    let count_str = format!("-{}", count);
    let skip_str = format!("--skip={}", skip);
    let format_str = format!("--format={}", LOG_FORMAT);

    // Force color=never to allow reliable regex parsing of graph structure vs content
    let mut args = vec!["log", &count_str, &skip_str, &format_str, "--graph", "--color=never"];

    if let Some(b) = branch {
        args.push(b);
    }

    let output = run_git(&args)?;
    let entries = parse_log_output(&output);
    Ok(entries)
}

/// Get the last N commits (shorthand for dashboard use).
pub fn get_recent_commits(count: usize) -> Result<Vec<CommitEntry>> {
    get_log(count, 0, None)
}

fn parse_log_output(output: &str) -> Vec<CommitEntry> {
    let mut entries = Vec::new();
    // Regex to find the start of the commit data: 40 hex chars followed by our separator
    let re = Regex::new(r"([0-9a-f]{40})\x1f").unwrap();

    for line in output.lines() {
        // Find where the Commit Hash starts
        if let Some(mat) = re.find(line) {
            let start_idx = mat.start();
            let graph = &line[..start_idx];
            let data = &line[start_idx..];

            let parts: Vec<&str> = data.split(SEPARATOR).collect();
            if parts.len() < 8 {
                continue;
            }

            let parents: Vec<String> = parts[6]
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();

            entries.push(CommitEntry {
                hash: parts[0].to_string(),
                short_hash: parts[1].to_string(),
                message: parts[2].to_string(),
                author: parts[3].to_string(),
                date: parts[4].to_string(),
                date_iso: parts[5].to_string(),
                parents,
                refs: parts[7].to_string(),
                graph: graph.to_string(),
            });
        } else {
            // No commit hash found -> This is a graph-only line (e.g. "| \")
            // We still want to render it to maintain the graph continuity
            if !line.trim().is_empty() {
                entries.push(CommitEntry {
                    hash: String::new(),
                    short_hash: String::new(),
                    message: String::new(),
                    author: String::new(),
                    date: String::new(),
                    date_iso: String::new(),
                    parents: Vec::new(),
                    refs: String::new(),
                    graph: line.to_string(),
                });
            }
        }
    }

    entries
}

/// Get the total number of commits in the current branch.
pub fn commit_count() -> Result<usize> {
    let output = run_git(&["rev-list", "--count", "HEAD"])?;
    Ok(output.trim().parse().unwrap_or(0))
}

/// Search commits by message text.
pub fn search_commits(query: &str, count: usize) -> Result<Vec<CommitEntry>> {
    let count_str = format!("-{}", count);
    let format_str = format!("--format={}", LOG_FORMAT);
    let grep_str = format!("--grep={}", query);

    // Note: search doesn't use --graph usually, but if we want consistent return type,
    // we can parse it. Without --graph, the regex matches at index 0, so graph is empty string.
    let output = run_git(&["log", &count_str, &format_str, &grep_str, "-i"])?;
    Ok(parse_log_output(&output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_output() {
        // Hash must be exactly 40 chars for regex to match correctly at start
        let sample = "* abc123def456abc123def456abc123def456abc1\x1fabc123d\x1ffeat: add login\x1fJohn\x1f2 hours ago\x1f2026-02-10T10:00:00+05:30\x1f\x1fHEAD -> main\n";
        let entries = parse_log_output(sample);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].short_hash, "abc123d");
        assert_eq!(entries[0].message, "feat: add login");
        assert_eq!(entries[0].author, "John");
        assert_eq!(entries[0].refs, "HEAD -> main");
        assert_eq!(entries[0].graph, "* ");
    }

    #[test]
    fn test_parse_graph_only_line() {
        let sample = "| \\ \n";
        let entries = parse_log_output(sample);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].hash, "");
        assert_eq!(entries[0].graph, "| \\ ");
    }
}
