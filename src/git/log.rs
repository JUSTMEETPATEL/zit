use anyhow::Result;
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

    let mut args = vec!["log", &count_str, &skip_str, &format_str, "--graph"];

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

    for line in output.lines() {
        // Lines may start with graph characters (*, |, /, \, space)
        // Find where the format data starts by looking for a hash-length hex string
        let (graph, data) = split_graph_and_data(line);

        if data.is_empty() {
            continue;
        }

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
    }

    entries
}

fn split_graph_and_data(line: &str) -> (&str, &str) {
    // Graph chars are: * | / \ space
    // The data starts at the first hex char that's part of a full hash
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '*' || c == '|' || c == '/' || c == '\\' || c == ' ' || c == '_' {
            i += 1;
        } else {
            break;
        }
    }
    (&line[..i], &line[i..])
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

    let output = run_git(&["log", &count_str, &format_str, &grep_str, "-i"])?;
    Ok(parse_log_output(&output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_output() {
        let sample = "* abc123def456abc123def456abc123def456abc123\x1fabc123d\x1ffeat: add login\x1fJohn\x1f2 hours ago\x1f2026-02-10T10:00:00+05:30\x1f\x1fHEAD -> main\n";
        let entries = parse_log_output(sample);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].short_hash, "abc123d");
        assert_eq!(entries[0].message, "feat: add login");
        assert_eq!(entries[0].author, "John");
        assert_eq!(entries[0].refs, "HEAD -> main");
        assert_eq!(entries[0].graph, "* ");
    }

    #[test]
    fn test_split_graph_and_data() {
        let (g, d) = split_graph_and_data("* | abc123");
        assert_eq!(g, "* | ");
        assert_eq!(d, "abc123");
    }
}
