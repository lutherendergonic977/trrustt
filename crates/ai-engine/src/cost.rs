// TRRUSTT — Cost Tracker. AI API cost tracking and budget enforcement.
use std::sync::atomic::{AtomicF64, Ordering};
use chrono::Local;
use shared::{ChatRequest, ChatResponse, Result};
use crate::providers::ModelPricing;

const MODEL_PRICES: &[(&str, f64, f64)] = &[
    ("gpt-4o", 2.50, 10.00), ("gpt-4o-mini", 0.15, 0.60), ("gpt-4-turbo", 10.00, 30.00),
    ("claude-3-opus", 15.00, 75.00), ("claude-3-sonnet", 3.00, 15.00), ("claude-3-haiku", 0.25, 1.25),
    ("gemini-1.5-pro", 1.25, 5.00), ("gemini-1.5-flash", 0.075, 0.30), ("deepseek-chat", 0.14, 0.28),
];

pub struct CostTracker {
    daily_spend: AtomicF64, monthly_spend: AtomicF64,
    daily_limit: AtomicF64, monthly_limit: AtomicF64,
    last_reset: std::sync::Mutex<String>,
}

impl CostTracker {
    pub fn new() -> Self {
        Self { daily_spend: AtomicF64::new(0.0), monthly_spend: AtomicF64::new(0.0),
            daily_limit: AtomicF64::new(5.0), monthly_limit: AtomicF64::new(500.0),
            last_reset: std::sync::Mutex::new(Local::now().format("%Y-%m-%d").to_string()) }
    }

    pub async fn check_budget(&self, _request: &ChatRequest) -> Result<()> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let mut last = self.last_reset.lock().unwrap();
        if *last != today { self.daily_spend.store(0.0, Ordering::Relaxed); *last = today; }
        drop(last);
        let daily = self.daily_spend.load(Ordering::Relaxed);
        if daily >= self.daily_limit.load(Ordering::Relaxed) {
            return Err(shared::AppError::AiCostLimitExceeded { reason: format!("Daily limit exceeded: ${:.2}", self.daily_limit.load(Ordering::Relaxed)) });
        }
        let monthly = self.monthly_spend.load(Ordering::Relaxed);
        if monthly >= self.monthly_limit.load(Ordering::Relaxed) {
            return Err(shared::AppError::AiCostLimitExceeded { reason: format!("Monthly limit exceeded: ${:.2}", self.monthly_limit.load(Ordering::Relaxed)) });
        }
        Ok(())
    }

    pub async fn record_usage(&self, response: &ChatResponse) -> Result<()> {
        self.daily_spend.fetch_add(response.cost_usd, Ordering::Relaxed);
        self.monthly_spend.fetch_add(response.cost_usd, Ordering::Relaxed);
        Ok(())
    }

    pub fn get_pricing(model: &str) -> Option<ModelPricing> {
        MODEL_PRICES.iter().find(|(n, _, _)| *n == model).map(|(_, i, o)| ModelPricing { input_cost_per_1m: *i, output_cost_per_1m: *o })
    }

    pub fn calculate_cost(model: &str, input_tokens: i64, output_tokens: i64) -> f64 {
        Self::get_pricing(model).map(|p| p.calculate_cost(input_tokens, output_tokens)).unwrap_or(0.0)
    }

    pub fn daily_spend(&self) -> f64 { self.daily_spend.load(Ordering::Relaxed) }
    pub fn set_daily_limit(&self, l: f64) { self.daily_limit.store(l, Ordering::Relaxed); }
}

impl Default for CostTracker { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_calculate_cost() {
        let c = CostTracker::calculate_cost("gpt-4o-mini", 1000, 500);
        assert!(c > 0.0 && c < 0.01);
    }
}
