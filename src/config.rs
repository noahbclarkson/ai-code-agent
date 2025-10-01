use std::path::PathBuf;
use std::sync::Arc;
use crate::llm::GeminiClient;

#[derive(Clone)]
pub struct Config {
    pub codebase_viewer_path: Arc<PathBuf>,
    pub gemini_client: Arc<GeminiClient>,
    pub token_char_limit: usize,
}
