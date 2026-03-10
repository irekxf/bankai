use std::process::Command;

use reqwest::Url;
use serde::Deserialize;

use crate::error::AppError;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "action")]
pub enum BrowserRequest {
    OpenUrl { url: String },
    ReadPage { url: String },
}

pub async fn execute_browser(request: BrowserRequest) -> Result<String, AppError> {
    match request {
        BrowserRequest::OpenUrl { url } => open_url(&url).await,
        BrowserRequest::ReadPage { url } => read_page(&url).await,
    }
}

async fn open_url(raw_url: &str) -> Result<String, AppError> {
    let url = parse_web_url(raw_url)?;

    launch_in_browser(url.as_str())?;
    let title = fetch_page(url.as_str())
        .await
        .map(|page| page.title)
        .unwrap_or_default();

    if title.is_empty() {
        Ok(format!("Opened {} in the system browser.", url))
    } else {
        Ok(format!(
            "Opened {} in the system browser. Page title: {}",
            url, title
        ))
    }
}

async fn read_page(raw_url: &str) -> Result<String, AppError> {
    let url = parse_web_url(raw_url)?;
    let page = fetch_page(url.as_str()).await?;

    let mut result = format!("URL: {}", url);
    if !page.title.is_empty() {
        result.push_str(&format!("\nTitle: {}", page.title));
    }
    if !page.text.is_empty() {
        result.push_str("\nContent:\n");
        result.push_str(&page.text);
    } else {
        result.push_str("\nContent: No readable text extracted.");
    }

    Ok(result)
}

fn parse_web_url(raw_url: &str) -> Result<Url, AppError> {
    let url = Url::parse(raw_url).map_err(|error| AppError::Message(error.to_string()))?;
    let scheme = url.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(AppError::Message(
            "Browser tool only allows http or https URLs.".to_string(),
        ));
    }
    Ok(url)
}

fn launch_in_browser(url: &str) -> Result<(), AppError> {
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

async fn fetch_page(url: &str) -> Result<PageSummary, AppError> {
    let body = reqwest::get(url)
        .await
        .map_err(|error| AppError::Message(error.to_string()))?
        .text()
        .await
        .map_err(|error| AppError::Message(error.to_string()))?;

    Ok(PageSummary {
        title: extract_title(&body).unwrap_or_default(),
        text: html_to_text(&body, 4000),
    })
}

fn extract_title(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let start = lower.find("<title>")?;
    let end = lower[start + 7..].find("</title>")?;
    let title = &html[start + 7..start + 7 + end];
    let cleaned = title.split_whitespace().collect::<Vec<_>>().join(" ");
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn html_to_text(html: &str, limit: usize) -> String {
    let mut text = String::with_capacity(limit.min(html.len()));
    let mut in_tag = false;
    let mut last_was_space = false;

    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
            continue;
        }
        if ch == '>' {
            in_tag = false;
            if !last_was_space && !text.is_empty() {
                text.push(' ');
                last_was_space = true;
            }
            continue;
        }
        if in_tag {
            continue;
        }

        let normalized = if ch.is_whitespace() { ' ' } else { ch };
        if normalized == ' ' {
            if last_was_space || text.is_empty() {
                continue;
            }
            text.push(' ');
            last_was_space = true;
        } else {
            text.push(normalized);
            last_was_space = false;
        }

        if text.len() >= limit {
            break;
        }
    }

    text.trim().to_string()
}

struct PageSummary {
    title: String,
    text: String,
}
