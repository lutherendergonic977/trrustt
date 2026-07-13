// TRRUSTT — Prompt Manager. Externalized prompt templates.
use std::collections::HashMap;
use std::path::PathBuf;
use shared::Result;

pub struct PromptManager {
    prompts_dir: PathBuf,
    cache: HashMap<String, String>,
}

impl PromptManager {
    pub fn new(prompts_dir: PathBuf) -> Self { Self { prompts_dir, cache: HashMap::new() } }
    pub fn load(&mut self, name: &str) -> Result<String> {
        if !self.cache.contains_key(name) {
            let path = self.prompts_dir.join(format!("{}.md", name));
            let content = std::fs::read_to_string(&path)
                .map_err(|e| shared::AppError::file_not_found(path.display().to_string()))?;
            self.cache.insert(name.to_string(), content);
        }
        Ok(self.cache.get(name).cloned().unwrap_or_default())
    }
    pub fn render(&mut self, template: &str, vars: &HashMap<String, String>) -> Result<String> {
        let tmpl = self.load(template)?;
        let mut result = tmpl;
        for (k, v) in vars {
            result = result.replace(&format!("{{{{{}}}}}", k), v);
        }
        Ok(result)
    }
    pub fn list(&self) -> Vec<String> { self.cache.keys().cloned().collect() }
    pub fn reload(&mut self) { self.cache.clear(); }
}

impl Default for PromptManager {
    fn default() -> Self { Self::new(PathBuf::from(".trrustt/prompts")) }
}
