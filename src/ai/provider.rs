//! AI Provider trait and implementations for multi-model support.
//!
//! Each provider encapsulates how to talk to a specific AI backend.
//! The `AiClient` uses a `Box<dyn AiProvider>` to dispatch requests.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::AiConfig;

// ─── Provider Trait ────────────────────────────────────────────

/// A backend that can answer AI chat requests.
pub trait AiProvider: Send + Sync {
    /// Send a chat completion and return the response text.
    fn chat(&self, system_prompt: &str, user_message: &str) -> Result<String>;

    /// Health check — returns a descriptive string or error.
    fn health_check(&self) -> Result<String>;

    /// Display name (for status bar / UI).
    fn name(&self) -> &str;

    /// Model being used (for display).
    fn model_name(&self) -> &str;
}

// ─── Factory ───────────────────────────────────────────────────

/// Create the appropriate provider from the user's config.
/// Returns `None` if AI is disabled or mis-configured.
pub fn create_provider(config: &AiConfig) -> Option<Box<dyn AiProvider>> {
    if !config.enabled {
        return None;
    }

    let provider = config.effective_provider();
    let model = config.effective_model();
    let endpoint = config.effective_endpoint()?;
    let api_key = config.resolved_api_key().unwrap_or_default();
    let timeout = config.timeout_secs.unwrap_or(30);

    match provider {
        "bedrock" => {
            if api_key.is_empty() {
                return None;
            }
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(timeout))
                .build()
                .ok()?;
            Some(Box::new(BedrockProvider {
                endpoint,
                api_key,
                timeout,
                client,
            }))
        }
        "openai" | "openrouter" => {
            if api_key.is_empty() {
                return None;
            }
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(timeout))
                .build()
                .ok()?;
            Some(Box::new(OpenAiCompatibleProvider {
                endpoint,
                api_key,
                model,
                timeout,
                provider_name: provider.to_string(),
                client,
            }))
        }
        "anthropic" => {
            if api_key.is_empty() {
                return None;
            }
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(timeout))
                .build()
                .ok()?;
            Some(Box::new(AnthropicProvider {
                endpoint,
                api_key,
                model,
                timeout,
                client,
            }))
        }
        "ollama" => {
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(timeout))
                .build()
                .ok()?;
            Some(Box::new(OllamaProvider {
                endpoint,
                model,
                timeout,
                client,
            }))
        }
        _ => None,
    }
}

// ─── Bedrock (existing Lambda backend) ─────────────────────────

/// Sends the full `MentorRequest` JSON to the user's Lambda.
/// Lambda constructs prompts server-side.
pub struct BedrockProvider {
    pub endpoint: String,
    pub api_key: String,
    pub timeout: u64,
    client: reqwest::blocking::Client,
}

/// Bedrock-specific request that mirrors the Lambda's expected input.
#[derive(Serialize)]
struct BedrockRequest {
    #[serde(rename = "type")]
    request_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Lambda success response.
#[derive(Deserialize)]
struct BedrockApiResponse {
    success: bool,
    response: Option<BedrockResponseData>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct BedrockResponseData {
    content: Option<String>,
}

impl AiProvider for BedrockProvider {
    fn chat(&self, _system_prompt: &str, user_message: &str) -> Result<String> {
        // For Bedrock, we don't use system_prompt/user_message directly.
        // Instead, we reconstruct a MentorRequest. This method is called from
        // AiClient which will use `call_bedrock()` directly for Bedrock.
        // This fallback sends as an "explain" request.
        let req = BedrockRequest {
            request_type: "explain".to_string(),
            context: None,
            query: Some(user_message.to_string()),
            error: None,
        };

        let resp = self.client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .json(&req)
            .send()
            .context("Failed to reach AI backend")?;

        let status = resp.status().as_u16();
        if status != 200 {
            let body = resp.text().unwrap_or_default();
            anyhow::bail!("Bedrock API error (HTTP {}): {}", status, body);
        }

        let api_resp: BedrockApiResponse = resp.json().context("Failed to parse Bedrock response")?;
        if !api_resp.success {
            anyhow::bail!("Bedrock error: {}", api_resp.error.unwrap_or_default());
        }

        api_resp
            .response
            .and_then(|r| r.content)
            .ok_or_else(|| anyhow::anyhow!("Empty response from Bedrock"))
    }

    fn health_check(&self) -> Result<String> {
        let url = self.endpoint.replace("/mentor", "/health");
        let resp = self.client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .send()
            .context("Health check failed")?;

        let body = resp.text().unwrap_or_default();
        Ok(format!("Bedrock (Lambda): {}", body))
    }

    fn name(&self) -> &str {
        "Amazon Bedrock"
    }

    fn model_name(&self) -> &str {
        "Claude 3 Sonnet (via Lambda)"
    }
}

impl BedrockProvider {
    /// Send a raw `MentorRequest`-shaped JSON body to the Lambda endpoint.
    /// This is the primary path for Bedrock — keeps full compatibility.
    pub fn call_raw(&self, body: &serde_json::Value) -> Result<String> {
        let resp = self.client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .json(body)
            .send()
            .context("Failed to reach AI backend")?;

        let status = resp.status().as_u16();
        if status != 200 {
            let body = resp.text().unwrap_or_default();
            anyhow::bail!("Bedrock API error (HTTP {}): {}", status, body);
        }

        let api_resp: BedrockApiResponse = resp.json().context("Failed to parse response")?;
        if !api_resp.success {
            anyhow::bail!("{}", api_resp.error.unwrap_or("Unknown error".into()));
        }

        api_resp
            .response
            .and_then(|r| r.content)
            .ok_or_else(|| anyhow::anyhow!("Empty response"))
    }
}

// ─── OpenAI-Compatible (OpenAI, OpenRouter) ────────────────────

pub struct OpenAiCompatibleProvider {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub timeout: u64,
    pub provider_name: String,
    client: reqwest::blocking::Client,
}

#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Option<Vec<OpenAiChoice>>,
    error: Option<OpenAiError>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiRespMessage,
}

#[derive(Deserialize)]
struct OpenAiRespMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiError {
    message: String,
}

impl AiProvider for OpenAiCompatibleProvider {
    fn chat(&self, system_prompt: &str, user_message: &str) -> Result<String> {
        log::debug!(
            "[{}] chat: endpoint={} model={}",
            self.provider_name, self.endpoint, self.model
        );
        let req = OpenAiRequest {
            model: self.model.clone(),
            messages: vec![
                OpenAiMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                OpenAiMessage {
                    role: "user".to_string(),
                    content: user_message.to_string(),
                },
            ],
            max_tokens: 1024,
            temperature: 0.7,
        };

        let mut builder = self.client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key));

        // OpenRouter wants extra headers
        if self.provider_name == "openrouter" {
            builder = builder
                .header("HTTP-Referer", "https://github.com/JUSTMEETPATEL/zit")
                .header("X-Title", "zit");
        }

        let resp = builder
            .json(&req)
            .send()
            .context("Failed to reach AI backend")?;

        let status = resp.status().as_u16();
        let body_text = resp.text().unwrap_or_default();
        log::debug!("[{}] response: status={} body_len={}", self.provider_name, status, body_text.len());

        if status != 200 {
            log::error!("[{}] API error (HTTP {}): {}", self.provider_name, status, body_text);
            anyhow::bail!("{} API error (HTTP {}): {}", self.provider_name, status, body_text);
        }

        let parsed: OpenAiResponse =
            serde_json::from_str(&body_text).context("Failed to parse response")?;

        if let Some(err) = parsed.error {
            anyhow::bail!("{} error: {}", self.provider_name, err.message);
        }

        parsed
            .choices
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.message.content)
            .ok_or_else(|| anyhow::anyhow!("Empty response from {}", self.provider_name))
    }

    fn health_check(&self) -> Result<String> {
        // Simple connectivity test — send a tiny request
        let result = self.chat("You are a helpful assistant.", "Say 'ok'.");
        match result {
            Ok(_) => Ok(format!("{} ({}) — reachable ✓", self.provider_name, self.model)),
            Err(e) => Err(e),
        }
    }

    fn name(&self) -> &str {
        &self.provider_name
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

// ─── Anthropic ─────────────────────────────────────────────────

pub struct AnthropicProvider {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub timeout: u64,
    client: reqwest::blocking::Client,
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<AnthropicMessage>,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Option<Vec<AnthropicContent>>,
    error: Option<AnthropicError>,
}

#[derive(Deserialize)]
struct AnthropicContent {
    text: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicError {
    message: String,
}

impl AiProvider for AnthropicProvider {
    fn chat(&self, system_prompt: &str, user_message: &str) -> Result<String> {
        let req = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 1024,
            system: system_prompt.to_string(),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: user_message.to_string(),
            }],
        };

        let resp = self.client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&req)
            .send()
            .context("Failed to reach Anthropic API")?;

        let status = resp.status().as_u16();
        let body_text = resp.text().unwrap_or_default();

        if status != 200 {
            anyhow::bail!("Anthropic API error (HTTP {}): {}", status, body_text);
        }

        let parsed: AnthropicResponse =
            serde_json::from_str(&body_text).context("Failed to parse Anthropic response")?;

        if let Some(err) = parsed.error {
            anyhow::bail!("Anthropic error: {}", err.message);
        }

        parsed
            .content
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.text)
            .ok_or_else(|| anyhow::anyhow!("Empty response from Anthropic"))
    }

    fn health_check(&self) -> Result<String> {
        let result = self.chat("You are a helpful assistant.", "Say 'ok'.");
        match result {
            Ok(_) => Ok(format!("Anthropic ({}) — reachable ✓", self.model)),
            Err(e) => Err(e),
        }
    }

    fn name(&self) -> &str {
        "Anthropic"
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

// ─── Ollama (local) ────────────────────────────────────────────

pub struct OllamaProvider {
    pub endpoint: String,
    pub model: String,
    pub timeout: u64,
    client: reqwest::blocking::Client,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    message: Option<OllamaRespMessage>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct OllamaRespMessage {
    content: Option<String>,
}

impl AiProvider for OllamaProvider {
    fn chat(&self, system_prompt: &str, user_message: &str) -> Result<String> {
        let req = OllamaRequest {
            model: self.model.clone(),
            messages: vec![
                OllamaMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                OllamaMessage {
                    role: "user".to_string(),
                    content: user_message.to_string(),
                },
            ],
            stream: false,
        };

        let url = format!("{}/api/chat", self.endpoint.trim_end_matches('/'));

        let resp = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .context("Failed to reach Ollama — is it running? (ollama serve)")?;

        let status = resp.status().as_u16();
        let body_text = resp.text().unwrap_or_default();

        if status != 200 {
            anyhow::bail!("Ollama error (HTTP {}): {}", status, body_text);
        }

        let parsed: OllamaResponse =
            serde_json::from_str(&body_text).context("Failed to parse Ollama response")?;

        if let Some(err) = parsed.error {
            anyhow::bail!("Ollama error: {}", err);
        }

        parsed
            .message
            .and_then(|m| m.content)
            .ok_or_else(|| anyhow::anyhow!("Empty response from Ollama"))
    }

    fn health_check(&self) -> Result<String> {
        let url = format!("{}/api/tags", self.endpoint.trim_end_matches('/'));

        let resp = self.client
            .get(&url)
            .send()
            .context("Cannot reach Ollama — is it running? (ollama serve)")?;

        if resp.status().is_success() {
            Ok(format!("Ollama ({}) at {} — reachable ✓", self.model, self.endpoint))
        } else {
            anyhow::bail!("Ollama returned HTTP {}", resp.status().as_u16())
        }
    }

    fn name(&self) -> &str {
        "Ollama"
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

// ─── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AiConfig;

    #[test]
    fn test_create_provider_disabled() {
        let config = AiConfig::default(); // not enabled
        assert!(create_provider(&config).is_none());
    }

    #[test]
    fn test_create_provider_bedrock() {
        let config = AiConfig {
            enabled: true,
            provider: "bedrock".to_string(),
            model: None,
            endpoint: Some("https://example.com/mentor".to_string()),
            api_key: Some("test-key-12345".to_string()),
            timeout_secs: Some(30),
        };
        let p = create_provider(&config);
        assert!(p.is_some());
        assert_eq!(p.unwrap().name(), "Amazon Bedrock");
    }

    #[test]
    fn test_create_provider_openai() {
        let config = AiConfig {
            enabled: true,
            provider: "openai".to_string(),
            model: Some("gpt-4o".to_string()),
            endpoint: None,
            api_key: Some("sk-test12345678".to_string()),
            timeout_secs: Some(30),
        };
        let p = create_provider(&config);
        assert!(p.is_some());
        assert_eq!(p.unwrap().name(), "openai");
    }

    #[test]
    fn test_create_provider_anthropic() {
        let config = AiConfig {
            enabled: true,
            provider: "anthropic".to_string(),
            model: None,
            endpoint: None,
            api_key: Some("sk-ant-test1234".to_string()),
            timeout_secs: Some(30),
        };
        let p = create_provider(&config);
        assert!(p.is_some());
        assert_eq!(p.unwrap().name(), "Anthropic");
    }

    #[test]
    fn test_create_provider_ollama_no_key() {
        std::env::remove_var("ZIT_AI_API_KEY");
        let config = AiConfig {
            enabled: true,
            provider: "ollama".to_string(),
            model: None,
            endpoint: None,
            api_key: None,
            timeout_secs: Some(30),
        };
        let p = create_provider(&config);
        assert!(p.is_some());
        assert_eq!(p.unwrap().name(), "Ollama");
    }

    #[test]
    fn test_create_provider_unknown() {
        let config = AiConfig {
            enabled: true,
            provider: "unknown".to_string(),
            model: None,
            endpoint: None,
            api_key: None,
            timeout_secs: None,
        };
        assert!(create_provider(&config).is_none());
    }
}
