use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::time::{sleep, Duration};

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("OpenAI API error: {0}")]
    Api(#[from] async_openai::error::OpenAIError),
    #[error("No response content from API")]
    NoContent,
}

pub struct GeminiClient {
    api_keys: Arc<Mutex<VecDeque<String>>>,
    api_base: String,
    model: String,
}

impl GeminiClient {
    pub fn new(api_keys: Vec<String>, model: Option<String>) -> Self {
        Self {
            api_keys: Arc::new(Mutex::new(VecDeque::from(api_keys))),
            api_base: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            model: model.unwrap_or_else(|| "gemini-2.5-pro".to_string()),
        }
    }

    fn get_next_api_key(&self) -> String {
        let mut keys = self.api_keys.lock().unwrap();
        if let Some(key) = keys.pop_front() {
            keys.push_back(key.clone());
            key
        } else {
            panic!("No API keys available");
        }
    }

    fn create_client(&self, api_key: &str) -> Client<OpenAIConfig> {
        let config = OpenAIConfig::new()
            .with_api_base(&self.api_base)
            .with_api_key(api_key);
        Client::with_config(config)
    }


    pub async fn generate_feature_plan(&self, context: String, prompt: String) -> Result<String, LlmError> {
        let system_prompt_1 = r#"You are a senior software architect with expertise in modern software design patterns and best practices.

Analyze the provided codebase report and create a high-level implementation plan for the requested feature.

Your response should include:
1. Architecture overview - how this feature fits into the existing system
2. Key components/modules that will be affected or created
3. High-level approach and design decisions
4. Potential challenges and considerations
5. Sequential implementation steps at a high level

Focus on architectural clarity and maintainability."#;
        let user_prompt_1 = format!("Codebase Report:\n{context}\n\nFeature Request: {prompt}");

        let high_level_plan = self.query(&self.model, system_prompt_1, &user_prompt_1).await?;

        let system_prompt_2 = r#"You are a senior software engineer creating a detailed implementation guide.

Using the codebase report, feature request, and high-level plan, generate a comprehensive, actionable implementation plan.

Your response MUST include:
1. Specific file paths that need to be created or modified
2. Detailed code snippets for key changes (not pseudocode - actual implementable code)
3. Dependencies or packages that need to be added
4. Database schema changes (if applicable)
5. API endpoint specifications (if applicable)
6. Testing strategy and test cases
7. Step-by-step implementation order with clear explanations
8. Edge cases and error handling considerations

Format your response in clear sections with markdown. Be specific and thorough."#;
        let user_prompt_2 = format!("Codebase Report:\n{context}\n\nOriginal Feature Request: {prompt}\n\nHigh-Level Plan:\n{high_level_plan}\n\nNow provide the detailed implementation plan with specific file paths, code snippets, and clear instructions/explanations.");

        self.query(&self.model, system_prompt_2, &user_prompt_2).await
    }

    pub async fn generate_bug_fix_plan(&self, context: String, prompt: String) -> Result<String, LlmError> {
        let system_prompt_1 = r#"You are a senior software developer specializing in debugging and root cause analysis.

Analyze the provided codebase and bug description to identify the root cause.

Your response should include:
1. Root cause analysis - what is causing the bug?
2. Affected components and files
3. Why the current implementation is failing
4. Impact assessment - what else might be affected?
5. Proposed approach to fix the bug
6. Potential side effects or risks of the fix

Be thorough in your analysis and consider edge cases."#;
        let user_prompt_1 = format!("Codebase Report:\n{context}\n\nBug Description: {prompt}");
        let analysis = self.query(&self.model, system_prompt_1, &user_prompt_1).await?;

        let system_prompt_2 = r#"You are a senior software engineer implementing bug fixes.

Using the codebase report, bug description, and root cause analysis, create a detailed remediation plan.

Your response MUST include:
1. Exact file paths that need to be modified
2. Specific code changes with before/after snippets
3. Why each change fixes the identified issue
4. Additional validation or defensive checks to add
5. Test cases to verify the fix and prevent regression
6. Step-by-step implementation instructions
7. Rollback plan if something goes wrong

Format your response in clear sections with markdown. Provide actual code, not pseudocode."#;
        let user_prompt_2 = format!("Codebase Report:\n{context}\n\nBug Description: {prompt}\n\nRoot Cause Analysis:\n{analysis}\n\nNow provide the detailed fix implementation plan with specific file paths and code changes.");
        self.query(&self.model, system_prompt_2, &user_prompt_2).await
    }

    pub async fn generate_explanation(&self, context: String, prompt: String) -> Result<String, LlmError> {
        let system_prompt_1 = r#"You are a principal engineer with expertise in code architecture and system design.

Analyze the codebase to identify all components relevant to the user's query.

Your response should include:
1. Key files and modules related to the query
2. Main architectural patterns or design approaches used
3. Important concepts or abstractions
4. Data flow and control flow overview
5. Dependencies and relationships between components
6. Any non-obvious implementation details

Focus on providing a complete picture of the relevant system."#;
        let user_prompt_1 = format!("Codebase Report:\n{context}\n\nQuery: {prompt}");
        let key_points = self.query(&self.model, system_prompt_1, &user_prompt_1).await?;

        let system_prompt_2 = r#"You are a principal engineer providing technical documentation and mentorship.

Using the codebase report and your previous analysis, create a comprehensive technical explanation.

Your response MUST include:
1. High-level overview of the system/component in question
2. Detailed walkthrough of how the code works
3. Specific file references with line-by-line explanations where helpful
4. Code snippets highlighting key implementation details
5. Explanation of design decisions and trade-offs
6. Common pitfalls or gotchas developers should know
7. How different components interact with each other
8. Suggestions for where to look for specific functionality

Make your explanation clear, well-structured, and educational. Use markdown formatting with code blocks."#;
        let user_prompt_2 = format!("Codebase Report:\n{context}\n\nOriginal Query: {prompt}\n\nKey Components Identified:\n{key_points}\n\nNow provide a comprehensive technical explanation with code examples and clear structure.");
        self.query(&self.model, system_prompt_2, &user_prompt_2).await
    }

    async fn query(&self, model: &str, system: &str, user: &str) -> Result<String, LlmError> {
        const RETRY_DELAYS: [u64; 3] = [10, 30, 65];

        for (attempt, &delay) in RETRY_DELAYS.iter().enumerate() {
            let api_key = self.get_next_api_key();
            let client = self.create_client(&api_key);

            tracing::debug!("API request attempt {} with delay {}s on failure", attempt + 1, delay);

            let request = match CreateChatCompletionRequestArgs::default()
                .model(model)
                .messages([
                    ChatCompletionRequestSystemMessageArgs::default().content(system).build()?.into(),
                    ChatCompletionRequestUserMessageArgs::default().content(user).build()?.into(),
                ])
                .build() {
                    Ok(req) => req,
                    Err(e) => return Err(LlmError::Api(e)),
                };

            match client.chat().create(request).await {
                Ok(response) => {
                    return response.choices.first()
                        .and_then(|c| c.message.content.as_ref())
                        .cloned()
                        .ok_or(LlmError::NoContent);
                }
                Err(e) => {
                    tracing::warn!("API request failed on attempt {}: {}. Retrying after {}s", attempt + 1, e, delay);
                    sleep(Duration::from_secs(delay)).await;
                    continue;
                }
            }
        }

        let api_key = self.get_next_api_key();
        let client = self.create_client(&api_key);

        tracing::debug!("Final API request attempt (no retry after this)");

        let request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default().content(system).build()?.into(),
                ChatCompletionRequestUserMessageArgs::default().content(user).build()?.into(),
            ])
            .build()?;

        match client.chat().create(request).await {
            Ok(response) => {
                response.choices.first()
                    .and_then(|c| c.message.content.as_ref())
                    .cloned()
                    .ok_or(LlmError::NoContent)
            }
            Err(e) => {
                tracing::error!("API request failed after all retries: {}", e);
                Err(LlmError::Api(e))
            }
        }
    }
}
