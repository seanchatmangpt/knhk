// knhk-unrdf: Script execution helper
// Execute Node.js scripts for unrdf integration

use crate::error::{UnrdfError, UnrdfResult};
use crate::state::get_state;
use tokio::process::Command;

/// Execute a script using Node.js and unrdf
pub async fn execute_unrdf_script(script_content: &str) -> UnrdfResult<String> {
    let state = get_state()?;

    // Write script to unrdf directory (so relative imports work)
    // Generate unique filename using timestamp (handle clock skew errors gracefully)
    let timestamp_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_else(|_| {
            // Fallback to random number if clock is before epoch (should never happen)
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            std::time::Instant::now().hash(&mut hasher);
            hasher.finish() as u128
        });
    let script_filename = format!("knhk_unrdf_{}.mjs", timestamp_nanos);
    let temp_file = std::path::Path::new(&state.unrdf_path).join(&script_filename);
    std::fs::write(&temp_file, script_content)
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to write script: {}", e)))?;

    // Execute via Node.js with timeout
    let output = Command::new("node")
        .arg(&script_filename)
        .current_dir(&state.unrdf_path)
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|e| UnrdfError::QueryFailed(format!("Failed to execute node: {}", e)))?;

    // Cleanup
    let _ = std::fs::remove_file(&temp_file);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Include exit code in error message
        let exit_code = output.status.code().unwrap_or(-1);
        return Err(UnrdfError::QueryFailed(format!(
            "Script failed (exit code {}): stderr={}, stdout={}",
            exit_code, stderr, stdout
        )));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| UnrdfError::QueryFailed(format!("Invalid output: {}", e)))?;

    // Trim whitespace and check for error messages
    let trimmed = stdout.trim();
    if trimmed.starts_with("ERROR:") || trimmed.contains("Error:") {
        return Err(UnrdfError::QueryFailed(format!(
            "Script reported error: {}",
            trimmed
        )));
    }

    Ok(trimmed.to_string())
}
