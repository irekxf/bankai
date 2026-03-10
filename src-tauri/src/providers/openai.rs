use async_openai::{
    config::OpenAIConfig,
    types::responses::{CreateResponseArgs, ResponseStreamEvent},
    Client,
};
use futures::StreamExt;

use crate::{
    error::AppError,
    settings::{load_openai_api_key, ProviderConfig},
};

pub async fn stream_chat_response<F>(
    config: &ProviderConfig,
    prompt: &str,
    mut on_delta: F,
) -> Result<(), AppError>
where
    F: FnMut(String) -> Result<(), AppError>,
{
    let api_key = load_openai_api_key()?;
    let openai_config = OpenAIConfig::new()
        .with_api_key(api_key)
        .with_api_base(config.base_url.clone());
    let client = Client::with_config(openai_config);

    let request = CreateResponseArgs::default()
        .model(config.model.clone())
        .input(prompt)
        .stream(true)
        .build()
        .map_err(|error| AppError::Message(error.to_string()))?;

    let mut stream = client
        .responses()
        .create_stream(request)
        .await
        .map_err(|error| AppError::Message(error.to_string()))?;

    while let Some(event) = stream.next().await {
        let event = event.map_err(|error| AppError::Message(error.to_string()))?;
        if let ResponseStreamEvent::ResponseOutputTextDelta(delta) = event {
            on_delta(delta.delta)?;
        }
    }

    Ok(())
}
