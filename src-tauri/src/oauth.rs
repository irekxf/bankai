use std::{
    io::{Read, Write},
    net::TcpListener,
    process::Command,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use keyring::Entry;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::AppError;

const OAUTH_SERVICE: &str = "bankai.openai.oauth";
const OAUTH_ACCOUNT: &str = "default";
const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const AUTHORIZE_URL: &str = "https://auth.openai.com/oauth/authorize";
const TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const CALLBACK_PORT: u16 = 1455;
const CALLBACK_PATH: &str = "/auth/callback";
const REDIRECT_URI: &str = "http://localhost:1455/auth/callback";
const OAUTH_SCOPE: &str = "openid profile email offline_access";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthSession {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: Option<String>,
    pub account_id: Option<String>,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthStatus {
    pub logged_in: bool,
    pub auth_mode: Option<String>,
    pub account_id: Option<String>,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    id_token: Option<String>,
    expires_in: u64,
}

pub async fn start_oauth_login() -> Result<OAuthStatus, AppError> {
    let verifier = generate_random_string(96);
    let challenge = build_code_challenge(&verifier);
    let state = generate_random_string(48);
    let auth_url = build_authorize_url(&challenge, &state)?;

    let callback_handle = tokio::task::spawn_blocking(move || wait_for_callback(&state));
    open_browser(&auth_url)?;
    let callback = callback_handle
        .await
        .map_err(|error| AppError::Message(error.to_string()))??;

    let session = exchange_code(&callback.code, &verifier).await?;
    save_oauth_session(&session)?;
    Ok(status_from_session(&session))
}

pub async fn get_oauth_status() -> Result<OAuthStatus, AppError> {
    match ensure_fresh_session().await? {
        Some(session) => Ok(status_from_session(&session)),
        None => Ok(OAuthStatus {
            logged_in: false,
            auth_mode: Some("chatgpt-oauth".to_string()),
            account_id: None,
            expires_at: None,
        }),
    }
}

pub async fn get_oauth_bearer_token() -> Result<Option<String>, AppError> {
    let Some(session) = ensure_fresh_session().await? else {
        return Ok(None);
    };

    Ok(Some(session.access_token))
}

fn build_authorize_url(challenge: &str, state: &str) -> Result<String, AppError> {
    let mut url =
        Url::parse(AUTHORIZE_URL).map_err(|error| AppError::Message(error.to_string()))?;
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", CLIENT_ID)
        .append_pair("redirect_uri", REDIRECT_URI)
        .append_pair("scope", OAUTH_SCOPE)
        .append_pair("code_challenge", challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("id_token_add_organizations", "true")
        .append_pair("codex_cli_simplified_flow", "true")
        .append_pair("originator", "bankai_tauri")
        .append_pair("state", state);
    Ok(url.into())
}

fn open_browser(url: &str) -> Result<(), AppError> {
    #[cfg(target_os = "windows")]
    {
        Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", url])
            .spawn()
            .map_err(|error| AppError::Message(error.to_string()))?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|error| AppError::Message(error.to_string()))?;
        return Ok(());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|error| AppError::Message(error.to_string()))?;
        Ok(())
    }
}

async fn exchange_code(code: &str, verifier: &str) -> Result<OAuthSession, AppError> {
    let client = reqwest::Client::new();
    let response = client
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", CLIENT_ID),
            ("code", code),
            ("code_verifier", verifier),
            ("redirect_uri", REDIRECT_URI),
        ])
        .send()
        .await
        .map_err(|error| AppError::Message(error.to_string()))?;

    if !response.status().is_success() {
        return Err(AppError::Message(format!(
            "OAuth token exchange failed with status {}",
            response.status()
        )));
    }

    let token: TokenResponse = response
        .json()
        .await
        .map_err(|error| AppError::Message(error.to_string()))?;
    Ok(build_session(token, None))
}

async fn refresh_session(session: &OAuthSession) -> Result<OAuthSession, AppError> {
    let client = reqwest::Client::new();
    let response = client
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", CLIENT_ID),
            ("refresh_token", session.refresh_token.as_str()),
        ])
        .send()
        .await
        .map_err(|error| AppError::Message(error.to_string()))?;

    if !response.status().is_success() {
        return Err(AppError::Message(format!(
            "OAuth refresh failed with status {}",
            response.status()
        )));
    }

    let token: TokenResponse = response
        .json()
        .await
        .map_err(|error| AppError::Message(error.to_string()))?;
    Ok(build_session(token, Some(session)))
}

fn build_session(token: TokenResponse, previous: Option<&OAuthSession>) -> OAuthSession {
    let now = unix_now();
    let account_id = extract_account_id(&token.access_token)
        .or_else(|| previous.and_then(|value| value.account_id.clone()));

    OAuthSession {
        access_token: token.access_token,
        refresh_token: token
            .refresh_token
            .or_else(|| previous.map(|value| value.refresh_token.clone()))
            .unwrap_or_default(),
        id_token: token
            .id_token
            .or_else(|| previous.and_then(|value| value.id_token.clone())),
        account_id,
        expires_at: now.saturating_add(token.expires_in),
    }
}

fn status_from_session(session: &OAuthSession) -> OAuthStatus {
    OAuthStatus {
        logged_in: true,
        auth_mode: Some("chatgpt-oauth".to_string()),
        account_id: session.account_id.clone(),
        expires_at: Some(session.expires_at),
    }
}

fn save_oauth_session(session: &OAuthSession) -> Result<(), AppError> {
    let entry = Entry::new(OAUTH_SERVICE, OAUTH_ACCOUNT)
        .map_err(|error| AppError::Message(error.to_string()))?;
    let payload =
        serde_json::to_string(session).map_err(|error| AppError::Message(error.to_string()))?;
    entry
        .set_password(&payload)
        .map_err(|error| AppError::Message(error.to_string()))?;
    Ok(())
}

pub fn clear_oauth_session() -> Result<(), AppError> {
    let entry = Entry::new(OAUTH_SERVICE, OAUTH_ACCOUNT)
        .map_err(|error| AppError::Message(error.to_string()))?;

    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(AppError::Message(error.to_string())),
    }
}

fn load_oauth_session() -> Result<Option<OAuthSession>, AppError> {
    let entry = Entry::new(OAUTH_SERVICE, OAUTH_ACCOUNT)
        .map_err(|error| AppError::Message(error.to_string()))?;
    match entry.get_password() {
        Ok(value) => {
            let session: OAuthSession = serde_json::from_str(&value)
                .map_err(|error| AppError::Message(error.to_string()))?;
            Ok(Some(session))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(AppError::Message(error.to_string())),
    }
}

async fn ensure_fresh_session() -> Result<Option<OAuthSession>, AppError> {
    let Some(session) = load_oauth_session()? else {
        return Ok(None);
    };

    if !is_expired(&session, 60) {
        return Ok(Some(session));
    }

    let refreshed = refresh_session(&session).await?;
    save_oauth_session(&refreshed)?;
    Ok(Some(refreshed))
}

fn wait_for_callback(expected_state: &str) -> Result<CallbackPayload, AppError> {
    let listener = TcpListener::bind(("127.0.0.1", CALLBACK_PORT))
        .map_err(|error| AppError::Message(format!("Cannot bind localhost callback: {}", error)))?;
    listener
        .set_ttl(64)
        .map_err(|error| AppError::Message(error.to_string()))?;

    let (mut stream, _) = listener
        .accept()
        .map_err(|error| AppError::Message(error.to_string()))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(30)))
        .map_err(|error| AppError::Message(error.to_string()))?;

    let mut buffer = [0_u8; 4096];
    let read = stream
        .read(&mut buffer)
        .map_err(|error| AppError::Message(error.to_string()))?;
    let request = String::from_utf8_lossy(&buffer[..read]).to_string();
    let first_line = request
        .lines()
        .next()
        .ok_or_else(|| AppError::Message("OAuth callback request was empty".to_string()))?;
    let path = first_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| AppError::Message("OAuth callback path is missing".to_string()))?;

    let url = Url::parse(&format!("http://localhost{}", path))
        .map_err(|error| AppError::Message(error.to_string()))?;
    let callback_path = url.path();
    if callback_path != CALLBACK_PATH {
        return Err(AppError::Message(format!(
            "Unexpected OAuth callback path {}",
            callback_path
        )));
    }

    let mut code = None;
    let mut state = None;
    for (key, value) in url.query_pairs() {
        if key == "code" {
            code = Some(value.to_string());
        } else if key == "state" {
            state = Some(value.to_string());
        }
    }

    let html = if state.as_deref() == Some(expected_state) && code.is_some() {
        success_page().to_string()
    } else {
        error_page("OAuth state mismatch or missing code")
    };
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    );
    stream
        .write_all(response.as_bytes())
        .map_err(|error| AppError::Message(error.to_string()))?;

    let actual_state =
        state.ok_or_else(|| AppError::Message("OAuth callback is missing state".to_string()))?;
    if actual_state != expected_state {
        return Err(AppError::Message("OAuth state mismatch".to_string()));
    }

    Ok(CallbackPayload {
        code: code
            .ok_or_else(|| AppError::Message("OAuth callback is missing code".to_string()))?,
    })
}

fn build_code_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

fn generate_random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

fn extract_account_id(access_token: &str) -> Option<String> {
    let payload = access_token.split('.').nth(1)?;
    let decoded = URL_SAFE_NO_PAD.decode(payload).ok()?;
    let value: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    value
        .get("https://api.openai.com/auth")
        .and_then(|field| field.get("chatgpt_account_id"))
        .and_then(|field| field.as_str())
        .map(ToString::to_string)
}

fn is_expired(session: &OAuthSession, skew_seconds: u64) -> bool {
    session.expires_at <= unix_now().saturating_add(skew_seconds)
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn success_page() -> &'static str {
    "<!doctype html><html><body style=\"font-family:Segoe UI,sans-serif;background:#0b1020;color:#e6ecff;display:grid;place-items:center;height:100vh;margin:0\"><div><h1>Bankai OAuth complete</h1><p>You can close this tab and return to the app.</p></div></body></html>"
}

fn error_page(message: &str) -> String {
    format!(
        "<!doctype html><html><body style=\"font-family:Segoe UI,sans-serif;background:#0b1020;color:#e6ecff;display:grid;place-items:center;height:100vh;margin:0\"><div><h1>Bankai OAuth failed</h1><p>{}</p></div></body></html>",
        message
    )
}

struct CallbackPayload {
    code: String,
}
