use crate::config::Config;
use crate::external;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{ServerCapabilities, ServerInfo};
use rmcp::{tool, tool_handler, tool_router, ServerHandler};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, JsonSchema)]
pub struct FeatureParams {
    #[schemars(description = "Full absolute path to the codebase directory (e.g., /workspace/myapp or C:/projects/myapp). Must NOT be a relative path.")]
    pub directory: String,
    pub feature_prompt: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct BugFixParams {
    #[schemars(description = "Full absolute path to the codebase directory (e.g., /workspace/myapp or C:/projects/myapp). Must NOT be a relative path.")]
    pub directory: String,
    pub bug_description: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ExplanationParams {
    #[schemars(description = "Full absolute path to the codebase directory (e.g., /workspace/myapp or C:/projects/myapp). Must NOT be a relative path.")]
    pub directory: String,
    pub explanation_query: String,
}

#[derive(Clone)]
pub struct CodeAgentServer {
    config: Config,
    tool_router: rmcp::handler::server::router::tool::ToolRouter<Self>,
}

#[tool_router]
impl CodeAgentServer {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Generates a comprehensive, two-step feature implementation plan using Gemini 2.5 Pro. Analyzes codebase structure, creates high-level architecture plan, then produces detailed implementation guide with file references and code snippets. For large projects, split requests by concern (e.g., separate frontend/backend or by module) to stay within 200k token limit. Best for small-medium codebases or focused subdirectories.")]
    async fn plan_feature(&self, params: Parameters<FeatureParams>) -> Result<String, String> {
        tracing::info!("Received 'plan_feature' request for directory: {}", params.0.directory);

        let report = match external::generate_codebase_report(
            &self.config.codebase_viewer_path,
            &PathBuf::from(params.0.directory),
            self.config.token_char_limit,
        ).await {
            Ok(r) => r,
            Err(e) => return Err(format!("Failed to generate codebase report: {e}")),
        };

        match self.config.gemini_client.generate_feature_plan(report, params.0.feature_prompt).await {
            Ok(plan) => Ok(plan),
            Err(e) => Err(format!("Failed to generate feature plan from Gemini: {e}")),
        }
    }

    #[tool(description = "Analyzes bugs and generates detailed fix implementation plans using Gemini 2.5 Pro. Performs root cause analysis, identifies affected files, and provides step-by-step remediation with code examples. For large projects, narrow scope to relevant subsystem (e.g., just authentication module or API layer) to stay within 200k token limit. Include error messages, stack traces, or reproduction steps in bug_description for best results.")]
    async fn plan_bug_fix(&self, params: Parameters<BugFixParams>) -> Result<String, String> {
        tracing::info!("Received 'plan_bug_fix' request for directory: {}", params.0.directory);
        let report = match external::generate_codebase_report(
            &self.config.codebase_viewer_path,
            &PathBuf::from(params.0.directory),
            self.config.token_char_limit,
        ).await {
            Ok(r) => r,
            Err(e) => return Err(format!("Failed to generate codebase report: {e}")),
        };

        match self.config.gemini_client.generate_bug_fix_plan(report, params.0.bug_description).await {
            Ok(plan) => Ok(plan),
            Err(e) => Err(format!("Failed to generate bug fix plan from Gemini: {e}")),
        }
    }

    #[tool(description = "Provides detailed technical explanations of codebase components using Gemini 2.5 Pro. Identifies key files, explains architecture patterns, data flow, and inter-component relationships with code examples. For large projects, target specific subsystems (e.g., 'explain the authentication system' vs 'explain the entire backend') to stay within 200k token limit. Best for onboarding, documentation, or understanding complex logic.")]
    async fn explain_code(&self, params: Parameters<ExplanationParams>) -> Result<String, String> {
        tracing::info!("Received 'explain_code' request for directory: {}", params.0.directory);
        let report = match external::generate_codebase_report(
            &self.config.codebase_viewer_path,
            &PathBuf::from(params.0.directory),
            self.config.token_char_limit,
        ).await {
            Ok(r) => r,
            Err(e) => return Err(format!("Failed to generate codebase report: {e}")),
        };

        match self.config.gemini_client.generate_explanation(report, params.0.explanation_query).await {
            Ok(explanation) => Ok(explanation),
            Err(e) => Err(format!("Failed to generate explanation from Gemini: {e}")),
        }
    }
}

#[tool_handler]
impl ServerHandler for CodeAgentServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
