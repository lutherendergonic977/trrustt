// TRRUSTT — AI Router
// Routes chat requests to the best available provider with fallback.
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, instrument};
use shared::{ChatRequest, ChatResponse, Result};
use crate::providers::AiProvider;
use crate::providers::openai::OpenAiProvider;
use crate::providers::anthropic::AnthropicProvider;
use crate::providers::gemini::GeminiProvider;
use crate::providers::deepseek::DeepSeekProvider;
use crate::providers::ollama::OllamaProvider;

pub struct AiRouter {
    providers: Arc<RwLock<HashMap<String, Box<dyn AiProvider>>>>,
    fallback_order: Vec<String>,
}

impl AiRouter {
    pub fn new() -> Self {
        let mut providers: HashMap<String, Box<dyn AiProvider>> = HashMap::new();
        providers.insert("openai".to_string(), Box::new(OpenAiProvider::new()));
        providers.insert("anthropic".to_string(), Box::new(AnthropicProvider::new()));
        providers.insert("gemini".to_string(), Box::new(GeminiProvider::new()));
        providers.insert("deepseek".to_string(), Box::new(DeepSeekProvider::new()));
        providers.insert("ollama".to_string(), Box::new(OllamaProvider::new()));
        let fallback_order = vec!["openai".into(), "anthropic".into(), "deepseek".into(), "gemini".into(), "ollama".into()];
        Self { providers: Arc::new(RwLock::new(providers)), fallback_order }
    }

    #[instrument(skip(self, request))]
    pub async fn route_chat(&self, request: &ChatRequest) -> Result<ChatResponse> {
        let name = request.provider.as_deref().unwrap_or("openai");
        if let Some(provider) = self.providers.read().await.get(name) {
            if provider.is_configured().await {
                return provider.chat(request).await;
            }
        }
        let mut errors = Vec::new();
        for fallback in &self.fallback_order {
            if fallback == name { continue; }
            if let Some(provider) = self.providers.read().await.get(fallback) {
                if provider.is_configured().await {
                    match provider.chat(request).await {
                        Ok(r) => { info!(provider=%fallback, "Fallback used"); return Ok(r); }
                        Err(e) => errors.push(format!("{}: {}", fallback, e)),
                    }
                }
            }
        }
        Err(shared::AppError::AiAllProvidersExhausted(errors))
    }

    pub async fn register(&self, name: &str, provider: Box<dyn AiProvider>) {
        self.providers.write().await.insert(name.to_string(), provider);
    }

    pub async fn is_configured(&self, name: &str) -> bool {
        self.providers.read().await.get(name).map(|p| p.is_configured()).unwrap_or(false)
    }
}

impl Default for AiRouter {
    fn default() -> Self { Self::new() }
}
