pub mod client;
pub mod prompts;
pub mod provider;

/// Maximum diff content included in AI context (chars). Truncated beyond this.
pub const DIFF_TRUNCATE_AT: usize = 4000;
