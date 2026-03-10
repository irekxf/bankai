use tokio::process::Command;

use crate::error::AppError;

pub async fn execute_shell(command: &str) -> Result<String, AppError> {
    let output = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(command)
        .output()
        .await
        .map_err(|error| AppError::Message(error.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    let mut combined = String::new();
    if !stdout.is_empty() {
        combined.push_str(&stdout);
    }
    if !stderr.is_empty() {
        if !combined.is_empty() {
            combined.push_str("\n\n");
        }
        combined.push_str(&stderr);
    }
    if combined.is_empty() {
        combined = format!("Command exited with status {}", output.status);
    }

    Ok(combined)
}
