# PROJECT.md — Bankai

> **Bankai** (卍解) — нативное десктопное приложение для управления AI-агентами с визуальным контролем, approval flow и поддержкой множества LLM-провайдеров.
>
> Названо в честь финального высвобождения из Bleach — полное раскрытие силы. Каждый агент — это твой Zanpakutō, Bankai — его высшая форма.

---

## 1. Видение продукта

Приложение заполняет нишу между чат-клиентами (Jan, Msty, LM Studio) и агентными платформами (OpenClaw, ZeroClaw). Чат-клиенты не умеют в агентность (tool use, shell, браузер). Агентные платформы управляются через CLI и мессенджеры без визуального десктопного UI.

Наш продукт — нативный десктоп с полным агентным функционалом: пользователь видит что агент делает, одобряет или отклоняет действия, настраивает инструменты и переключает модели — всё в одном окне.

В будущем — система расширений по аналогии с VS Code. Но на первом этапе MCP (Model Context Protocol) покрывает значительную часть этой потребности, позволяя подключать сотни существующих MCP-серверов.

---

## 2. Технологический стэк

### 2.1 Desktop & Frontend

| Компонент | Технология | Версия / Примечания |
|-----------|-----------|---------------------|
| Desktop framework | **Tauri v2** | Rust бэкенд, WebView фронтенд |
| Frontend фреймворк | **Svelte 5** | Runes API ($state, $derived, $effect) |
| JS runtime / bundler | **Bun** | Замена Node.js + Vite |
| UI kit | **@material/web** | Официальные Web Components от Google (Material Design 3 / Material You). Используются как обычные HTML-теги: `<md-filled-button>`, `<md-outlined-text-field>`, `<md-dialog>` и т.д. Не требуют обёрток для Svelte. |
| Frontend state | **Svelte stores** | Встроенные, без внешних зависимостей |
| Markdown рендеринг | **svelte-streamdown** | Оптимизирован для LLM-стриминга, встроенная подсветка кода, math, mermaid |
| Код-эдитор | **Monaco Editor** | Через `@monaco-editor/loader` или `svelte-monaco` |
| Терминал | **xterm.js** | Через `xterm-svelte` (@battlefieldduck/xterm-svelte) |
| Подсветка синтаксиса | **shiki** или **svelte-highlight** | Для блоков кода вне Monaco |
| Git | На стороне Rust (**git2** крейт) | Не использовать isomorphic-git на JS |

### 2.2 Rust-бэкенд (src-tauri)

| Компонент | Крейт | Назначение |
|-----------|-------|------------|
| Async runtime | `tokio` | Все async операции |
| OpenAI + ChatGPT API | `async-openai` | Основной провайдер MVP: tool use, streaming, compatible endpoints |
| OpenAI-compatible | `async-openai` | Ollama, Groq, Together и другие OpenAI-compatible API |
| MCP клиент | `rmcp` | Официальный Rust SDK для Model Context Protocol. Подключение внешних MCP-серверов как источников инструментов |
| HTTP клиент | `reqwest` | Общие HTTP-запросы |
| База данных | `sqlx` + SQLite | Персистентное хранение |
| Хранение секретов | `keyring` | API-ключи через системный keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service) |
| Логирование | `tracing` + `tracing-subscriber` | Структурированное логирование |
| Shell-инструмент | `tokio::process` | Запуск команд с контролем |
| Браузер-инструмент | `chromiumoxide` | Управление браузером через Chrome DevTools Protocol (CDP) |
| Git | `git2` | Биндинги к libgit2 |
| Сериализация | `serde` + `serde_json` | JSON сериализация повсюду |

---

## 3. Архитектура

### 3.1 Общая схема

```
┌─────────────────────────────────────────────────────┐
│                    Svelte 5 Frontend                │
│  ┌──────────┐ ┌──────────┐ ┌───────────────────┐   │
│  │   Chat   │ │ Terminal │ │  Code Editor      │   │
│  │   View   │ │ (xterm)  │ │  (Monaco)         │   │
│  └────┬─────┘ └────┬─────┘ └────────┬──────────┘   │
│       │             │                │              │
│  ┌────┴─────────────┴────────────────┴──────────┐   │
│  │         Tool Call & Approval Panel           │   │
│  └──────────────────┬───────────────────────────┘   │
└─────────────────────┼───────────────────────────────┘
                      │ Tauri IPC (commands + events)
┌─────────────────────┼───────────────────────────────┐
│                     │        Rust Backend            │
│  ┌─────────────────────────────────────────────┐    │
│  │              Agent Loop                      │    │
│  │  1. Получить промпт пользователя             │    │
│  │  2. Отправить в LLM через Provider           │    │
│  │  3. Если LLM вернул tool_call:               │    │
│  │     a. Отправить на фронт для approval       │    │
│  │     b. Ждать одобрения/отклонения            │    │
│  │     c. Выполнить инструмент                  │    │
│  │     d. Отправить результат обратно в LLM     │    │
│  │  4. Повторять пока LLM не даст финальный     │    │
│  │     текстовый ответ                          │    │
│  └──────┬──────────────┬───────────────┬────────┘    │
│         │              │               │             │
│  ┌──────┴──────┐ ┌─────┴─────┐ ┌──────┴──────┐     │
│  │  Providers  │ │   Tools   │ │    MCP      │     │
│  │  (LLM API) │ │  (shell,  │ │  (external  │     │
│  │             │ │  fs, CDP) │ │   servers)  │     │
│  └─────────────┘ └───────────┘ └─────────────┘     │
│                                                      │
│  ┌──────────────────────────────────────────────┐    │
│  │  SQLite (sqlx): sessions, messages,           │    │
│  │  tool_calls, provider_configs, agent_profiles │    │
│  └──────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────┘
```

### 3.2 IPC между Rust и Frontend

**Rust → Frontend (события):** Tauri events для стриминга и обновлений:
- `agent:message-delta` — дельта текста от LLM (стриминг)
- `agent:tool-call-request` — агент хочет выполнить инструмент, ждёт одобрения
- `agent:tool-call-result` — результат выполнения инструмента
- `agent:error` — ошибка в агентном цикле
- `agent:status` — смена статуса (idle, thinking, awaiting_approval, executing_tool)

**Frontend → Rust (команды):** Tauri commands:
- `send_message(session_id, text)` — отправить промпт
- `approve_tool_call(call_id)` — одобрить действие
- `reject_tool_call(call_id, reason?)` — отклонить действие
- `cancel_generation(session_id)` — отменить генерацию
- `create_session(agent_profile_id?)` — создать сессию
- `list_sessions()` — получить список сессий
- `get_session_messages(session_id)` — история сообщений
- `configure_provider(provider_type, config)` — настроить провайдер
- `list_tools()` — список доступных инструментов
- `toggle_tool(tool_id, enabled)` — включить/выключить инструмент

### 3.3 Rust: ключевые трейты

```rust
/// Унифицированный интерфейс для LLM-провайдеров
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Имя провайдера ("anthropic", "openai", "ollama")
    fn name(&self) -> &str;

    /// Список доступных моделей
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;

    /// Отправить сообщения и получить ответ (стриминг через канал)
    async fn chat(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        model: &str,
        tx: mpsc::Sender<StreamEvent>,
    ) -> Result<()>;
}

/// Унифицированный интерфейс для инструментов агента
#[async_trait]
pub trait AgentTool: Send + Sync {
    /// Имя инструмента (для tool_call matching)
    fn name(&self) -> &str;

    /// Описание (передаётся в LLM)
    fn description(&self) -> &str;

    /// JSON Schema параметров (передаётся в LLM)
    fn parameters_schema(&self) -> serde_json::Value;

    /// Требует ли одобрения пользователя
    fn requires_approval(&self) -> bool;

    /// Выполнить инструмент
    async fn execute(&self, params: serde_json::Value) -> Result<ToolResult>;
}

/// Стриминг-события от LLM
pub enum StreamEvent {
    TextDelta(String),
    ToolCallStart { id: String, name: String, arguments: String },
    ToolCallEnd { id: String },
    Done { usage: TokenUsage },
    Error(String),
}
```

---

## 4. Структура проекта

```
bankai/
├── PROJECT.md                  # ← этот файл
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs             # Точка входа Tauri
│       ├── lib.rs              # Регистрация commands
│       ├── state.rs            # AppState (Arc<Mutex<...>>)
│       ├── error.rs            # Единая обработка ошибок
│       ├── db/
│       │   ├── mod.rs          # Инициализация SQLite, миграции
│       │   ├── sessions.rs     # CRUD для сессий
│       │   ├── messages.rs     # CRUD для сообщений
│       │   └── settings.rs     # Настройки провайдеров
│       ├── providers/
│       │   ├── mod.rs          # trait LlmProvider + registry
│       │   ├── anthropic.rs    # Реализация для Claude (anthropic-sdk-rust)
│       │   ├── openai.rs       # Реализация для OpenAI (async-openai)
│       │   └── ollama.rs       # Реализация для Ollama (async-openai с custom base_url)
│       ├── tools/
│       │   ├── mod.rs          # trait AgentTool + registry
│       │   ├── shell.rs        # Выполнение shell-команд (tokio::process)
│       │   ├── filesystem.rs   # Чтение/запись файлов
│       │   ├── browser.rs      # Браузер-автоматизация (chromiumoxide)
│       │   └── http.rs         # HTTP-запросы
│       ├── mcp/
│       │   ├── mod.rs          # MCP клиент менеджер (rmcp)
│       │   └── bridge.rs       # Мост: MCP tools → AgentTool trait
│       ├── agent/
│       │   ├── mod.rs          # Основной агентный цикл
│       │   ├── loop.rs         # Цикл: prompt → LLM → tool_call → approval → execute → repeat
│       │   ├── approval.rs     # Система одобрения (каналы Rust ↔ Frontend)
│       │   └── memory.rs       # Персистентная память агента
│       └── ipc/
│           ├── mod.rs
│           ├── commands.rs     # Все #[tauri::command] функции
│           └── events.rs       # Типы событий для Tauri emit
├── src/                        # Svelte 5 Frontend
│   ├── app.html
│   ├── app.css                 # Глобальные стили, @material/web токены
│   ├── lib/
│   │   ├── stores/
│   │   │   ├── sessions.ts     # Svelte store для сессий
│   │   │   ├── messages.ts     # Svelte store для сообщений текущей сессии
│   │   │   ├── agent.ts        # Статус агента, pending tool calls
│   │   │   └── settings.ts     # Настройки приложения
│   │   ├── tauri/
│   │   │   ├── commands.ts     # Типизированные обёртки над invoke()
│   │   │   └── events.ts       # Подписки на Tauri events
│   │   └── utils/
│   │       └── markdown.ts     # Утилиты для markdown рендеринга
│   ├── components/
│   │   ├── chat/
│   │   │   ├── ChatView.svelte         # Основной чат
│   │   │   ├── MessageBubble.svelte    # Одно сообщение (user / assistant)
│   │   │   ├── ToolCallCard.svelte     # Визуализация tool call
│   │   │   ├── ApprovalPanel.svelte    # Одобрить / отклонить действие
│   │   │   └── ChatInput.svelte        # Поле ввода с отправкой
│   │   ├── sidebar/
│   │   │   ├── SessionList.svelte      # Список сессий
│   │   │   └── ToolList.svelte         # Доступные инструменты
│   │   ├── settings/
│   │   │   ├── ProviderConfig.svelte   # Настройка API-ключей и моделей
│   │   │   └── ToolConfig.svelte       # Настройка инструментов
│   │   ├── terminal/
│   │   │   └── TerminalView.svelte     # xterm.js обёртка
│   │   └── editor/
│   │       └── CodeEditor.svelte       # Monaco обёртка
│   └── routes/
│       └── +page.svelte                # Главная страница (layout)
├── bun.lockb
├── package.json
├── svelte.config.js
├── tsconfig.json
└── vite.config.ts
```

---

## 5. База данных (SQLite)

### Таблицы

```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    title TEXT,
    agent_profile_id TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions(id),
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system', 'tool')),
    content TEXT,
    tool_call_id TEXT,          -- для role='tool': ID вызова
    tool_calls_json TEXT,       -- для role='assistant': JSON массив tool_calls
    token_usage_json TEXT,      -- {"input": N, "output": N}
    model TEXT,
    provider TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE tool_calls (
    id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL REFERENCES messages(id),
    session_id TEXT NOT NULL REFERENCES sessions(id),
    tool_name TEXT NOT NULL,
    arguments_json TEXT NOT NULL,
    result_json TEXT,
    status TEXT NOT NULL CHECK (status IN ('pending', 'approved', 'rejected', 'completed', 'failed')),
    approved_at DATETIME,
    completed_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE provider_configs (
    id TEXT PRIMARY KEY,
    provider_type TEXT NOT NULL, -- 'anthropic', 'openai', 'ollama'
    display_name TEXT,
    base_url TEXT,               -- NULL для дефолтных, custom для Ollama и т.д.
    default_model TEXT,
    is_enabled INTEGER DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
-- API-ключи хранятся в keyring, НЕ в БД

CREATE TABLE agent_profiles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    system_prompt TEXT,
    provider_config_id TEXT REFERENCES provider_configs(id),
    model TEXT,
    tools_json TEXT,            -- JSON массив enabled tool names
    temperature REAL DEFAULT 0.7,
    max_tokens INTEGER DEFAULT 4096,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

---

## 6. Последовательность разработки (фазы)

### Фаза 1 — Минимальный агент (MVP)
- [ ] Scaffolding: Tauri v2 + Svelte 5 + Bun
- [ ] Подключить @material/web компоненты
- [ ] Один провайдер: OpenAI / ChatGPT (`async-openai`)
- [ ] Один инструмент: shell (tokio::process)
- [ ] Агентный цикл: prompt → LLM → tool_call → approval → execute → response
- [ ] Базовый чат UI: сообщения, стриминг, tool call карточки, approval кнопки
- [ ] SQLite: сессии и сообщения
- [ ] Хранение API-ключа через keyring

### Фаза 2 — Мульти-провайдер + инструменты
- [ ] Добавить OpenAI провайдер (async-openai)
- [ ] Добавить Ollama провайдер (async-openai с custom base_url)
- [ ] Инструмент: filesystem (чтение/запись/листинг)
- [ ] Инструмент: browser (chromiumoxide)
- [ ] UI: настройка провайдеров и моделей
- [ ] UI: переключение модели на лету
- [ ] Sidebar: список сессий, создание/удаление

### Фаза 3 — Память и профили
- [ ] Персистентная память между сессиями
- [ ] Системные промпты
- [ ] Профили агентов (разные наборы инструментов + промпт + модель)
- [ ] Markdown рендеринг с подсветкой кода (svelte-streamdown)
- [ ] Встроенный терминал (xterm.js)

### Фаза 4 — MCP
- [ ] MCP клиент через rmcp
- [ ] Мост: MCP tools → AgentTool trait (автоматическая конвертация)
- [ ] UI: подключение/отключение MCP-серверов
- [ ] UI: просмотр доступных MCP-инструментов

### Фаза 5 — Расширения (будущее)
- [ ] Система расширений по аналогии с VS Code
- [ ] Мультиагентность
- [ ] Голосовой ввод

---

## 7. Важные правила

### 7.1 Код
- Rust: используй `thiserror` для ошибок, `anyhow` не использовать в библиотечном коде
- Rust: все Tauri commands возвращают `Result<T, String>` (требование Tauri)
- Rust: агентный цикл работает в отдельном tokio task, общение с фронтом только через events/commands
- Svelte: используй Svelte 5 Runes API ($state, $derived, $effect), не legacy API
- Svelte: @material/web компоненты используются как обычные HTML-элементы, типы через `/// <reference types="@material/web" />`
- TypeScript строгий режим, никаких `any`

### 7.2 Безопасность
- API-ключи ТОЛЬКО в keyring, НИКОГДА в SQLite, конфигах или коде
- Для ChatGPT/OpenAI в MVP используем API key, не OAuth login flow
- Shell-инструмент ВСЕГДА требует approval перед выполнением
- Browser-инструмент ВСЕГДА требует approval
- Filesystem write ВСЕГДА требует approval
- Filesystem read можно выполнять без approval (настраиваемо)

### 7.3 UI/UX
- Material You (M3) дизайн-система — следовать гайдлайнам Material Design 3
- Тёмная и светлая тема через M3 dynamic color
- Чат показывает tool calls inline как карточки с деталями (имя инструмента, аргументы, результат)
- Approval — крупные кнопки "Approve" / "Reject" с превью действия
- Стриминг текста — посимвольный вывод, как в ChatGPT

---

## 8. Зависимости для быстрого старта

### package.json (ключевые)
```json
{
  "name": "bankai",
  "dependencies": {
    "@material/web": "^2.0.0",
    "@battlefieldduck/xterm-svelte": "latest",
    "@monaco-editor/loader": "latest",
    "svelte-streamdown": "latest"
  },
  "devDependencies": {
    "@sveltejs/vite-plugin-svelte": "latest",
    "@tauri-apps/api": "latest",
    "@tauri-apps/cli": "latest",
    "svelte": "^5.0.0",
    "typescript": "latest",
    "vite": "latest"
  }
}
```

### Cargo.toml (ключевые)
```toml
[package]
name = "bankai"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri = { version = "2", features = [] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
reqwest = { version = "0.12", features = ["json"] }
keyring = "3"
tracing = "0.1"
tracing-subscriber = "0.3"
async-openai = "0.26"
rmcp = { version = "0.16", features = ["client"] }
chromiumoxide = "0.7"
git2 = "0.19"
thiserror = "2"
async-trait = "0.1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

---

## 9. Контекст для агентов

- **Не используем** наработки конкурентов (Jan, Msty, OpenClaw и т.д.)
- **Не используем** OAuth как основной способ доступа к ChatGPT в MVP; используем OpenAI API key
- **Не используем** genai крейт — tool use недостаточно проработан
- **Не используем** Electron, React, Vue — осознанный выбор в пользу Tauri + Svelte
- **Не используем** полностью Rust GUI (egui, iced, GPUI) — нет готовых компонентов (Monaco, xterm, markdown)
- **Monaco Editor и xterm.js** — фреймворк-агностичные JS библиотеки, интегрируются через loader/обёртки
- **@material/web** — стандартные Web Components, работают в Svelte без обёрток
- **rmcp (MCP)** — стратегический выбор, заменяет половину будущей системы расширений
- **Git операции** — на стороне Rust через git2, не на JS
