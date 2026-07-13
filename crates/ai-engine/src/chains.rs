// TRRUSTT — AI Prompt Chains. Multi-step AI workflow orchestration.
use std::collections::HashMap;
use shared::{ChatRequest, Result};
use config_engine::ConfigEngine;
use crate::router::AiRouter;
use crate::prompt::PromptManager;

/// Executes multi-step AI prompt chains for complex workflows.
/// Each chain step reads its model, temperature, and max_tokens from config.
pub struct ChainExecutor {
    router: AiRouter,
    prompts: PromptManager,
    config: ConfigEngine,
}

impl ChainExecutor {
    /// Create a new chain executor.
    pub fn new(router: AiRouter, prompts: PromptManager, config: ConfigEngine) -> Self {
        Self { router, prompts, config }
    }

    /// Analyze a Power BI data model schema and identify key KPIs, relationships,
    /// and recommended DAX measures.
    #[tracing::instrument(skip(self, schema_json), fields(schema_len = schema_json.len()))]
    pub async fn analyze_schema(&mut self, schema_json: &str) -> Result<String> {
        let mut vars = HashMap::new();
        vars.insert("schema".into(), schema_json.to_string());
        let prompt = self.prompts.render("analyze_schema", &vars).unwrap_or_else(|_|
            format!("Analyze this Power BI data model schema and identify key KPIs, relationships, and recommended DAX measures:\n{}", schema_json)
        );

        let model: String = self.config.get("ai.chains.analyze_schema.model")?;
        let temperature: f64 = self.config.get("ai.chains.analyze_schema.temperature")?;
        let max_tokens: u32 = self.config.get("ai.chains.analyze_schema.max_tokens")?;

        let request = ChatRequest {
            system_prompt: Some("You are a Power BI data modeling expert.".into()),
            user_message: prompt, history: vec![], provider: None,
            model: Some(model), temperature: Some(temperature),
            max_tokens: Some(max_tokens), response_format: None, extra_params: None,
        };
        let response = self.router.route_chat(&request).await?;
        Ok(response.content)
    }

    /// Generate a DAX measure from a natural language description.
    #[tracing::instrument(skip(self, description, schema_context), fields(complexity = %complexity))]
    pub async fn generate_dax(&mut self, description: &str, schema_context: &str, complexity: &str) -> Result<String> {
        let prompt = format!("Generate a DAX measure for: {}\nSchema:\n{}\nComplexity: {}\nReturn ONLY the DAX expression.", description, schema_context, complexity);

        let model: String = self.config.get("ai.chains.generate_dax.model")?;
        let temperature: f64 = self.config.get("ai.chains.generate_dax.temperature")?;
        let max_tokens: u32 = self.config.get("ai.chains.generate_dax.max_tokens")?;

        let request = ChatRequest {
            system_prompt: Some("You are a DAX expert. Generate valid, optimized DAX. Return only code.".into()),
            user_message: prompt, history: vec![], provider: None,
            model: Some(model), temperature: Some(temperature),
            max_tokens: Some(max_tokens), response_format: None, extra_params: None,
        };
        let response = self.router.route_chat(&request).await?;
        Ok(response.content)
    }

    /// Plan a Power BI dashboard layout from a natural language intent.
    #[tracing::instrument(skip(self, intent, schema_summary))]
    pub async fn plan_dashboard(&mut self, intent: &str, schema_summary: &str) -> Result<String> {
        let prompt = format!("Plan a Power BI dashboard layout.\nIntent: {}\nData: {}\nOutput JSON with pages, visuals, positions, and measure bindings.", intent, schema_summary);

        let model: String = self.config.get("ai.chains.plan_dashboard.model")?;
        let temperature: f64 = self.config.get("ai.chains.plan_dashboard.temperature")?;
        let max_tokens: u32 = self.config.get("ai.chains.plan_dashboard.max_tokens")?;

        let request = ChatRequest {
            system_prompt: Some("You are a dashboard design expert. Output valid JSON.".into()),
            user_message: prompt, history: vec![], provider: None,
            model: Some(model), temperature: Some(temperature),
            max_tokens: Some(max_tokens), response_format: None, extra_params: None,
        };
        let response = self.router.route_chat(&request).await?;
        Ok(response.content)
    }
}
