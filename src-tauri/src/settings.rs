use std::{fs, path::PathBuf};

use keyring::Entry;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::{error::AppError, oauth::get_oauth_bearer_token};

const OPENAI_SERVICE: &str = "bankai.openai";
const OPENAI_ACCOUNT: &str = "default";
const PROVIDER_FILE: &str = "provider.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    pub provider: String,
    pub display_name: String,
    pub base_url: String,
    pub model: String,
    pub api_key_status: ApiKeyStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeyStatus {
    Missing,
    Configured,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProviderConfigInput {
    pub provider: String,
    pub display_name: String,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            display_name: "ChatGPT / OpenAI".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4.1".to_string(),
            api_key_status: ApiKeyStatus::Missing,
        }
    }
}

pub fn load_provider_config(app: &AppHandle) -> Result<ProviderConfig, AppError> {
    let mut config = read_provider_file(app)?.unwrap_or_default();
    config.api_key_status = read_api_key_status()?;
    Ok(config)
}

pub fn load_openai_api_key() -> Result<String, AppError> {
    let entry = Entry::new(OPENAI_SERVICE, OPENAI_ACCOUNT)
        .map_err(|error| AppError::Message(error.to_string()))?;

    let value = entry
        .get_password()
        .map_err(|error| AppError::Message(error.to_string()))?;
    let trimmed = value.trim().to_string();

    if trimmed.is_empty() {
        return Err(AppError::Message(
            "OpenAI API key is empty. Save it in Provider settings first.".to_string(),
        ));
    }

    Ok(trimmed)
}

pub async fn load_openai_bearer_token() -> Result<String, AppError> {
    if let Ok(api_key) = load_openai_api_key() {
        if !api_key.trim().is_empty() {
            return Ok(api_key);
        }
    }

    if let Some(token) = get_oauth_bearer_token().await? {
        return Ok(token);
    }

    Err(AppError::Message(
        "No OpenAI API key or OAuth session is configured.".to_string(),
    ))
}

pub fn save_provider_config(
    app: &AppHandle,
    input: SaveProviderConfigInput,
) -> Result<ProviderConfig, AppError> {
    let config = ProviderConfig {
        provider: input.provider,
        display_name: input.display_name,
        base_url: input.base_url,
        model: input.model,
        api_key_status: ApiKeyStatus::Missing,
    };

    write_provider_file(app, &config)?;

    if let Some(api_key) = input.api_key {
        let trimmed = api_key.trim();
        if trimmed.is_empty() {
            clear_api_key()?;
        } else {
            save_api_key(trimmed)?;
        }
    }

    load_provider_config(app)
}

fn provider_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|error| AppError::Message(error.to_string()))?;
    fs::create_dir_all(&dir).map_err(|error| AppError::Message(error.to_string()))?;
    Ok(dir.join(PROVIDER_FILE))
}

fn read_provider_file(app: &AppHandle) -> Result<Option<ProviderConfig>, AppError> {
    let path = provider_path(app)?;
    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(path).map_err(|error| AppError::Message(error.to_string()))?;
    let mut config: ProviderConfig =
        serde_json::from_str(&contents).map_err(|error| AppError::Message(error.to_string()))?;
    config.api_key_status = ApiKeyStatus::Missing;
    Ok(Some(config))
}

fn write_provider_file(app: &AppHandle, config: &ProviderConfig) -> Result<(), AppError> {
    let path = provider_path(app)?;
    let payload =
        serde_json::to_string_pretty(config).map_err(|error| AppError::Message(error.to_string()))?;
    fs::write(path, payload).map_err(|error| AppError::Message(error.to_string()))?;
    Ok(())
}

fn read_api_key_status() -> Result<ApiKeyStatus, AppError> {
    let entry = Entry::new(OPENAI_SERVICE, OPENAI_ACCOUNT)
        .map_err(|error| AppError::Message(error.to_string()))?;

    match entry.get_password() {
        Ok(value) if !value.trim().is_empty() => Ok(ApiKeyStatus::Configured),
        Ok(_) => Ok(ApiKeyStatus::Missing),
        Err(keyring::Error::NoEntry) => Ok(ApiKeyStatus::Missing),
        Err(error) => Err(AppError::Message(error.to_string())),
    }
}

fn save_api_key(api_key: &str) -> Result<(), AppError> {
    let entry = Entry::new(OPENAI_SERVICE, OPENAI_ACCOUNT)
        .map_err(|error| AppError::Message(error.to_string()))?;
    entry
        .set_password(api_key)
        .map_err(|error| AppError::Message(error.to_string()))?;
    Ok(())
}

fn clear_api_key() -> Result<(), AppError> {
    let entry = Entry::new(OPENAI_SERVICE, OPENAI_ACCOUNT)
        .map_err(|error| AppError::Message(error.to_string()))?;

    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(AppError::Message(error.to_string())),
    }
}
