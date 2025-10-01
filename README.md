# AI Code Agent MCP Server

An intelligent Model Context Protocol (MCP) server that leverages Google's Gemini 2.5 Pro and codebase analysis to provide AI-powered feature planning, bug fixing, and code explanation capabilities.

## Features

- **Feature Implementation Planning**: Generate comprehensive, two-step implementation plans with architectural guidance, file references, and code snippets
- **Bug Fix Planning**: Perform root cause analysis and create detailed remediation plans with step-by-step instructions
- **Code Explanation**: Get in-depth technical explanations of codebase components, architecture patterns, and data flows
- **Smart Token Management**: Configurable token limits with automatic truncation to stay within API constraints
- **Docker Ready**: Multi-stage Dockerfile for easy containerization and deployment

## Architecture

The server integrates two powerful tools:

1. **codebase_viewer**: Generates comprehensive codebase reports
2. **Gemini 2.5 Pro**: Performs two-step AI analysis for deeper insights

Each tool uses a two-phase prompting strategy:

- **Phase 1**: High-level analysis and planning
- **Phase 2**: Detailed implementation with specific file references and code

## Prerequisites

- Rust 1.90+ (for local development)
- Docker (for containerized deployment)
- Google Gemini API key
- `codebase_viewer` binary ([installation instructions](https://github.com/example/codebase_viewer))

## Installation

### Local Development

1. Clone the repository:

    ```bash
    git clone <repository-url>
    cd ai_code_agent
    ```

2. Set up environment variables:

    ```bash
    cp .env.example .env
    # Edit .env with your configuration
    ```

3. Build the project:

    ```bash
    cargo build --release
    ```

4. Run the server:

    ```bash
    # Set CODEBASE_VIEWER_PATH in .env, or pass as argument:
    ./target/release/ai_code_agent --codebase-viewer-path /path/to/codebase_viewer

    # Or just run if CODEBASE_VIEWER_PATH is set in .env:
    ./target/release/ai_code_agent
    ```

5. The AI provides full paths directly in tool calls:

    ```json
    {
    "directory": "C:/Users/yourname/projects/myapp",
    "feature_prompt": "Add authentication"
    }
    ```

### Docker Deployment with Docker Compose

1. Ensure `codebase_viewer` source is available in `./codebase_viewer` directory

2. Create a `.env` file from the template:

    ```bash
    cp .env.docker .env
    # Edit .env with your configuration
    ```

3. Update `.env` with your settings:

    ```env
    GEMINI_API_KEYS=key1,key2,key3
    CODEBASE_VIEWER_PATH=C:\path\to\codebase_viewer.exe
    ```

4. Run with Docker Compose:

    ```bash
    docker-compose up
    ```

**Note:** Docker uses the codebase_viewer built into the image. The AI provides full paths to analyze in tool calls.

### Docker Deployment (Manual)

1. Ensure `codebase_viewer` source is available in `./codebase_viewer` directory

2. Build the Docker image:

    ```bash
    docker build -t ai-code-agent .
    ```

3. Run the container:

    ```bash
    docker run -i --rm \
    -e GEMINI_API_KEY="your_api_key_here" \
    -e TOKEN_CHAR_LIMIT=200000 \
    ai-code-agent
    ```

**Note:** Docker container has codebase_viewer built-in. The AI provides full paths to analyze in tool calls.

## Configuration

All configuration is done via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `GEMINI_API_KEY` | *Required* | Single Google Gemini API key (use this OR `GEMINI_API_KEYS`) |
| `GEMINI_API_KEYS` | - | Multiple API keys (comma-separated) for rotation to avoid rate limits |
| `GEMINI_MODEL` | `gemini-2.5-pro` | Gemini model to use |
| `TOKEN_CHAR_LIMIT` | `200000` | Character limit for codebase reports (~50k tokens) |
| `CODEBASE_VIEWER_PATH` | - | Path to codebase_viewer executable (can also use `--codebase-viewer-path` flag) |

### API Key Rotation

To avoid rate limits, you can provide multiple API keys:

```env
GEMINI_API_KEYS=key1,key2,key3
```

The server will:

- Use keys in rotation (round-robin)
- Automatically retry with exponential backoff (10s, 30s, 65s) on rate limit errors
- Switch to the next key on each request for load distribution

### Command-line Arguments

- `--codebase-viewer-path`: Path to the codebase_viewer executable (required)

## MCP Tools

The server exposes three MCP tools via stdio transport:

### 1. `plan_feature`

Generates comprehensive feature implementation plans.

**Parameters:**

- `directory` (string): **Full absolute path** to the codebase directory (e.g., `/workspace/myapp` or `C:/projects/myapp`). Must NOT be a relative path.
- `feature_prompt` (string): Description of the feature to implement

**Best Practices:**

- For large projects, split by concern (e.g., separate frontend/backend calls)
- Be specific in feature descriptions
- Include acceptance criteria and edge cases

**Example:**

```json
{
  "directory": "C:/Users/yourname/projects/myapp",
  "feature_prompt": "Add user authentication with JWT tokens, including login/logout endpoints and middleware"
}
```

### 2. `plan_bug_fix`

Analyzes bugs and creates detailed fix implementation plans.

**Parameters:**

- `directory` (string): **Full absolute path** to the codebase directory. Must NOT be a relative path.
- `bug_description` (string): Detailed bug description with error messages/stack traces

**Best Practices:**

- Include error messages, stack traces, or reproduction steps
- Narrow scope to relevant subsystem (e.g., just the authentication module)
- Specify expected vs actual behavior

**Example:**

```json
{
  "directory": "C:/Users/yourname/projects/api",
  "bug_description": "API returns 500 error when user tries to update profile. Stack trace shows null pointer in UserController.update() at line 45"
}
```

### 3. `explain_code`

Provides detailed technical explanations of codebase components.

**Parameters:**

- `directory` (string): **Full absolute path** to the codebase directory. Must NOT be a relative path.
- `explanation_query` (string): What you want explained

**Best Practices:**

- Target specific subsystems for large projects
- Ask about architecture, patterns, or specific functionality
- Use for onboarding, documentation, or understanding complex logic

**Example:**

```json
{
  "directory": "C:/Users/yourname/projects/src",
  "explanation_query": "Explain how the authentication system works, including session management and token refresh"
}
```

**Important:** All tools require **full absolute paths** provided by the AI (e.g., `C:/Users/yourname/projects/myapp` not `./myapp`).

## Token Limits & Large Codebases

The default token limit is 200,000 characters (~50,000 tokens), suitable for small to medium codebases. For large projects:

1. **Split by Layer**: Analyze frontend and backend separately
2. **Split by Module**: Focus on specific modules or subsystems
3. **Split by Concern**: Separate authentication, data access, UI, etc.
4. **Adjust Limit**: Increase `TOKEN_CHAR_LIMIT` if you have higher API quotas

## Integration with Claude Code

Add to your Claude Code MCP configuration:

```json
{
  "mcpServers": {
    "ai-code-agent": {
      "command": "/path/to/ai_code_agent",
      "args": ["--codebase-viewer-path", "/path/to/codebase_viewer"],
      "env": {
        "GEMINI_API_KEY": "your_api_key_here",
        "TOKEN_CHAR_LIMIT": "200000"
      }
    }
  }
}
```

## Development

### Project Structure

```txt
ai_code_agent/
├── src/
│   ├── main.rs           # CLI entrypoint and server initialization
│   ├── config.rs         # Configuration management
│   ├── server.rs         # MCP tools implementation
│   ├── external.rs       # codebase_viewer integration
│   └── llm.rs           # Gemini API client with prompting logic
├── Dockerfile           # Multi-stage containerization
├── Cargo.toml          # Dependencies and metadata
└── README.md           # This file
```

### Building from Source

```bash
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Checking Code Quality

```bash
cargo clippy
cargo fmt --check
```

## Troubleshooting

### "codebase_viewer failed with status..."

- Ensure `codebase_viewer` is in your PATH or specify full path
- Check that the target directory exists and is readable
- Verify `codebase_viewer` has execute permissions

### "GEMINI_API_KEY environment variable not set"

- Create a `.env` file with your API key
- Or set the environment variable directly: `export GEMINI_API_KEY=your_key`

### "Report truncated due to token limit"

- This is normal for large codebases
- Consider splitting your request by module/layer
- Or increase `TOKEN_CHAR_LIMIT` if your API quota allows

### API Rate Limits

- Gemini has rate limits; space out requests if needed
- Consider using exponential backoff for retries
- Check your API quota at the Google Cloud Console

## License

[Your License Here]

## Contributing

Contributions welcome! Please open an issue or submit a PR.
