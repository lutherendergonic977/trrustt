// TRRUSTT — RAG Pipeline. Schema-aware retrieval for AI queries.
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use shared::Result;

#[derive(Debug, Clone)]
pub struct RagDocument { pub content: String, pub metadata: HashMap<String, String> }
pub struct RagSearchResult { pub documents: Vec<RagDocument> }

pub struct RagIndex { documents: Arc<RwLock<Vec<RagDocument>>> }

impl RagIndex {
    pub fn new() -> Self { Self { documents: Arc::new(RwLock::new(Vec::new())) } }
    pub async fn index(&self, content: &str, metadata: HashMap<String, String>) -> Result<()> {
        self.documents.write().push(RagDocument { content: content.to_string(), metadata });
        Ok(())
    }
    pub async fn search(&self, query: &str, top_k: usize) -> Result<RagSearchResult> {
        let docs = self.documents.read();
        let ql = query.to_lowercase();
        let mut scored: Vec<(usize, &RagDocument)> = docs.iter().enumerate()
            .map(|(i, d)| { let c = d.content.to_lowercase(); (ql.split_whitespace().filter(|w| c.contains(w)).count(), d) })
            .filter(|(s, _)| *s > 0).collect();
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        let top: Vec<RagDocument> = scored.into_iter().take(top_k).map(|(_, d)| d.clone()).collect();
        Ok(RagSearchResult { documents: top })
    }
    pub fn document_count(&self) -> usize { self.documents.read().len() }
    pub fn clear(&self) { self.documents.write().clear(); }
}

impl Default for RagIndex { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_rag_search() {
        let idx = RagIndex::new();
        idx.index("Sales table: revenue, product_id, date", HashMap::new()).await.unwrap();
        idx.index("Products table: name, category, price", HashMap::new()).await.unwrap();
        let r = idx.search("sales revenue", 3).await.unwrap();
        assert!(!r.documents.is_empty());
    }
}
