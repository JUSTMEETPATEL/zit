use anyhow::Result;
use super::runner::run_git;

pub struct RemoteOps;

impl RemoteOps {
    /// List all remotes with their URLs.
    pub fn list() -> Result<Vec<(String, String)>> {
        let output = run_git(&["remote", "-v"])?;
        let mut remotes = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for line in output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0].to_string();
                if seen.insert(name.clone()) {
                    remotes.push((name, parts[1].to_string()));
                }
            }
        }

        Ok(remotes)
    }

    /// Add a new remote.
    pub fn add(name: &str, url: &str) -> Result<()> {
        run_git(&["remote", "add", name, url])?;
        Ok(())
    }

    /// Push to a remote.
    pub fn push(remote: &str, branch: &str, set_upstream: bool) -> Result<String> {
        let mut args = vec!["push"];
        if set_upstream {
            args.push("-u");
        }
        args.push(remote);
        args.push(branch);
        run_git(&args)
    }

    /// Fetch from a remote.
    pub fn fetch(remote: &str) -> Result<String> {
        run_git(&["fetch", remote])
    }
}
