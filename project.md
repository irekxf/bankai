# PROJECT.md — Bankai

> **Bankai** (卍解) — нативное десктопное приложение для управления AI-агентами с визуальным контролем, approval flow и поддержкой множества LLM-провайдеров.
>
> Названо в честь финального высвобождения из Bleach — полное раскрытие силы. Каждый агент — это твой Zanpakutō, Bankai — его высшая форма.

---

## 1. Видение продукта

Bankai закрывает нишу между обычными AI чат-клиентами и агентными платформами без удобного desktop UI.

Чего мы хотим:
- нативное desktop-приложение, а не Electron
- полноценный агентный цикл с tool use
- явный approval flow перед опасными действиями
- визуальный контроль над тем, что делает агент
- возможность подключать разные модели и внешние инструменты

Продуктовая идея: пользователь должен видеть не только ответ модели, но и сам процесс работы агента. Shell, browser, filesystem, MCP-инструменты и дальнейшие расширения должны быть встроены в один desktop-интерфейс.

В будущем планируются расширения по модели VS Code и более развитая мультиагентность. На ранних этапах значительную часть расширяемости закрывает MCP.

---

## 2. Принятые решения

### 2.1 Что используем

- Desktop framework: `Tauri v2`
- Frontend: `Svelte 5`
- Bundler/runtime для фронта: `Vite`, `Bun`
- UI kit: `@material/web`
- Backend: `Rust + tokio`
- OpenAI integration: `async-openai` с Responses API
- Persistence: `SQLite + sqlx`
- Secrets: `keyring`
- Logging: `tracing`
- Git: `git2`

### 2.2 Что считаем актуальным сейчас

- OpenAI / ChatGPT остаётся основным провайдером для MVP
- Для авторизации допускаются два сценария: API key и OAuth
- OAuth не считается временным экспериментом, это допустимый рабочий путь проекта
- Approval flow обязателен для потенциально опасных инструментов
- Текущий фокус не на абстрактной архитектуре, а на рабочем MVP с дальнейшим наращиванием возможностей

### 2.3 Что не используем

- Electron
- React / Vue
- полностью Rust GUI вместо webview-подхода
- JS-библиотеки для git-операций вместо `git2`

---

## 3. Архитектура

### 3.1 Общая схема

```text
Svelte 5 frontend
  -> chat UI
  -> provider/settings UI
  -> session list
  -> tool approval UI
  -> subscriptions to Tauri events

Tauri IPC
  -> commands from frontend to Rust
  -> events from Rust to frontend

Rust backend
  -> agent loop
  -> OpenAI provider integration
  -> tool execution layer
  -> approval state
  -> persistence layer
  -> OAuth / API key auth handling
```

### 3.2 Основной агентный цикл

1. Пользователь отправляет сообщение
2. Rust сохраняет user message и запускает provider request
3. Модель возвращает либо обычный текст, либо tool call
4. Tool call уходит в approval queue
5. После одобрения инструмент выполняется
6. Результат инструмента возвращается в модель
7. Агент продолжает цикл до финального ответа

### 3.3 События и команды

События, которые уже формируют основу UI:
- `agent:message-delta`
- `agent:tool-call-request`
- `agent:tool-call-result`
- `agent:error`
- `agent:status`

Команды, которые уже важны для MVP:
- `send_message`
- `list_sessions`
- `create_session`
- `get_session_messages`
- `approve_tool_call`
- `reject_tool_call`
- provider config commands
- auth-related commands for OAuth / API key flow

---

## 4. Текущая структура проекта

```text
bankai/
├── project.md
├── README.md
├── docs/
│   ├── AGENT_COORDINATION.md
│   └── tasks/
├── src/
│   ├── routes/
│   ├── components/
│   ├── lib/stores/
│   └── lib/tauri/
├── src-tauri/
│   ├── Cargo.toml
│   └── src/
│       ├── agent/
│       ├── db/
│       ├── ipc/
│       ├── providers/
│       ├── tools/
│       ├── settings.rs
│       ├── oauth.rs
│       └── lib.rs
└── package.json
```

Сейчас репозиторий уже содержит:
- базовый chat UI
- session list
- provider config UI
- OAuth onboarding / auth handling
- OpenAI provider integration
- shell / filesystem / browser tool execution base
- SQLite persistence для сессий, сообщений и pending tool calls

---

## 5. Ближайшие продуктовые задачи

### 5.1 Высокий приоритет

- стабилизировать auth UX: API key + OAuth без дублирующей и confusing логики
- добавить полноценный tool registry и включение/выключение инструментов
- улучшить agent loop для нескольких последовательных tool calls
- расширить и нормализовать хранение сообщений и tool call metadata

### 5.2 Средний приоритет

- улучшить session timeline и отображение tool activity в UI
- подготовить model/provider abstraction для OpenAI-compatible endpoints
- добавить встроенный terminal view
- добавить code editor pane

### 5.3 Следующие большие этапы

- agent profiles
- memory between sessions
- MCP integration
- extension system
- multi-agent workflows inside the app

---

## 6. Правила реализации

### 6.1 Код

- Rust: использовать `thiserror` для прикладных ошибок
- Tauri commands: `Result<T, String>`
- TypeScript: strict mode, без `any`
- Svelte: ориентироваться на Svelte 5 подход и не тащить legacy patterns без причины

### 6.2 Безопасность

- секреты только через `keyring`
- shell требует approval
- browser требует approval
- filesystem write требует approval
- filesystem read может быть отдельной политикой, но должен оставаться явно контролируемым

### 6.3 UX

- пользователь должен понимать, когда агент думает, ждёт approval и когда выполнил tool
- tool calls должны отображаться в UI явно, а не скрываться за текстовым ответом
- настройки провайдера должны быть рабочими, а не декоративными

---

## 7. Контекст для агентов

Если код и документация расходятся, ориентируемся на фактическое решение команды и обновляем документацию.

На текущий момент это означает:
- OAuth допустим и поддерживается проектом
- API key тоже допустим
- MVP не ограничен строго одним auth-потоком
- приоритет на рабочий агентный опыт, а не на догматичную чистоту первоначального плана
