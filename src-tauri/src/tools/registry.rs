use std::collections::HashMap;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub requires_approval: bool,
    pub enabled: bool,
}

struct ToolMeta {
    name: &'static str,
    description: &'static str,
    requires_approval: bool,
}

const KNOWN_TOOLS: &[ToolMeta] = &[
    ToolMeta {
        name: "shell",
        description: "Run a shell command on the local machine.",
        requires_approval: true,
    },
    ToolMeta {
        name: "filesystem",
        description: "Read files, write files, or list directory contents on the local workspace.",
        requires_approval: true,
    },
    ToolMeta {
        name: "browser",
        description: "Open a web page in the system browser or read a web page into text.",
        requires_approval: true,
    },
];

pub fn build_tool_list(enabled_map: &HashMap<String, bool>) -> Vec<ToolInfo> {
    KNOWN_TOOLS
        .iter()
        .map(|meta| {
            let enabled = enabled_map.get(meta.name).copied().unwrap_or(true);
            ToolInfo {
                name: meta.name.to_string(),
                description: meta.description.to_string(),
                requires_approval: meta.requires_approval,
                enabled,
            }
        })
        .collect()
}

pub fn is_known_tool(name: &str) -> bool {
    KNOWN_TOOLS.iter().any(|meta| meta.name == name)
}
