use async_openai::{
    config::OpenAIConfig,
    types::responses::{
        CreateResponseArgs, FunctionCallOutputItemParam, FunctionTool, Item, OutputItem, Response,
        Tool,
    },
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    error::AppError,
    settings::{load_openai_bearer_token, ProviderConfig},
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
) -> Result<ModelTurn, AppError> {
    let client = build_client(config).await?;
    let request = CreateResponseArgs::default()
        .model(config.model.clone())
        .input(prompt)
        .parallel_tool_calls(false)
        .tools(vec![shell_tool(), filesystem_tool()])
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

fn shell_tool() -> Tool {
    Tool::Function(FunctionTool {
        name: "shell".to_string(),
        description: Some(
            "Run a shell command on the local machine. Use only when necessary.".to_string(),
        ),
        parameters: Some(json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Powershell command to execute"
                }
            },
            "required": ["command"],
            "additionalProperties": false
        })),
        strict: Some(true),
    })
}

fn filesystem_tool() -> Tool {
    Tool::Function(FunctionTool {
        name: "filesystem".to_string(),
        description: Some(
            "Read files, write files, or list directory contents on the local workspace."
                .to_string(),
        ),
        parameters: Some(json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["read_file", "write_file", "list_dir"]
                },
                "path": {
                    "type": "string",
                    "description": "Absolute or workspace-relative path"
                },
                "content": {
                    "type": "string",
                    "description": "Required when action is write_file"
                }
            },
            "required": ["action", "path"],
            "additionalProperties": false
        })),
        strict: Some(true),
    })
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
