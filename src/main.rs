mod config;
mod external;
mod llm;
mod server;

use anyhow::Result;
use clap::Parser;
use config::Config;
use rmcp::ServiceExt;
use server::CodeAgentServer;
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(author, version, about = "AI Code Agent MCP Server")]
struct Cli {
    #[arg(long)]
    codebase_viewer_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let cli = Cli::parse();

    let codebase_viewer_path = cli.codebase_viewer_path
        .or_else(|| std::env::var("CODEBASE_VIEWER_PATH").ok().map(PathBuf::from))
        .expect("CODEBASE_VIEWER_PATH must be set via --codebase-viewer-path flag or environment variable");

    let api_keys = if let Ok(keys_str) = std::env::var("GEMINI_API_KEYS") {
        keys_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>()
    } else if let Ok(single_key) = std::env::var("GEMINI_API_KEY") {
        vec![single_key]
    } else {
        panic!("Either GEMINI_API_KEY or GEMINI_API_KEYS environment variable must be set");
    };

    if api_keys.is_empty() {
        panic!("No valid API keys found in environment variables");
    }

    tracing::info!("Initialized with {} API key(s) for rotation", api_keys.len());

    let gemini_model = std::env::var("GEMINI_MODEL").ok();
    let gemini_client = Arc::new(llm::GeminiClient::new(api_keys, gemini_model));

    let token_char_limit = std::env::var("TOKEN_CHAR_LIMIT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(200_000);

    let config = Config {
        codebase_viewer_path: Arc::new(codebase_viewer_path),
        gemini_client,
        token_char_limit,
    };

    tracing::info!("Starting AI Code Agent MCP Server...");
    let server = CodeAgentServer::new(config)
        .serve(rmcp::transport::stdio())
        .await?;

    tracing::info!("Server initialized and listening on stdio.");
    server.waiting().await?;
    tracing::info!("Server shut down.");

    Ok(())
}
