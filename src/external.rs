use anyhow::{Context, Result};
use std::path::Path;
use tokio::process::Command;

pub async fn generate_codebase_report(
    viewer_path: &Path,
    target_path: &Path,
    token_char_limit: usize,
) -> Result<String> {
    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join(format!("report-{}.md", uuid::Uuid::new_v4()));

    tracing::info!("Generating report for '{}' using '{}'", target_path.display(), viewer_path.display());

    let mut cmd = Command::new(viewer_path);
    cmd.arg("generate")
        .arg("--path")
        .arg(target_path)
        .arg("--output")
        .arg(&temp_file_path)
        .arg("--all");

    let output = cmd.output().await.context("Failed to execute codebase_viewer")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "codebase_viewer failed with status {}: {}",
            output.status,
            stderr
        ));
    }

    let mut report = tokio::fs::read_to_string(&temp_file_path)
        .await
        .context("Failed to read generated report file")?;

    let _ = tokio::fs::remove_file(&temp_file_path).await;

    if report.len() > token_char_limit {
        tracing::warn!(
            "Report length ({}) exceeds character limit ({}). Truncating.",
            report.len(),
            token_char_limit
        );
        if let Some((idx, _)) = report.char_indices().nth(token_char_limit) {
            report.truncate(idx);
            report.push_str("\n\n--- REPORT TRUNCATED DUE TO TOKEN LIMIT ---");
        }
    }

    Ok(report)
}
