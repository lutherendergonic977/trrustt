// TRRUSTT — AI Providers
// Common trait and all provider implementations.

use async_trait::async_trait;
use std::env;
use shared::{ChatRequest, ChatResponse, ChatMessage, TokenUsage, Result};
use chrono::Utc;

// ── Common Provider Trait ───────────────────────────────────────────

#[async_trait]
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn is_configured(&self) -> bool;
    async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse>;
    fn available_models(&self) -> Vec<String>;
}

#[derive(Debug, Clone)]
pub struct ModelPricing {
    pub input_cost_per_1m: f64,
    pub output_cost_per_1m: f64,
}

impl ModelPricing {
    pub fn calculate_cost(&self, input_tokens: i64, output_tokens: i64) -> f64 {
        (input_tokens as f64 / 1_000_000.0) * self.input_cost_per_1m
            + (output_tokens as f64 / 1_000_000.0) * self.output_cost_per_1m
    }
}

/// Get the API key for a provider from environment variables.
fn get_api_key(provider_env_var: &str) -> Option<String> {
    env::var(provider_env_var).ok().filter(|k| !k.is_empty())
}

// ── OpenAI Provider ─────────────────────────────────────────────────

pub mod openai {
    use super::*;

    pub struct OpenAiProvider { api_key: Option<String> }

    impl OpenAiProvider {
        pub fn new() -> Self {
            Self { api_key: get_api_key("OPENAI_API_KEY") }
        }
    }

    #[async_trait]
    impl AiProvider for OpenAiProvider {
        fn name(&self) -> &str { "openai" }
        async fn is_configured(&self) -> bool { self.api_key.is_some() }
        async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
            let api_key = self.api_key.as_ref()
                .ok_or_else(|| shared::AppError::AiProviderNotConfigured("openai".into(), "OPENAI_API_KEY".into()))?;

            let client = reqwest::Client::new();
            let model = request.model.as_deref().unwrap_or("gpt-4o");
            let messages: Vec<serde_json::Value> = std::iter::once(
                request.system_prompt.as_ref().map(|s| serde_json::json!({"role": "system", "content": s}))
            ).flatten().chain(
                request.history.iter().map(|m| serde_json::json!({"role": m.role, "content": m.content}))
            ).chain(std::iter::once(
                serde_json::json!({"role": "user", "content": request.user_message})
            )).collect();

            let body = serde_json::json!({
                "model": model,
                "messages": messages,
                "temperature": request.temperature.unwrap_or(0.3),
                "max_tokens": request.max_tokens.unwrap_or(4096),
            });

            let resp = client.post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send().await.map_err(|e| shared::AppError::AiProvider { provider: "openai".into(), message: e.to_string() })?;

            if !resp.status().is_success() {
                let status = resp.status().as_u16();
                let err_body = resp.text().await.unwrap_or_default();
                return Err(shared::AppError::AiProvider { provider: "openai".into(), message: format!("HTTP {}: {}", status, err_body) });
            }

            let json: serde_json::Value = resp.json().await.map_err(|e| shared::AppError::AiJsonParse(e.to_string()))?;
            let content = json["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
            let usage = &json["usage"];
            let input_tokens = usage["prompt_tokens"].as_i64().unwrap_or(0);
            let output_tokens = usage["completion_tokens"].as_i64().unwrap_or(0);
            let pricing = ModelPricing { input_cost_per_1m: 2.50, output_cost_per_1m: 10.0 };

            Ok(ChatResponse {
                content,
                provider: "openai".to_string(),
                model: model.to_string(),
                tokens: TokenUsage { input_tokens, output_tokens, total_tokens: input_tokens + output_tokens },
                cost_usd: pricing.calculate_cost(input_tokens, output_tokens),
                duration_ms: 0,
                from_cache: false,
            })
        }
        fn available_models(&self) -> Vec<String> { vec!["gpt-4o".into(), "gpt-4o-mini".into(), "gpt-4-turbo".into()] }
    }
}

// ── Anthropic Provider ──────────────────────────────────────────────

pub mod anthropic {
    use super::*;

    pub struct AnthropicProvider { api_key: Option<String> }

    impl AnthropicProvider {
        pub fn new() -> Self {
            Self { api_key: get_api_key("ANTHROPIC_API_KEY") }
        }
    }

    #[async_trait]
    impl AiProvider for AnthropicProvider {
        fn name(&self) -> &str { "anthropic" }
        async fn is_configured(&self) -> bool { self.api_key.is_some() }
        async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
            let api_key = self.api_key.as_ref()
                .ok_or_else(|| shared::AppError::AiProviderNotConfigured("anthropic".into(), "ANTHROPIC_API_KEY".into()))?;

            let client = reqwest::Client::new();
            let model = request.model.as_deref().unwrap_or("claude-3-sonnet-20240229");
            let system = request.system_prompt.clone().unwrap_or_default();
            let messages: Vec<serde_json::Value> = request.history.iter()
                .map(|m| serde_json::json!({"role": m.role, "content": m.content}))
                .chain(std::iter::once(serde_json::json!({"role": "user", "content": request.user_message})))
                .collect();

            let body = serde_json::json!({
                "model": model,
                "system": system,
                "messages": messages,
                "max_tokens": request.max_tokens.unwrap_or(4096),
                "temperature": request.temperature.unwrap_or(0.3),
            });

            let resp = client.post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&body)
                .send().await.map_err(|e| shared::AppError::AiProvider { provider: "anthropic".into(), message: e.to_string() })?;

            let json: serde_json::Value = resp.json().await.map_err(|e| shared::AppError::AiJsonParse(e.to_string()))?;
            let content = json["content"][0]["text"].as_str().unwrap_or("").to_string();
            let input_tokens = json["usage"]["input_tokens"].as_i64().unwrap_or(0);
            let output_tokens = json["usage"]["output_tokens"].as_i64().unwrap_or(0);
            let pricing = ModelPricing { input_cost_per_1m: 3.0, output_cost_per_1m: 15.0 };

            Ok(ChatResponse {
                content, provider: "anthropic".to_string(), model: model.to_string(),
                tokens: TokenUsage { input_tokens, output_tokens, total_tokens: input_tokens + output_tokens },
                cost_usd: pricing.calculate_cost(input_tokens, output_tokens),
                duration_ms: 0, from_cache: false,
            })
        }
        fn available_models(&self) -> Vec<String> { vec!["claude-3-opus-20240229".into(), "claude-3-sonnet-20240229".into(), "claude-3-haiku-20240307".into()] }
    }
}

// ── Google Gemini Provider ──────────────────────────────────────────

pub mod gemini {
    use super::*;

    pub struct GeminiProvider { api_key: Option<String> }

    impl GeminiProvider {
        pub fn new() -> Self {
            Self { api_key: get_api_key("GOOGLE_API_KEY") }
        }
    }

    #[async_trait]
    impl AiProvider for GeminiProvider {
        fn name(&self) -> &str { "google" }
        async fn is_configured(&self) -> bool { self.api_key.is_some() }
        async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
            let api_key = self.api_key.as_ref()
                .ok_or_else(|| shared::AppError::AiProviderNotConfigured("google".into(), "GOOGLE_API_KEY".into()))?;

            let client = reqwest::Client::new();
            let model = request.model.as_deref().unwrap_or("gemini-1.5-pro");
            let system_instruction = request.system_prompt.clone();
            let contents: Vec<serde_json::Value> = request.history.iter()
                .map(|m| serde_json::json!({"role": if m.role == "assistant" { "model" } else { "user" }, "parts": [{"text": m.content}]}))
                .chain(std::iter::once(serde_json::json!({"role": "user", "parts": [{"text": request.user_message}]})))
                .collect();

            let mut body = serde_json::json!({
                "contents": contents,
                "generationConfig": {
                    "temperature": request.temperature.unwrap_or(0.3),
                    "maxOutputTokens": request.max_tokens.unwrap_or(4096),
                }
            });
            if let Some(sys) = system_instruction {
                body["systemInstruction"] = serde_json::json!({"parts": [{"text": sys}]});
            }

            let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", model, api_key);
            let resp = client.post(&url)
                .header("Content-Type", "application/json")
                .json(&body)
                .send().await.map_err(|e| shared::AppError::AiProvider { provider: "google".into(), message: e.to_string() })?;

            let json: serde_json::Value = resp.json().await.map_err(|e| shared::AppError::AiJsonParse(e.to_string()))?;
            let content = json["candidates"][0]["content"]["parts"][0]["text"].as_str().unwrap_or("").to_string();
            let usage = &json.get("usageMetadata").cloned().unwrap_or(serde_json::json!({}));
            let input_tokens = usage["promptTokenCount"].as_i64().unwrap_or(0);
            let output_tokens = usage["candidatesTokenCount"].as_i64().unwrap_or(0);
            let pricing = ModelPricing { input_cost_per_1m: 1.25, output_cost_per_1m: 5.0 };

            Ok(ChatResponse {
                content, provider: "google".to_string(), model: model.to_string(),
                tokens: TokenUsage { input_tokens, output_tokens, total_tokens: input_tokens + output_tokens },
                cost_usd: pricing.calculate_cost(input_tokens, output_tokens),
                duration_ms: 0, from_cache: false,
            })
        }
        fn available_models(&self) -> Vec<String> { vec!["gemini-1.5-pro".into(), "gemini-1.5-flash".into()] }
    }
}

// ── DeepSeek Provider ───────────────────────────────────────────────

pub mod deepseek {
    use super::*;

    pub struct DeepSeekProvider { api_key: Option<String> }

    impl DeepSeekProvider {
        pub fn new() -> Self {
            Self { api_key: get_api_key("DEEPSEEK_API_KEY") }
        }
    }

    #[async_trait]
    impl AiProvider for DeepSeekProvider {
        fn name(&self) -> &str { "deepseek" }
        async fn is_configured(&self) -> bool { self.api_key.is_some() }
        async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
            let api_key = self.api_key.as_ref()
                .ok_or_else(|| shared::AppError::AiProviderNotConfigured("deepseek".into(), "DEEPSEEK_API_KEY".into()))?;

            let client = reqwest::Client::new();
            let model = request.model.as_deref().unwrap_or("deepseek-chat");
            let messages: Vec<serde_json::Value> = std::iter::once(
                request.system_prompt.as_ref().map(|s| serde_json::json!({"role": "system", "content": s}))
            ).flatten().chain(
                request.history.iter().map(|m| serde_json::json!({"role": m.role, "content": m.content}))
            ).chain(std::iter::once(
                serde_json::json!({"role": "user", "content": request.user_message})
            )).collect();

            let body = serde_json::json!({
                "model": model,
                "messages": messages,
                "temperature": request.temperature.unwrap_or(0.3),
                "max_tokens": request.max_tokens.unwrap_or(4096),
            });

            let resp = client.post("https://api.deepseek.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send().await.map_err(|e| shared::AppError::AiProvider { provider: "deepseek".into(), message: e.to_string() })?;

            let json: serde_json::Value = resp.json().await.map_err(|e| shared::AppError::AiJsonParse(e.to_string()))?;
            let content = json["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();
            let usage = &json["usage"];
            let input_tokens = usage["prompt_tokens"].as_i64().unwrap_or(0);
            let output_tokens = usage["completion_tokens"].as_i64().unwrap_or(0);
            let pricing = ModelPricing { input_cost_per_1m: 0.14, output_cost_per_1m: 0.28 };

            Ok(ChatResponse {
                content, provider: "deepseek".to_string(), model: model.to_string(),
                tokens: TokenUsage { input_tokens, output_tokens, total_tokens: input_tokens + output_tokens },
                cost_usd: pricing.calculate_cost(input_tokens, output_tokens),
                duration_ms: 0, from_cache: false,
            })
        }
        fn available_models(&self) -> Vec<String> { vec!["deepseek-chat".into(), "deepseek-reasoner".into()] }
    }
}

// ── Ollama Provider (local) ─────────────────────────────────────────

pub mod ollama {
    use super::*;

    pub struct OllamaProvider { base_url: String }

    impl OllamaProvider {
        pub fn new() -> Self {
            Self { base_url: env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".into()) }
        }
    }

    #[async_trait]
    impl AiProvider for OllamaProvider {
        fn name(&self) -> &str { "ollama" }
        async fn is_configured(&self) -> bool { true } // Ollama is local, always "configured"
        async fn chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
            let client = reqwest::Client::new();
            let model = request.model.as_deref().unwrap_or("llama3");
            let messages: Vec<serde_json::Value> = std::iter::once(
                request.system_prompt.as_ref().map(|s| serde_json::json!({"role": "system", "content": s}))
            ).flatten().chain(
                request.history.iter().map(|m| serde_json::json!({"role": m.role, "content": m.content}))
            ).chain(std::iter::once(
                serde_json::json!({"role": "user", "content": request.user_message})
            )).collect();

            let body = serde_json::json!({
                "model": model,
                "messages": messages,
                "stream": false,
                "options": {
                    "temperature": request.temperature.unwrap_or(0.3),
                    "num_predict": request.max_tokens.unwrap_or(4096),
                }
            });

            let resp = client.post(format!("{}/api/chat", self.base_url))
                .json(&body)
                .send().await.map_err(|e| shared::AppError::AiProvider { provider: "ollama".into(), message: e.to_string() })?;

            let json: serde_json::Value = resp.json().await.map_err(|e| shared::AppError::AiJsonParse(e.to_string()))?;
            let content = json["message"]["content"].as_str().unwrap_or("").to_string();
            let input_tokens = json.get("prompt_eval_count").and_then(|v| v.as_i64()).unwrap_or(0);
            let output_tokens = json.get("eval_count").and_then(|v| v.as_i64()).unwrap_or(0);

            Ok(ChatResponse {
                content, provider: "ollama".to_string(), model: model.to_string(),
                tokens: TokenUsage { input_tokens, output_tokens, total_tokens: input_tokens + output_tokens },
                cost_usd: 0.0, // Ollama is free (local)
                duration_ms: 0, from_cache: false,
            })
        }
        fn available_models(&self) -> Vec<String> { vec!["llama3".into(), "mistral".into(), "codellama".into()] }
    }
}
