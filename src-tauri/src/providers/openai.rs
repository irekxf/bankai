use async_openai::{
    config::OpenAIConfig,
    types::responses::{
        CreateResponseArgs, FunctionCallOutputItemParam, Item, OutputItem, Response,
    },
    Client,
};
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    settings::{load_openai_bearer_token, ProviderConfig},
    tools::response_tools_for_names,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunctionCall {
    pub response_id: String,
    pub call_id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone)]
pub enum ModelTurn {
    Text(String),
    ToolCall(ToolFunctionCall),
}

pub async fn list_models(config: &ProviderConfig) -> Result<Vec<String>, AppError> {
    let client = build_client(config).await?;
    let response = client
        .models()
        .list()
        .await
        .map_err(|error| AppError::Message(error.to_string()))?;

    let mut models = response
        .data
        .into_iter()
        .map(|model| model.id)
        .filter(|model| is_user_selectable_model(model))
        .collect::<Vec<_>>();
    if !models.iter().any(|model| model == &config.model) {
        models.push(config.model.clone());
    }
    models.sort_by_cached_key(|model| model_sort_key(model));
    models.dedup();
    Ok(models)
}

pub async fn create_tool_aware_response(
    config: &ProviderConfig,
    prompt: &str,
    enabled_tool_names: &[String],
) -> Result<ModelTurn, AppError> {
    let client = build_client(config).await?;
    let tool_definitions = response_tools_for_names(enabled_tool_names);
    let mut request = CreateResponseArgs::default();
    let mut request = request
        .model(config.model.clone())
        .input(prompt)
        .parallel_tool_calls(false);

    if !tool_definitions.is_empty() {
        request = request.tools(tool_definitions);
    }

    let request = request
        .build()
        .map_err(|error| AppError::Message(error.to_string()))?;

    let response = client
        .responses()
        .create(request)
        .await
        .map_err(|error| AppError::Message(error.to_string()))?;

    map_response_turn(response)
}

pub async fn continue_after_function_output(
    config: &ProviderConfig,
    previous_response_id: &str,
    call_id: &str,
    output: &str,
) -> Result<String, AppError> {
    let client = build_client(config).await?;
    let output_item = Item::FunctionCallOutput(FunctionCallOutputItemParam {
        call_id: call_id.to_string(),
        output: output.to_string().into(),
        id: None,
        status: None,
    });

    let request = CreateResponseArgs::default()
        .model(config.model.clone())
        .previous_response_id(previous_response_id)
        .input(output_item)
        .build()
        .map_err(|error| AppError::Message(error.to_string()))?;

    let response = client
        .responses()
        .create(request)
        .await
        .map_err(|error| AppError::Message(error.to_string()))?;

    Ok(response
        .output_text()
        .unwrap_or_else(|| "OpenAI returned an empty response.".to_string()))
}

async fn build_client(config: &ProviderConfig) -> Result<Client<OpenAIConfig>, AppError> {
    let api_key = load_openai_bearer_token(config).await?;
    let openai_config = OpenAIConfig::new()
        .with_api_key(api_key)
        .with_api_base(config.base_url.clone());
    Ok(Client::with_config(openai_config))
}

fn map_response_turn(response: Response) -> Result<ModelTurn, AppError> {
    for item in &response.output {
        if let OutputItem::FunctionCall(call) = item {
            return Ok(ModelTurn::ToolCall(ToolFunctionCall {
                response_id: response.id.clone(),
                call_id: call.call_id.clone(),
                name: call.name.clone(),
                arguments: call.arguments.clone(),
            }));
        }
    }

    Ok(ModelTurn::Text(response.output_text().unwrap_or_else(
        || "OpenAI returned an empty response.".to_string(),
    )))
}

fn is_user_selectable_model(model: &str) -> bool {
    let blocked_prefixes = [
        "babbage",
        "davinci",
        "dall-e",
        "embedding",
        "ft:",
        "omni-moderation",
        "text-embedding",
        "text-moderation",
        "tts-",
        "whisper",
    ];

    let allowed_prefixes = ["chatgpt-", "codex-", "gpt-", "o1", "o3", "o4"];
    let allowed_exact = ["gpt-4.1", "gpt-4o", "gpt-4o-mini"];

    if blocked_prefixes
        .iter()
        .any(|prefix| model.starts_with(prefix))
    {
        return false;
    }

    allowed_exact.contains(&model)
        || allowed_prefixes
            .iter()
            .any(|prefix| model.starts_with(prefix))
}

fn model_sort_key(model: &str) -> (u8, String) {
    let rank = if model.starts_with("gpt-5") {
        0
    } else if model.starts_with("gpt-4.1") {
        1
    } else if model.starts_with("gpt-4o") {
        2
    } else if model.starts_with("o4") {
        3
    } else if model.starts_with("o3") {
        4
    } else if model.starts_with("o1") {
        5
    } else if model.starts_with("codex-") {
        6
    } else if model.starts_with("chatgpt-") {
        7
    } else if model.starts_with("gpt-") {
        8
    } else {
        9
    };

    (rank, model.to_string())
}
