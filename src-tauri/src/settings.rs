use std::{fs, path::PathBuf};

use keyring::Entry;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::{
    error::AppError,
    oauth::{get_oauth_bearer_token, get_oauth_status, OAuthStatus},
};

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
    #[serde(default)]
    pub preferred_auth: PreferredAuth,
    pub api_key_status: ApiKeyStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ApiKeyStatus {
    Missing,
    Configured,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActiveAuth {
    ApiKey,
    Oauth,
    None,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatus {
    pub provider: String,
    pub display_name: String,
    pub base_url: String,
    pub model: String,
    pub preferred_auth: PreferredAuth,
    pub api_key_status: ApiKeyStatus,
    pub oauth_logged_in: bool,
    pub oauth_auth_mode: Option<String>,
    pub oauth_account_id: Option<String>,
    pub oauth_expires_at: Option<u64>,
    pub active_auth: ActiveAuth,
    pub auth_ready: bool,
    pub can_load_models: bool,
    pub can_send_messages: bool,
    pub auth_message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PreferredAuth {
    Auto,
    ApiKey,
    Oauth,
}

impl Default for PreferredAuth {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveProviderConfigInput {
    pub provider: String,
    pub display_name: String,
    pub base_url: String,
    pub model: String,
    #[serde(default)]
    pub preferred_auth: Option<PreferredAuth>,
    pub api_key: Option<String>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            display_name: "ChatGPT / OpenAI".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4.1".to_string(),
            preferred_auth: PreferredAuth::Auto,
            api_key_status: ApiKeyStatus::Missing,
        }
    }
}

pub fn load_provider_config(app: &AppHandle) -> Result<ProviderConfig, AppError> {
    let mut config = read_provider_file(app)?.unwrap_or_default();
    config.api_key_status = read_api_key_status()?;
    Ok(config)
}

pub async fn load_provider_status(app: &AppHandle) -> Result<ProviderStatus, AppError> {
    let config = load_provider_config(app)?;
    let oauth = get_oauth_status().await?;
    Ok(build_provider_status(config, oauth))
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

pub async fn load_openai_bearer_token(config: &ProviderConfig) -> Result<String, AppError> {
    match config.preferred_auth {
        PreferredAuth::ApiKey => load_openai_api_key(),
        PreferredAuth::Oauth => get_oauth_bearer_token()
            .await?
            .ok_or_else(|| AppError::Message("OAuth session is not configured.".to_string())),
        PreferredAuth::Auto => {
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
    }
}

pub fn save_provider_config(
    app: &AppHandle,
    input: SaveProviderConfigInput,
) -> Result<ProviderConfig, AppError> {
    let existing = read_provider_file(app)?.unwrap_or_default();
    let config = ProviderConfig {
        provider: input.provider,
        display_name: input.display_name,
        base_url: input.base_url,
        model: input.model,
        preferred_auth: input.preferred_auth.unwrap_or(existing.preferred_auth),
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

pub fn set_preferred_auth(
    app: &AppHandle,
    preferred_auth: PreferredAuth,
) -> Result<ProviderConfig, AppError> {
    let mut config = read_provider_file(app)?.unwrap_or_default();
    config.preferred_auth = preferred_auth;
    write_provider_file(app, &config)?;
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

    let contents =
        fs::read_to_string(path).map_err(|error| AppError::Message(error.to_string()))?;
    let mut config: ProviderConfig =
        serde_json::from_str(&contents).map_err(|error| AppError::Message(error.to_string()))?;
    config.api_key_status = ApiKeyStatus::Missing;
    Ok(Some(config))
}

fn write_provider_file(app: &AppHandle, config: &ProviderConfig) -> Result<(), AppError> {
    let path = provider_path(app)?;
    let payload = serde_json::to_string_pretty(config)
        .map_err(|error| AppError::Message(error.to_string()))?;
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

fn build_provider_status(config: ProviderConfig, oauth: OAuthStatus) -> ProviderStatus {
    let active_auth = resolve_active_auth(config.preferred_auth, config.api_key_status, oauth.logged_in);
    let auth_ready = active_auth != ActiveAuth::None;

    ProviderStatus {
        provider: config.provider,
        display_name: config.display_name,
        base_url: config.base_url,
        model: config.model,
        preferred_auth: config.preferred_auth,
        api_key_status: config.api_key_status,
        oauth_logged_in: oauth.logged_in,
        oauth_auth_mode: oauth.auth_mode,
        oauth_account_id: oauth.account_id,
        oauth_expires_at: oauth.expires_at,
        active_auth,
        auth_ready,
        can_load_models: auth_ready,
        can_send_messages: auth_ready,
        auth_message: build_auth_message(config.preferred_auth, active_auth),
    }
}

fn resolve_active_auth(
    preferred_auth: PreferredAuth,
    api_key_status: ApiKeyStatus,
    oauth_logged_in: bool,
) -> ActiveAuth {
    match preferred_auth {
        PreferredAuth::ApiKey => {
            if api_key_status == ApiKeyStatus::Configured {
                ActiveAuth::ApiKey
            } else {
                ActiveAuth::None
            }
        }
        PreferredAuth::Oauth => {
            if oauth_logged_in {
                ActiveAuth::Oauth
            } else {
                ActiveAuth::None
            }
        }
        PreferredAuth::Auto => {
            if api_key_status == ApiKeyStatus::Configured {
                ActiveAuth::ApiKey
            } else if oauth_logged_in {
                ActiveAuth::Oauth
            } else {
                ActiveAuth::None
            }
        }
    }
}

fn build_auth_message(preferred_auth: PreferredAuth, active_auth: ActiveAuth) -> String {
    match active_auth {
        ActiveAuth::ApiKey => {
            "Requests will use the saved API key from the system keyring.".to_string()
        }
        ActiveAuth::Oauth => {
            "Requests will use the current OAuth session and refresh it when possible.".to_string()
        }
        ActiveAuth::None => match preferred_auth {
            PreferredAuth::ApiKey => {
                "Save an API key to enable model listing and message sending.".to_string()
            }
            PreferredAuth::Oauth => {
                "Connect OAuth to enable model listing and message sending.".to_string()
            }
            PreferredAuth::Auto => "Choose either OAuth or API key. Until one is configured, the provider cannot load models or send requests.".to_string(),
        },
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
