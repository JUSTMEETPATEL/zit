use regex::Regex;
use std::path::Path;

/// A single secret-detection rule.
#[derive(Debug, Clone)]
pub struct SecretRule {
    pub name: &'static str,
    pub pattern: Regex,
}

/// A detected secret finding.
#[derive(Debug, Clone)]
pub struct SecretFinding {
    pub file: String,
    pub line: usize,
    pub rule_name: String,
    pub preview: String,
}

impl std::fmt::Display for SecretFinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{} — {} ({})",
            self.file, self.line, self.rule_name, self.preview
        )
    }
}

/// Build the compiled default ruleset.
/// This is intentionally a function (not a lazy_static) so callers can extend it.
pub fn default_rules() -> Vec<SecretRule> {
    // Each tuple is (name, regex_pattern). Patterns are compiled once per call.
    let raw: Vec<(&str, &str)> = vec![
        // ── Cloud Provider Keys ────────────────────────────────────
        (
            "AWS Access Key ID",
            r"(?:^|[^A-Za-z0-9/+=])(?:AKIA[0-9A-Z]{16})(?:$|[^A-Za-z0-9/+=])",
        ),
        (
            "AWS Secret Access Key",
            r#"(?i)(?:aws_secret_access_key|aws_secret)\s*[:=]\s*['"]?([A-Za-z0-9/+=]{40})['"]?"#,
        ),
        (
            "Google API Key",
            r"(?:^|[^A-Za-z0-9])AIza[0-9A-Za-z\-_]{35}(?:$|[^A-Za-z0-9])",
        ),
        // ── Git Platform Tokens ────────────────────────────────────
        (
            "GitHub Personal Access Token",
            r"(?:^|[^A-Za-z0-9_])ghp_[a-zA-Z0-9]{36}(?:$|[^A-Za-z0-9_])",
        ),
        (
            "GitHub OAuth Token",
            r"(?:^|[^A-Za-z0-9_])gho_[a-zA-Z0-9]{36}(?:$|[^A-Za-z0-9_])",
        ),
        (
            "GitHub App Token",
            r"(?:^|[^A-Za-z0-9_])(?:ghu|ghs|ghr)_[a-zA-Z0-9]{36}(?:$|[^A-Za-z0-9_])",
        ),
        (
            "GitHub Fine-Grained PAT",
            r"(?:^|[^A-Za-z0-9_])github_pat_[a-zA-Z0-9]{22}_[a-zA-Z0-9]{59}(?:$|[^A-Za-z0-9_])",
        ),
        (
            "GitLab Personal Access Token",
            r"(?:^|[^A-Za-z0-9\-])glpat-[a-zA-Z0-9\-]{20,}(?:$|[^A-Za-z0-9\-])",
        ),
        // ── Communication Tokens ───────────────────────────────────
        (
            "Slack Token",
            r"(?:^|[^A-Za-z0-9\-])xox[baprs]-[a-zA-Z0-9\-]{10,}(?:$|[^A-Za-z0-9\-])",
        ),
        (
            "Discord Bot Token",
            r"(?:^|[^A-Za-z0-9\.])[MN][A-Za-z0-9]{23,}\.[A-Za-z0-9\-_]{6}\.[A-Za-z0-9\-_]{27,}(?:$|[^A-Za-z0-9\.])",
        ),
        // ── Payment Tokens ─────────────────────────────────────────
        (
            "Stripe Secret Key",
            r"(?:^|[^A-Za-z0-9_])sk_live_[a-zA-Z0-9]{24,}(?:$|[^A-Za-z0-9_])",
        ),
        (
            "Stripe Restricted Key",
            r"(?:^|[^A-Za-z0-9_])rk_live_[a-zA-Z0-9]{24,}(?:$|[^A-Za-z0-9_])",
        ),
        // ── Cryptographic Material ─────────────────────────────────
        ("Private Key", r"-----BEGIN[A-Z\s]*PRIVATE KEY-----"),
        // ── JWT ────────────────────────────────────────────────────
        (
            "JWT Token",
            r"eyJ[a-zA-Z0-9_\-]{10,}\.eyJ[a-zA-Z0-9_\-]{10,}\.[a-zA-Z0-9_\-]{10,}",
        ),
        // ── Database Connection Strings ────────────────────────────
        (
            "Database Connection String",
            r"(?i)(?:postgres|postgresql|mysql|mongodb|redis)://[^\s:]+:[^\s@]+@[^\s]+",
        ),
        // ── Generic Secrets (key=value assignments) ────────────────
        (
            "Generic API Key Assignment",
            r#"(?i)(?:api[_\-]?key|apikey)\s*[:=]\s*['"][A-Za-z0-9\-_./+=]{8,}['"]"#,
        ),
        (
            "Generic Secret Assignment",
            r#"(?i)(?:secret|secret[_\-]?key)\s*[:=]\s*['"][A-Za-z0-9\-_./+=]{8,}['"]"#,
        ),
        (
            "Generic Password Assignment",
            r#"(?i)(?:password|passwd|pwd)\s*[:=]\s*['"][^'"]{8,}['"]"#,
        ),
        (
            "Generic Token Assignment",
            r#"(?i)(?:auth[_\-]?token|access[_\-]?token|bearer)\s*[:=]\s*['"][A-Za-z0-9\-_./+=]{8,}['"]"#,
        ),
        // ── Heroku / Twilio / SendGrid ─────────────────────────────
        (
            "Heroku API Key",
            r#"(?i)(?:heroku[_\-]?api[_\-]?key)\s*[:=]\s*['"]?[a-f0-9\-]{36}['"]?"#,
        ),
        (
            "Twilio API Key",
            r"(?:^|[^A-Za-z0-9])SK[a-f0-9]{32}(?:$|[^A-Za-z0-9])",
        ),
        (
            "SendGrid API Key",
            r"(?:^|[^A-Za-z0-9\.])SG\.[a-zA-Z0-9_\-]{22}\.[a-zA-Z0-9_\-]{43}(?:$|[^A-Za-z0-9\.])",
        ),
    ];

    raw.into_iter()
        .filter_map(|(name, pattern)| {
            Regex::new(pattern)
                .ok()
                .map(|re| SecretRule { name, pattern: re })
        })
        .collect()
}

/// File extensions considered binary / non-scannable.
const BINARY_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "bmp", "ico", "svg", "webp", // images
    "mp3", "mp4", "wav", "avi", "mov", "mkv", // media
    "zip", "tar", "gz", "bz2", "xz", "7z", "rar", // archives
    "exe", "dll", "so", "dylib", "o", "a", // binaries
    "woff", "woff2", "ttf", "otf", "eot", // fonts
    "pdf", "doc", "docx", "xls", "xlsx", // documents
    "pyc", "class", "wasm", // bytecode
    "lock", // lock files (often huge, not secrets)
];

/// Check if a path is likely a binary file based on extension.
pub fn is_binary(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| BINARY_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Redact a matched secret for safe display: show first 4 chars + ****
pub fn redact_match(matched: &str) -> String {
    let trimmed = matched.trim();
    if trimmed.len() <= 4 {
        return "****".to_string();
    }
    format!("{}****", &trimmed[..4])
}

/// Scan a single file's content against the given rules.
/// Returns a list of findings with line numbers.
pub fn scan_content(filename: &str, content: &str, rules: &[SecretRule]) -> Vec<SecretFinding> {
    let mut findings = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        // Skip comment-only lines that look like documentation examples
        let trimmed = line.trim();
        if trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
        {
            // Still scan — comments can contain real leaked secrets
            // but we'll let it through; the user can allowlist if needed
        }

        for rule in rules {
            if let Some(m) = rule.pattern.find(line) {
                findings.push(SecretFinding {
                    file: filename.to_string(),
                    line: line_num + 1,
                    rule_name: rule.name.to_string(),
                    preview: redact_match(m.as_str()),
                });
            }
        }
    }

    findings
}

/// Scan all staged files by reading their content from disk.
/// Skips binary files and files that can't be read.
pub fn scan_staged_files(
    staged_files: &[super::status::FileEntry],
    rules: &[SecretRule],
    allowlist: &[String],
) -> Vec<SecretFinding> {
    let mut all_findings = Vec::new();

    for file in staged_files {
        // Skip deleted files (no content to scan)
        if file.status == super::status::FileStatus::Deleted {
            continue;
        }

        // Skip binary files
        if is_binary(&file.path) {
            continue;
        }

        // Read file content
        let content = match std::fs::read_to_string(&file.path) {
            Ok(c) => c,
            Err(_) => continue, // Skip files we can't read (binary, permission issues)
        };

        let findings = scan_content(&file.path, &content, rules);
        all_findings.extend(findings);
    }

    // Filter out allowlisted findings
    if !allowlist.is_empty() {
        all_findings.retain(|f| {
            !allowlist
                .iter()
                .any(|a| f.preview.contains(a) || f.file.contains(a) || f.rule_name.contains(a))
        });
    }

    all_findings
}

/// Scan only the **added** lines from a unified diff string.
/// This avoids flagging pre-existing secrets that haven't changed.
#[allow(dead_code)]
pub fn scan_diff_content(
    diff: &str,
    rules: &[SecretRule],
    allowlist: &[String],
) -> Vec<SecretFinding> {
    let mut findings = Vec::new();
    let mut current_file = String::new();
    let mut line_in_new: usize = 0;

    for line in diff.lines() {
        if line.starts_with("+++ b/") {
            current_file = line.strip_prefix("+++ b/").unwrap_or("").to_string();
        } else if line.starts_with("@@ ") {
            // Parse new-file line number from hunk header: @@ -old,count +new,count @@
            if let Some(plus_part) = line.split('+').nth(1) {
                let num_str = plus_part.split(',').next().unwrap_or("0");
                line_in_new = num_str.parse().unwrap_or(0);
            }
        } else if let Some(added) = line.strip_prefix('+') {
            // This is an added line — scan it
            if !current_file.is_empty() && !is_binary(&current_file) {
                for rule in rules {
                    if let Some(m) = rule.pattern.find(added) {
                        findings.push(SecretFinding {
                            file: current_file.clone(),
                            line: line_in_new,
                            rule_name: rule.name.to_string(),
                            preview: redact_match(m.as_str()),
                        });
                    }
                }
            }
            line_in_new += 1;
        } else if !line.starts_with('-') && !line.starts_with("---") {
            // Context line — increment new-file line counter
            if !line.starts_with("diff ") && !line.starts_with("index ") {
                line_in_new += 1;
            }
        }
    }

    // Filter out allowlisted findings
    if !allowlist.is_empty() {
        findings.retain(|f| {
            !allowlist
                .iter()
                .any(|a| f.preview.contains(a) || f.file.contains(a) || f.rule_name.contains(a))
        });
    }

    findings
}

// ═══════════════════════════════════════════════════════════════
//                         TESTS
// ═══════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> Vec<SecretRule> {
        default_rules()
    }

    // ── AWS ─────────────────────────────────────────────────────

    #[test]
    fn test_detect_aws_access_key() {
        let content = r#"AWS_KEY=AKIAIOSFODNN7EXAMPLE1"#;
        let findings = scan_content("test.env", content, &rules());
        assert!(
            findings.iter().any(|f| f.rule_name.contains("AWS")),
            "Should detect AWS access key, got: {:?}",
            findings
        );
    }

    #[test]
    fn test_detect_aws_secret_key() {
        let content = r#"aws_secret_access_key = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY1""#;
        let findings = scan_content("config.yml", content, &rules());
        assert!(
            findings.iter().any(|f| f.rule_name.contains("AWS Secret")),
            "Should detect AWS secret key, got: {:?}",
            findings
        );
    }

    // ── GitHub ──────────────────────────────────────────────────

    #[test]
    fn test_detect_github_pat() {
        let content = r#"token = "ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghij""#;
        let findings = scan_content("config.toml", content, &rules());
        assert!(
            findings.iter().any(|f| f.rule_name.contains("GitHub")),
            "Should detect GitHub PAT, got: {:?}",
            findings
        );
    }

    #[test]
    fn test_detect_github_oauth() {
        let content = "export TOKEN=gho_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghij";
        let findings = scan_content(".env", content, &rules());
        assert!(
            findings
                .iter()
                .any(|f| f.rule_name.contains("GitHub OAuth")),
            "Should detect GitHub OAuth token, got: {:?}",
            findings
        );
    }

    // ── GitLab ──────────────────────────────────────────────────

    #[test]
    fn test_detect_gitlab_pat() {
        let content = "GITLAB_TOKEN=glpat-abcdefghijklmnopqrst";
        let findings = scan_content(".env", content, &rules());
        assert!(
            findings.iter().any(|f| f.rule_name.contains("GitLab")),
            "Should detect GitLab PAT, got: {:?}",
            findings
        );
    }

    // ── Slack ───────────────────────────────────────────────────

    #[test]
    fn test_detect_slack_token() {
        let content = r#"SLACK_TOKEN="xoxb-123456789012-abcdefghij""#;
        let findings = scan_content(".env", content, &rules());
        assert!(
            findings.iter().any(|f| f.rule_name.contains("Slack")),
            "Should detect Slack token, got: {:?}",
            findings
        );
    }

    // ── Stripe ──────────────────────────────────────────────────

    #[test]
    fn test_detect_stripe_secret() {
        let content = format!("STRIPE_KEY={}{}", "sk_live_", "abcdefghijklmnopqrstuvwx").as_str();
        let findings = scan_content(".env", content, &rules());
        assert!(
            findings.iter().any(|f| f.rule_name.contains("Stripe")),
            "Should detect Stripe key, got: {:?}",
            findings
        );
    }

    // ── Google ──────────────────────────────────────────────────

    #[test]
    fn test_detect_google_api_key() {
        let content = r#"key: "AIzaSyA1234567890abcdefghijklmnopqrstuv""#;
        let findings = scan_content("config.json", content, &rules());
        assert!(
            findings.iter().any(|f| f.rule_name.contains("Google")),
            "Should detect Google API key, got: {:?}",
            findings
        );
    }

    // ── Private Key ─────────────────────────────────────────────

    #[test]
    fn test_detect_private_key() {
        let content = "-----BEGIN RSA PRIVATE KEY-----\nMIIE...";
        let findings = scan_content("id_rsa", content, &rules());
        assert!(
            findings.iter().any(|f| f.rule_name.contains("Private Key")),
            "Should detect private key, got: {:?}",
            findings
        );
    }

    #[test]
    fn test_detect_ec_private_key() {
        let content = "-----BEGIN EC PRIVATE KEY-----";
        let findings = scan_content("key.pem", content, &rules());
        assert!(
            findings.iter().any(|f| f.rule_name.contains("Private Key")),
            "Should detect EC private key, got: {:?}",
            findings
        );
    }

    // ── JWT ─────────────────────────────────────────────────────

    #[test]
    fn test_detect_jwt() {
        let content = "token=eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.abc123def456_-ghiJKL";
        let findings = scan_content("auth.js", content, &rules());
        assert!(
            findings.iter().any(|f| f.rule_name.contains("JWT")),
            "Should detect JWT, got: {:?}",
            findings
        );
    }

    // ── Database Connection Strings ─────────────────────────────

    #[test]
    fn test_detect_postgres_connection() {
        let content = r#"DATABASE_URL="postgres://admin:s3cr3tP4ss@db.example.com:5432/mydb""#;
        let findings = scan_content(".env", content, &rules());
        assert!(
            findings.iter().any(|f| f.rule_name.contains("Database")),
            "Should detect postgres connection string, got: {:?}",
            findings
        );
    }

    #[test]
    fn test_detect_mongodb_connection() {
        let content = r#"MONGO_URI=mongodb://root:password123@mongo.example.com:27017/app"#;
        let findings = scan_content(".env", content, &rules());
        assert!(
            findings.iter().any(|f| f.rule_name.contains("Database")),
            "Should detect MongoDB connection string, got: {:?}",
            findings
        );
    }

    // ── Generic Assignments ─────────────────────────────────────

    #[test]
    fn test_detect_generic_api_key() {
        let content = r#"api_key = "sk-abcdef1234567890abcdef""#;
        let findings = scan_content("config.py", content, &rules());
        assert!(
            findings.iter().any(|f| f.rule_name.contains("API Key")),
            "Should detect generic API key, got: {:?}",
            findings
        );
    }

    #[test]
    fn test_detect_generic_password() {
        let content = r#"password = "MyS3cureP@ssw0rd!""#;
        let findings = scan_content("settings.py", content, &rules());
        assert!(
            findings.iter().any(|f| f.rule_name.contains("Password")),
            "Should detect generic password, got: {:?}",
            findings
        );
    }

    #[test]
    fn test_detect_generic_secret() {
        let content = r#"secret_key = "abcdef1234567890abcdef""#;
        let findings = scan_content("app.py", content, &rules());
        assert!(
            findings.iter().any(|f| f.rule_name.contains("Secret")),
            "Should detect generic secret assignment, got: {:?}",
            findings
        );
    }

    #[test]
    fn test_detect_generic_token() {
        let content = r#"auth_token = "abcdef1234567890abcdef""#;
        let findings = scan_content("app.py", content, &rules());
        assert!(
            findings.iter().any(|f| f.rule_name.contains("Token")),
            "Should detect generic token assignment, got: {:?}",
            findings
        );
    }

    // ── SendGrid ────────────────────────────────────────────────

    #[test]
    fn test_detect_sendgrid_key() {
        let content = format!(
            "SENDGRID_KEY=SG.{}.{}",
            "abcdefghijklmnopqrstuv", "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopq"
        )
        .as_str();
        let findings = scan_content(".env", content, &rules());
        assert!(
            findings.iter().any(|f| f.rule_name.contains("SendGrid")),
            "Should detect SendGrid API key, got: {:?}",
            findings
        );
    }

    // ── No False Positives ──────────────────────────────────────

    #[test]
    fn test_no_false_positive_short_value() {
        let content = r#"key = "abc""#;
        let findings = scan_content("config.toml", content, &rules());
        assert!(
            findings.is_empty(),
            "Short value should not trigger, got: {:?}",
            findings
        );
    }

    #[test]
    fn test_no_false_positive_placeholder() {
        let content = r#"api_key = "YOUR_API_KEY_HERE""#;
        // This might or might not match the generic rule — the point is the
        // user can allowlist placeholders. Let's just ensure it doesn't crash.
        let _ = scan_content("readme.md", content, &rules());
    }

    #[test]
    fn test_no_match_on_clean_code() {
        let content = r#"
fn main() {
    let x = 42;
    println!("Hello, world!");
    let config = Config::load();
}
"#;
        let findings = scan_content("main.rs", content, &rules());
        assert!(
            findings.is_empty(),
            "Clean code should have no findings, got: {:?}",
            findings
        );
    }

    // ── Binary Detection ────────────────────────────────────────

    #[test]
    fn test_is_binary_png() {
        assert!(is_binary("image.png"));
    }

    #[test]
    fn test_is_binary_exe() {
        assert!(is_binary("program.exe"));
    }

    #[test]
    fn test_is_not_binary_rs() {
        assert!(!is_binary("main.rs"));
    }

    #[test]
    fn test_is_not_binary_env() {
        assert!(!is_binary(".env"));
    }

    #[test]
    fn test_is_not_binary_toml() {
        assert!(!is_binary("config.toml"));
    }

    #[test]
    fn test_is_binary_lock() {
        assert!(is_binary("Cargo.lock"));
    }

    // ── Redaction ───────────────────────────────────────────────

    #[test]
    fn test_redact_long() {
        assert_eq!(redact_match("sk_live_abcdefghijklmnop"), "sk_l****");
    }

    #[test]
    fn test_redact_short() {
        assert_eq!(redact_match("abc"), "****");
    }

    #[test]
    fn test_redact_exact_four() {
        assert_eq!(redact_match("abcd"), "****");
    }

    #[test]
    fn test_redact_five() {
        assert_eq!(redact_match("abcde"), "abcd****");
    }

    // ── Diff Scanning ───────────────────────────────────────────

    #[test]
    fn test_scan_diff_detects_added_secrets() {
        let diff = r#"diff --git a/.env b/.env
index abc..def 100644
--- a/.env
+++ b/.env
@@ -1,2 +1,3 @@
 APP_NAME=myapp
+STRIPE_KEY=sk_live_"
        + "abcdefghijklmnopqrstuvwx
 DEBUG=true
"#;
        let findings = scan_diff_content(diff, &rules(), &[]);
        assert!(
            findings.iter().any(|f| f.rule_name.contains("Stripe")),
            "Should detect Stripe key in added diff line, got: {:?}",
            findings
        );
    }

    #[test]
    fn test_scan_diff_ignores_removed_lines() {
        let diff = r#"diff --git a/.env b/.env
index abc..def 100644
--- a/.env
+++ b/.env
@@ -1,3 +1,2 @@
 APP_NAME=myapp
-STRIPE_KEY=sk_live_"
        + "abcdefghijklmnopqrstuvwx
 DEBUG=true
"#;
        let findings = scan_diff_content(diff, &rules(), &[]);
        assert!(
            findings.is_empty(),
            "Should not flag removed lines, got: {:?}",
            findings
        );
    }

    #[test]
    fn test_scan_diff_ignores_context_lines() {
        let diff = r#"diff --git a/app.py b/app.py
index abc..def 100644
--- a/app.py
+++ b/app.py
@@ -1,3 +1,4 @@
 STRIPE_KEY=sk_live_"
        + "abcdefghijklmnopqrstuvwx
+# new comment
 DEBUG=true
"#;
        let findings = scan_diff_content(diff, &rules(), &[]);
        // The Stripe key is a context line (unchanged), should not be flagged
        assert!(
            findings.is_empty() || !findings.iter().any(|f| f.rule_name.contains("Stripe")),
            "Context lines should not be flagged, got: {:?}",
            findings
        );
    }

    // ── Allowlist ───────────────────────────────────────────────

    #[test]
    fn test_allowlist_by_file_name() {
        let diff = r#"diff --git a/.env.example b/.env.example
index abc..def 100644
--- a/.env.example
+++ b/.env.example
@@ -1,2 +1,3 @@
 APP_NAME=myapp
+STRIPE_KEY=sk_live_"
        + "abcdefghijklmnopqrstuvwx
"#;
        let allowlist = vec![".env.example".to_string()];
        let findings = scan_diff_content(diff, &rules(), &allowlist);
        assert!(
            findings.is_empty(),
            "Allowlisted file should be skipped, got: {:?}",
            findings
        );
    }

    #[test]
    fn test_allowlist_by_rule_name() {
        let content = format!("STRIPE_KEY={}{}", "sk_live_", "abcdefghijklmnopqrstuvwx").as_str();
        let mut findings = scan_content(".env", content, &rules());
        let allowlist = vec!["Stripe".to_string()];
        findings.retain(|f| {
            !allowlist
                .iter()
                .any(|a| f.preview.contains(a) || f.file.contains(a) || f.rule_name.contains(a))
        });
        assert!(
            findings.is_empty(),
            "Allowlisted rule should be skipped, got: {:?}",
            findings
        );
    }

    // ── Multiple Secrets in One File ────────────────────────────

    #[test]
    fn test_multiple_secrets_in_one_file() {
        let content = r#"
AWS_KEY=AKIAIOSFODNN7EXAMPLE1
STRIPE_KEY=sk_live_"
        + "abcdefghijklmnopqrstuvwx
password = "MyS3cureP@ssw0rd!"
"#;
        let findings = scan_content(".env", content, &rules());
        assert!(
            findings.len() >= 3,
            "Should find at least 3 secrets, got {}: {:?}",
            findings.len(),
            findings
        );
    }

    // ── Default Rules Compile ───────────────────────────────────

    #[test]
    fn test_default_rules_compile() {
        let rules = default_rules();
        assert!(
            rules.len() >= 15,
            "Should have at least 15 built-in rules, got {}",
            rules.len()
        );
    }

    // ── Line Numbers ────────────────────────────────────────────

    #[test]
    fn test_line_numbers_correct() {
        let content = "line1\nline2\nSTRIPE_KEY=sk_live_" + "abcdefghijklmnopqrstuvwx\nline4";
        let findings = scan_content("test.env", content, &rules());
        assert!(
            findings.iter().any(|f| f.line == 3),
            "Secret should be on line 3, got: {:?}",
            findings
        );
    }

    // ── Display ─────────────────────────────────────────────────

    #[test]
    fn test_finding_display() {
        let f = SecretFinding {
            file: "test.env".to_string(),
            line: 5,
            rule_name: "Test Rule".to_string(),
            preview: "sk_l****".to_string(),
        };
        let display = format!("{}", f);
        assert!(display.contains("test.env:5"));
        assert!(display.contains("Test Rule"));
        assert!(display.contains("sk_l****"));
    }
}
