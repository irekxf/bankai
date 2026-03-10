use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::error::AppError;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "action")]
pub enum FilesystemRequest {
    ReadFile { path: String },
    WriteFile { path: String, content: String },
    ListDir { path: String },
}

pub async fn execute_filesystem(
    workspace_root: &Path,
    request: FilesystemRequest,
) -> Result<String, AppError> {
    match request {
        FilesystemRequest::ReadFile { path } => {
            let canonical = resolve_existing_path(workspace_root, &path)?;
            let contents =
                fs::read_to_string(&canonical).map_err(|error| AppError::Message(error.to_string()))?;
            Ok(contents)
        }
        FilesystemRequest::WriteFile { path, content } => {
            let canonical = resolve_write_path(workspace_root, &path)?;
            if let Some(parent) = canonical.parent() {
                fs::create_dir_all(parent).map_err(|error| AppError::Message(error.to_string()))?;
            }
            fs::write(&canonical, content).map_err(|error| AppError::Message(error.to_string()))?;
            Ok(format!("Wrote file {}", canonical.display()))
        }
        FilesystemRequest::ListDir { path } => {
            let canonical = resolve_existing_path(workspace_root, &path)?;
            let mut entries = Vec::new();
            for entry in fs::read_dir(&canonical).map_err(|error| AppError::Message(error.to_string()))? {
                let entry = entry.map_err(|error| AppError::Message(error.to_string()))?;
                let entry_type =
                    entry.file_type().map_err(|error| AppError::Message(error.to_string()))?;
                let label = if entry_type.is_dir() { "dir" } else { "file" };
                entries.push(format!("{} {}", label, entry.file_name().to_string_lossy()));
            }
            entries.sort();
            Ok(entries.join("\n"))
        }
    }
}

fn resolve_existing_path(workspace_root: &Path, path: &str) -> Result<PathBuf, AppError> {
    let candidate = PathBuf::from(path);
    let resolved = if candidate.is_absolute() {
        candidate
    } else {
        workspace_root.join(candidate)
    };

    let canonical = resolved
        .canonicalize()
        .map_err(|error| AppError::Message(error.to_string()))?;
    ensure_inside_workspace(workspace_root, &canonical)?;
    Ok(canonical)
}

fn resolve_write_path(workspace_root: &Path, path: &str) -> Result<PathBuf, AppError> {
    let candidate = PathBuf::from(path);
    let resolved = if candidate.is_absolute() {
        candidate
    } else {
        workspace_root.join(candidate)
    };

    let normalized_parent = resolved
        .parent()
        .unwrap_or(workspace_root)
        .canonicalize()
        .map_err(|error| AppError::Message(error.to_string()))?;
    ensure_inside_workspace(workspace_root, &normalized_parent)?;

    Ok(resolved)
}

fn ensure_inside_workspace(workspace_root: &Path, candidate: &Path) -> Result<(), AppError> {
    let canonical_root = workspace_root
        .canonicalize()
        .map_err(|error| AppError::Message(error.to_string()))?;

    if candidate.starts_with(&canonical_root) {
        return Ok(());
    }

    Err(AppError::Message(format!(
        "Filesystem access is restricted to the workspace root: {}",
        canonical_root.display()
    )))
}
