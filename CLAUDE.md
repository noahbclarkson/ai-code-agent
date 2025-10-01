# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

AI Code Agent is an MCP (Model Context Protocol) server that combines `codebase_viewer` analysis with Gemini 2.5 Pro to provide three main capabilities: feature planning, bug fix planning, and code explanation. The server operates via stdio transport and is designed for integration with Claude Code and other MCP clients.

## Development Commands

### Build and Run

```bash
# Development build and run (loads .env automatically)
cargo run

# Release build
cargo build --release

# Run release binary
./target/release/ai_code_agent

# Check compilation without building
cargo check

# Format code
cargo fmt

# Lint with Clippy
cargo clippy
```

### Environment Setup

The `.env` file must contain:

- `GEMINI_API_KEYS` or `GEMINI_API_KEY` (required)
- `CODEBASE_VIEWER_PATH` (required) - **Use forward slashes even on Windows** (e.g., `C:/path/to/file.exe`)
- `GEMINI_MODEL` (optional, defaults to `gemini-2.5-pro`)
- `TOKEN_CHAR_LIMIT` (optional, defaults to 200000)

**Critical:** Windows paths in `.env` files must use forward slashes, not backslashes. Backslashes are treated as escape characters.

## Architecture

### Core Flow

1. **Main** (`main.rs`): CLI entry, loads env vars, initializes server
2. **Server** (`server.rs`): Defines 3 MCP tools using `#[tool]` macro from rmcp
3. **External** (`external.rs`): Executes `codebase_viewer` CLI, generates reports, truncates to token limit
4. **LLM** (`llm.rs`): Two-phase Gemini API querying with retry logic and key rotation
5. **Config** (`config.rs`): Shared state container

### MCP Tool Pattern

Each tool follows this pattern:

1. Receive parameters (directory path + query)
2. Call `external::generate_codebase_report()` to get codebase analysis
3. Pass report + query to appropriate `GeminiClient` method
4. Return Gemini's response via MCP

### Two-Phase LLM Prompting

All three tools use a two-step approach:

- **Phase 1**: High-level analysis (architecture, root causes, key files)
- **Phase 2**: Detailed implementation (code snippets, file paths, step-by-step)

This improves output quality by giving Gemini context to build upon.

### API Key Rotation & Retry Logic

`GeminiClient` implements:

- **Round-robin key rotation**: Keys stored in `VecDeque`, rotated on each request
- **Automatic retry**: 4 attempts total with exponential backoff (10s, 30s, 65s)
- **Retry on any failure**: Gemini doesn't always return proper rate limit status codes, so we retry all failures

### Token Management

`external.rs` truncates codebase reports at the configured character limit (~4 chars per token). Reports are generated in temp files and cleaned up immediately after reading.

## Critical Implementation Details

### MCP Tool Requirements

- All tools **require absolute paths** (enforced via `schemars` descriptions)
- Paths are passed directly to `codebase_viewer` without modification
- No relative path resolution is performed

### Environment Variable Handling

- Uses `dotenvy` to load `.env` files automatically on startup
- `CODEBASE_VIEWER_PATH` is checked both as env var and CLI flag (`--codebase-viewer-path`)
- API keys can be single (`GEMINI_API_KEY`) or comma-separated (`GEMINI_API_KEYS`)

### Error Handling Strategy

- `anyhow` for external.rs (context-rich error chains)
- `thiserror` for llm.rs (typed error enums)
- Tool functions return `Result<String, String>` per MCP convention
- Errors are logged via `tracing` before being returned to client

### Logging

Uses `tracing` with env-based filtering. Default level is `info`. Set `RUST_LOG` env var to change:

```bash
RUST_LOG=debug cargo run
```

## MCP Integration

To connect this server to Claude Code, add to `mcp_settings.json`:

```json
{
  "mcpServers": {
    "ai-code-agent": {
      "command": "C:/path/to/ai_code_agent.exe",
      "env": {
        "GEMINI_API_KEYS": "key1,key2",
        "CODEBASE_VIEWER_PATH": "C:/path/to/codebase_viewer.exe",
        "TOKEN_CHAR_LIMIT": "200000"
      }
    }
  }
}
```

The server runs in stdio mode - Claude Code starts/stops it as needed.

## Common Pitfalls

1. **Backslashes in .env**: Will cause path parsing to fail. Always use forward slashes.
2. **Relative paths in tool calls**: Will fail. Clients must provide absolute paths.
3. **Large codebases**: Will be truncated. Advise users to analyze subdirectories separately.
4. **API rate limits**: Handled automatically with retries, but may still exhaust quota with heavy usage.

## Dependencies

Key crates and their roles:

- `rmcp`: MCP protocol server implementation with macros
- `async-openai`: HTTP client for Gemini API (configured with custom base URL)
- `tokio`: Async runtime for process spawning and network I/O
- `clap`: CLI argument parsing with derive macros
- `schemars`: JSON schema generation for MCP tool parameters
