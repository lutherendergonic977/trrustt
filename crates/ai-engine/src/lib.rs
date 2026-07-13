// TRRUSTT — AI Engine
// Multi-provider LLM routing, RAG, vision, prompt management.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod router;
pub mod providers;
pub mod prompt;
pub mod chains;
pub mod rag;
pub mod vision;
pub mod cost;
pub mod rate_limit;

use shared::{ChatRequest, ChatResponse, Result};
use crate::router::AiRouter;
use crate::cost::CostTracker;
use crate::rate_limit::RateLimiter;

/// The AI engine — orchestrates all LLM interactions.
pub struct AiEngine {
    router: AiRouter,
    cost_tracker: CostTracker,
    rate_limiter: RateLimiter,
}

impl AiEngine {
    /// Create a new AI engine with all configured providers.
    pub fn new() -> Self {
        Self {
            router: AiRouter::new(),
            cost_tracker: CostTracker::new(),
            rate_limiter: RateLimiter::new(),
        }
    }

    /// Send a chat request to the best available AI provider.
    /// Routes through provider selection, rate limiting, and cost tracking.
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        // Check rate limits
        let provider = request.provider.as_deref().unwrap_or("openai");
        self.rate_limiter.check(provider).await?;

        // Check cost before sending
        self.cost_tracker.check_budget(&request).await?;

        // Route through provider chain
        let response = self.router.route_chat(&request).await?;

        // Track cost
        self.cost_tracker.record_usage(&response).await?;

        Ok(response)
    }

    /// Get the AI router for direct provider access.
    pub fn router(&self) -> &AiRouter { &self.router }

    /// Get the cost tracker.
    pub fn cost_tracker(&self) -> &CostTracker { &self.cost_tracker }
}

impl Default for AiEngine {
    fn default() -> Self { Self::new() }
}
