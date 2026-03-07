//! System prompts for AI Mentor — ported from `aws/lambda/prompts.py`.
//!
//! These are used by direct providers (OpenAI, Anthropic, OpenRouter, Ollama).
//! The Bedrock provider sends raw requests to Lambda, which has its own copy.

use crate::ai::client::RepoContext;
use super::DIFF_TRUNCATE_AT;

// ─── System Prompts ────────────────────────────────────────────

pub const PROMPT_EXPLAIN: &str = r#"You are a friendly Git mentor helping developers understand their repository state.

Your role:
- Explain Git concepts in plain, simple English
- Be concise but thorough
- Use analogies when helpful
- Never assume advanced Git knowledge
- Focus on what the user needs to know right now

Format your response as:
1. A brief summary (1-2 sentences)
2. Key details (bullet points)
3. What this means for the user (1 sentence)

Keep responses under 200 words."#;

pub const PROMPT_ERROR: &str = r#"You are a Git troubleshooter helping developers understand and fix Git errors.

Your role:
- Translate cryptic Git errors into plain English
- Explain WHY the error occurred
- Provide step-by-step fix instructions
- Warn about any risks in the suggested fix
- Prioritize SAFE solutions over quick ones

Format your response as:
1. What happened (1-2 sentences)
2. Why it happened (1-2 sentences)
3. How to fix it (numbered steps)
4. How to prevent it (optional, 1 sentence)

Keep responses under 250 words."#;

pub const PROMPT_RECOMMEND: &str = r#"You are a cautious Git advisor helping developers choose safe operations.

Your role:
- Always prioritize data safety
- Recommend non-destructive operations when possible
- Clearly label operations as: SAFE / CAUTION / DESTRUCTIVE
- Explain trade-offs between different approaches
- If the user wants something risky, suggest safer alternatives first

Format your response as:
1. Recommended approach (with safety label)
2. Steps to execute
3. Alternative approaches (if any)
4. What to do if something goes wrong

Keep responses under 200 words."#;

pub const PROMPT_COMMIT: &str = r#"You are a commit message assistant following conventional commit standards.

Generate commit messages that are:
- Under 50 characters for the subject line
- In imperative mood ("Add feature" not "Added feature")
- Descriptive but concise
- Following conventional commits format when appropriate (feat:, fix:, docs:, etc.)

Provide 2-3 suggestions, from most specific to most general."#;

pub const PROMPT_LEARN: &str = r#"You are an expert Git teacher helping developers learn Git concepts.

Your role:
- Explain Git concepts clearly for beginners
- Use real-world analogies (like snapshots, timelines, parallel universes)
- Provide practical examples they can try
- Connect theory to their current repository state when available
- Build confidence by starting simple and adding complexity

Format your response as:
1. Simple explanation (2-3 sentences, no jargon)
2. An analogy or mental model
3. Practical example with commands
4. Quick tip or common mistake to avoid

Keep responses under 300 words."#;

pub const PROMPT_REVIEW: &str = r#"You are an expert code reviewer helping developers improve their changes.

Your role:
- Review diffs for bugs, logic errors, and edge cases
- Highlight security concerns or performance issues
- Suggest concrete improvements with brief code snippets when helpful
- Note positive patterns worth keeping
- Be constructive, specific, and actionable

Format your response as:
1. Summary (1 sentence overall assessment)
2. Issues found (bullet points, severity: 🔴 Critical / 🟡 Warning / 🔵 Info)
3. Suggestions (numbered, most important first)
4. Good patterns (optional, things done well)

Keep responses under 250 words."#;

pub const PROMPT_MERGE_RESOLVE: &str = r#"You are an expert Git merge conflict resolver helping developers safely resolve conflicts.

Your role:
- Analyze conflict markers (<<<<<<< HEAD, =======, >>>>>>>) to understand both sides
- Determine which changes should be kept based on code logic and intent
- Provide a clear recommendation: ACCEPT_CURRENT, ACCEPT_INCOMING, or MERGE_BOTH
- When recommending MERGE_BOTH, provide the exact resolved content
- Explain WHY one side should be preferred
- Warn about potential issues (logic breaks, missing imports, etc.)

Format your response EXACTLY as:
RECOMMENDATION: <ACCEPT_CURRENT|ACCEPT_INCOMING|MERGE_BOTH>

EXPLANATION:
<2-3 sentences explaining the reasoning>

CURRENT CHANGES (HEAD):
<brief description of what the current branch changed>

INCOMING CHANGES:
<brief description of what the incoming branch changed>

RESOLVED CONTENT:
```
<the final resolved code if MERGE_BOTH, or state which side to keep>
```

FOLLOW-UP:
- <actionable next step 1>
- <actionable next step 2>

Keep responses under 400 words."#;

pub const PROMPT_MERGE_STRATEGY: &str = r#"You are a cautious Git merge/rebase advisor helping developers choose the safest integration strategy.

Your role:
- Analyze the branch topology (ahead/behind counts, conflict potential)
- Recommend the safest merge strategy: MERGE_NO_FF, REBASE, FAST_FORWARD, or MERGE_SQUASH
- Label each option with safety level: ✅ SAFE / ⚠️ CAUTION / 🔴 RISKY
- Consider: shared branches (never rebase), conflict count, commit history cleanliness
- Provide the exact git commands to execute

Format your response EXACTLY as:
RECOMMENDED: <strategy name>
SAFETY: <✅ SAFE|⚠️ CAUTION|🔴 RISKY>

WHY:
<2-3 sentences explaining the recommendation>

COMMANDS:
```
<exact git commands to run, one per line>
```

ALTERNATIVES:
1. <alternative strategy> (<safety label>) - <one line reason>
2. <alternative strategy> (<safety label>) - <one line reason>

WARNINGS:
- <potential issue to watch for>

FOLLOW-UP:
- <what to do after the merge>
- <how to verify everything is correct>

Keep responses under 300 words."#;

pub const PROMPT_GITIGNORE: &str = r#"You are an expert developer helping create the perfect .gitignore file for a project.

Your role:
- Analyze the project's file structure to determine the tech stack, build tools, and frameworks
- Generate a comprehensive .gitignore that covers all relevant patterns
- Organize rules into clearly commented sections
- Include common OS-specific files (macOS .DS_Store, Windows Thumbs.db, etc.)
- Include IDE/editor files (.idea/, .vscode/, *.swp, etc.)
- Never ignore files that are essential to the project (source code, configs checked in intentionally)
- If an existing .gitignore is provided, improve and extend it

Output ONLY the raw .gitignore content — no markdown code fences, no explanations before or after.
Use comments (lines starting with #) to organize sections, for example:
# Dependencies
# Build output
# IDE / Editor
# OS generated files
# Environment / secrets

Keep the output clean and production-ready."#;

// ─── Lookup ────────────────────────────────────────────────────

/// Return the system prompt for a given request type.
pub fn system_prompt_for(request_type: &str) -> &'static str {
    match request_type {
        "explain" => PROMPT_EXPLAIN,
        "error" => PROMPT_ERROR,
        "recommend" => PROMPT_RECOMMEND,
        "commit_suggestion" | "commit" => PROMPT_COMMIT,
        "learn" => PROMPT_LEARN,
        "review" => PROMPT_REVIEW,
        "merge_resolve" => PROMPT_MERGE_RESOLVE,
        "merge_strategy" => PROMPT_MERGE_STRATEGY,
        "generate_gitignore" => PROMPT_GITIGNORE,
        _ => PROMPT_EXPLAIN,
    }
}

// ─── Context formatting (port of Python `format_context`) ──────

/// Format a `RepoContext` into a human-readable string for the LLM.
pub fn format_context(ctx: &RepoContext) -> String {
    let mut lines: Vec<String> = Vec::new();

    if let Some(ref b) = ctx.branch {
        lines.push(format!("Current Branch: {}", b));
    }
    if !ctx.staged_files.is_empty() {
        lines.push(format!(
            "Staged Files ({}): {}",
            ctx.staged_files.len(),
            ctx.staged_files.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    if !ctx.unstaged_files.is_empty() {
        lines.push(format!(
            "Unstaged Files ({}): {}",
            ctx.unstaged_files.len(),
            ctx.unstaged_files.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    if ctx.has_conflicts {
        lines.push("⚠️ MERGE CONFLICTS PRESENT".to_string());
    }
    if !ctx.conflict_files.is_empty() {
        lines.push(format!(
            "Conflicted Files ({}): {}",
            ctx.conflict_files.len(),
            ctx.conflict_files.iter().take(10).cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    if let Some(ref cd) = ctx.conflict_diff {
        let trimmed: String = cd.chars().take(DIFF_TRUNCATE_AT).collect();
        lines.push(format!("Conflict Content:\n{}", trimmed));
    }
    if let Some(ref mt) = ctx.merge_type {
        lines.push(format!("Merge Type: {}", mt));
    }
    if ctx.detached_head {
        lines.push("⚠️ DETACHED HEAD STATE".to_string());
    }
    if let Some(ref d) = ctx.diff {
        let trimmed: String = d.chars().take(DIFF_TRUNCATE_AT).collect();
        lines.push(format!("Diff:\n{}", trimmed));
    }

    if lines.is_empty() {
        "No context provided".to_string()
    } else {
        lines.join("\n")
    }
}

/// Build a complete user message for a given request type.
pub fn build_user_message(
    request_type: &str,
    ctx: &RepoContext,
    query: Option<&str>,
    error: Option<&str>,
) -> String {
    let context_str = format_context(ctx);

    match request_type {
        "error" => {
            let err = error.unwrap_or("Unknown error");
            format!(
                "Git Error Message:\n{}\n\nRepository Context:\n{}\n\nPlease explain this error and suggest how to fix it.",
                err, context_str
            )
        }
        "commit_suggestion" | "commit" => {
            let staged = if ctx.staged_files.is_empty() {
                "None specified".to_string()
            } else {
                ctx.staged_files.join(", ")
            };
            let diff_preview = ctx.diff.as_ref().map(|d| {
                let trimmed: String = d.chars().take(DIFF_TRUNCATE_AT).collect();
                format!("Diff Preview: {}...", trimmed)
            }).unwrap_or_default();

            let ds = ctx.diff_stats.as_ref();
            format!(
                "Staged Files: {}\nDiff Statistics:\n- Files changed: {}\n- Insertions: {}\n- Deletions: {}\n{}\n\nSuggest a concise, conventional commit message.",
                staged,
                ds.map_or(ctx.staged_files.len(), |s| s.files_changed),
                ds.map_or(0, |s| s.insertions),
                ds.map_or(0, |s| s.deletions),
                diff_preview
            )
        }
        "learn" => {
            let topic = query.unwrap_or("basic Git workflow");
            format!(
                "Repository Context:\n{}\n\nTopic to learn about: {}\n\nProvide a beginner-friendly explanation with practical examples.",
                context_str, topic
            )
        }
        "review" => {
            let diff = ctx.diff.as_deref().unwrap_or("No diff provided");
            let trimmed: String = diff.chars().take(DIFF_TRUNCATE_AT).collect();
            let staged = if ctx.staged_files.is_empty() {
                "Unknown".to_string()
            } else {
                ctx.staged_files.join(", ")
            };
            let notes = query.map(|q| format!("Reviewer Notes: {}", q))
                .unwrap_or_else(|| "Review this diff for issues and improvements.".to_string());
            format!(
                "Repository Context:\n{}\n\nFiles Under Review: {}\n\nDiff Content:\n{}\n\n{}",
                context_str, staged, trimmed, notes
            )
        }
        "merge_resolve" => {
            let conflict_diff = ctx.conflict_diff.as_deref().unwrap_or("No conflict content provided");
            let trimmed: String = conflict_diff.chars().take(DIFF_TRUNCATE_AT).collect();
            let conflict_files = if ctx.conflict_files.is_empty() {
                "Unknown".to_string()
            } else {
                ctx.conflict_files.join(", ")
            };
            let notes = query.map(|q| format!("Developer Notes: {}", q))
                .unwrap_or_else(|| "Analyze this merge conflict and recommend the best resolution.".to_string());
            format!(
                "Repository Context:\n{}\n\nConflicted Files: {}\n\nConflict Content (with markers):\n{}\n\n{}",
                context_str, conflict_files, trimmed, notes
            )
        }
        "merge_strategy" => {
            let notes = query.map(|q| format!("Developer Question: {}", q))
                .unwrap_or_else(|| "What is the safest strategy to integrate these branches?".to_string());
            format!(
                "Repository Context:\n{}\n\n{}\n\nPlease recommend the best merge/rebase strategy given the current repository state.",
                context_str, notes
            )
        }
        "generate_gitignore" => {
            let file_listing = query.unwrap_or("No file listing available.");
            let existing = error.map(|e| format!("\n\nExisting .gitignore:\n{}", e))
                .unwrap_or_default();
            format!(
                "Project File Structure:\n{}{}\n\nGenerate a comprehensive .gitignore for this project.",
                file_listing, existing
            )
        }
        _ => {
            // explain, recommend, ask_question
            let q = query.unwrap_or("Explain the current repository state.");
            format!(
                "Repository Context:\n{}\n\nUser Question: {}",
                context_str, q
            )
        }
    }
}

// ─── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt_for_all_types() {
        let types = ["explain", "error", "recommend", "commit_suggestion", "learn", "review", "merge_resolve", "merge_strategy"];
        for t in &types {
            let prompt = system_prompt_for(t);
            assert!(!prompt.is_empty(), "prompt for '{}' should not be empty", t);
        }
    }

    #[test]
    fn test_system_prompt_unknown_falls_back() {
        assert_eq!(system_prompt_for("nonexistent"), PROMPT_EXPLAIN);
    }

    #[test]
    fn test_format_context_empty() {
        let ctx = RepoContext {
            branch: None,
            staged_files: vec![],
            unstaged_files: vec![],
            diff_stats: None,
            diff: None,
            conflict_files: vec![],
            conflict_diff: None,
            has_conflicts: false,
            merge_type: None,
            detached_head: false,
        };
        assert_eq!(format_context(&ctx), "No context provided");
    }

    #[test]
    fn test_format_context_with_conflicts() {
        let ctx = RepoContext {
            branch: Some("main".to_string()),
            staged_files: vec![],
            unstaged_files: vec![],
            diff_stats: None,
            diff: None,
            conflict_files: vec!["app.py".to_string()],
            conflict_diff: Some("<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>>".to_string()),
            has_conflicts: true,
            merge_type: Some("merge".to_string()),
            detached_head: false,
        };
        let out = format_context(&ctx);
        assert!(out.contains("MERGE CONFLICTS"));
        assert!(out.contains("app.py"));
    }

    #[test]
    fn test_build_user_message_commit() {
        let ctx = RepoContext {
            branch: Some("main".to_string()),
            staged_files: vec!["src/main.rs".to_string()],
            unstaged_files: vec![],
            diff_stats: None,
            diff: Some("+new line".to_string()),
            conflict_files: vec![],
            conflict_diff: None,
            has_conflicts: false,
            merge_type: None,
            detached_head: false,
        };
        let msg = build_user_message("commit_suggestion", &ctx, None, None);
        assert!(msg.contains("src/main.rs"));
        assert!(msg.contains("commit message"));
    }
}
