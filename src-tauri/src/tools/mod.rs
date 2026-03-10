pub mod browser;
pub mod filesystem;
pub mod shell;

use std::collections::HashMap;

use async_openai::types::responses::{FunctionTool, Tool};
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Clone, Copy)]
pub struct BuiltinTool {
    pub name: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub requires_approval: bool,
    pub enabled_by_default: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolRegistryEntry {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub enabled: bool,
    pub requires_approval: bool,
}

const BUILTIN_TOOLS: [BuiltinTool; 3] = [
    BuiltinTool {
        name: "shell",
        display_name: "Shell",
        description: "Run a shell command on the local machine when the task truly needs it.",
        requires_approval: true,
        enabled_by_default: true,
    },
    BuiltinTool {
        name: "filesystem",
        display_name: "Filesystem",
        description: "Read files, write files, or list directories inside the workspace.",
        requires_approval: true,
        enabled_by_default: true,
    },
    BuiltinTool {
        name: "browser",
        display_name: "Browser",
        description: "Open a URL in the system browser or read a web page into text.",
        requires_approval: true,
        enabled_by_default: true,
    },
];

pub fn builtin_tools() -> &'static [BuiltinTool] {
    &BUILTIN_TOOLS
}

pub fn is_builtin_tool(name: &str) -> bool {
    builtin_tools().iter().any(|tool| tool.name == name)
}

pub fn registry_entries(enabled_map: &HashMap<String, bool>) -> Vec<ToolRegistryEntry> {
    builtin_tools()
        .iter()
        .map(|tool| ToolRegistryEntry {
            name: tool.name.to_string(),
            display_name: tool.display_name.to_string(),
            description: tool.description.to_string(),
            enabled: enabled_map
                .get(tool.name)
                .copied()
                .unwrap_or(tool.enabled_by_default),
            requires_approval: tool.requires_approval,
        })
        .collect()
}

pub fn registry_entry(name: &str, enabled_map: &HashMap<String, bool>) -> Option<ToolRegistryEntry> {
    registry_entries(enabled_map)
        .into_iter()
        .find(|tool| tool.name == name)
}

pub fn response_tools_for_names(enabled_tool_names: &[String]) -> Vec<Tool> {
    enabled_tool_names
        .iter()
        .filter_map(|tool_name| response_tool_by_name(tool_name))
        .collect()
}

fn response_tool_by_name(name: &str) -> Option<Tool> {
    match name {
        "shell" => Some(shell_tool()),
        "filesystem" => Some(filesystem_tool()),
        "browser" => Some(browser_tool()),
        _ => None,
    }
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

fn browser_tool() -> Tool {
    Tool::Function(FunctionTool {
        name: "browser".to_string(),
        description: Some(
            "Open a web page in the user's browser or read a web page into text. Use when the user asks to open, navigate to, or inspect a site."
                .to_string(),
        ),
        parameters: Some(json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["open_url", "read_page"]
                },
                "url": {
                    "type": "string",
                    "description": "http or https URL to open or read"
                }
            },
            "required": ["action", "url"],
            "additionalProperties": false
        })),
        strict: Some(true),
    })
}
