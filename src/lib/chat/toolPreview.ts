type ToolPayload = Record<string, unknown>;

function readText(payload: ToolPayload, key: string): string | undefined {
  const value = payload[key];
  return typeof value === "string" && value.length > 0 ? value : undefined;
}

export function formatToolPreview(toolName: string, argumentsJson?: string): string {
  if (!argumentsJson) {
    return "No arguments";
  }

  try {
    const payload = JSON.parse(argumentsJson) as ToolPayload;

    if (toolName === "shell") {
      return readText(payload, "command") ?? argumentsJson;
    }

    if (toolName === "filesystem") {
      const action = readText(payload, "action") ?? "unknown";
      const path = readText(payload, "path") ?? "";
      return `${action} ${path}`.trim();
    }

    if (toolName === "browser") {
      const action = readText(payload, "action") ?? "unknown";
      const url = readText(payload, "url") ?? "";
      return `${action} ${url}`.trim();
    }
  } catch {
    return argumentsJson;
  }

  return argumentsJson;
}
