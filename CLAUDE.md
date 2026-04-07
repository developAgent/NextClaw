# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CEOClaw is a cross-platform desktop AI assistant built with Rust (Tauri 2.0) and React/TypeScript. It enables non-technical users to interact with Anthropic Claude to execute commands, process files, and automate tasks through natural language.

## Development Commands

```bash
# Start Tauri development server (frontend + backend)
npm run tauri dev

# Build production bundle
npm run tauri build

# Development-only commands
npm run dev          # Start Vite dev server only
npm run build        # Build frontend only
npm run preview      # Preview frontend build

# Code quality
npm run lint         # ESLint check
npm run lint:fix     # ESLint auto-fix
```

## Requirements

- Rust 1.70+ with 2021 edition
- Node.js >=18.14.1
- Tauri CLI (installed via npm)
- Platform-specific build tools:
  - Linux: libwebkit2gtk-4.0-dev, build-essential, curl, wget, file, libssl-dev, libayatana-appindicator3-dev, librsvg2-dev
  - macOS: Xcode Command Line Tools
  - Windows: Microsoft C++ Build Tools, WebView2

## Project Structure

```
ceo-claw/
├── src/                    # React frontend
│   ├── components/         # React components
│   ├── store/             # Zustand state management
│   ├── types/             # TypeScript type definitions
│   └── utils/             # Utility functions
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── ai/            # Claude API integration
│   │   ├── commands/      # Tauri command handlers
│   │   ├── db/            # DuckDB database layer
│   │   ├── exec/          # Command execution engine
│   │   ├── telemetry/     # Logging setup
│   │   └── utils/         # Shared utilities
│   └── tauri.conf.json    # Tauri configuration
├── Cargo.toml             # Rust workspace config
└── package.json           # Node dependencies
```

## Key Technologies

**Backend (Rust 2021):**
- Tauri 2.0 - Desktop app framework
- DuckDB - Embedded database
- Tokio - Async runtime
- anthropic-rs - Claude API SDK
- Tracing - Structured logging
- secrecy - Secret handling for API keys

**Frontend (TypeScript):**
- React 18 + Vite
- Tailwind CSS
- Zustand - State management
- Lucide React - Icons

## Architecture Patterns

### Tauri Commands
All backend functions exposed to frontend are in `src-tauri/src/commands/`. Commands use Tauri's `#[tauri::command]` macro and return `Result<T>` for error handling.

### Database Layer
DuckDB connection is managed by `db::Database` and injected via Tauri state. All database operations use prepared statements for safety.

### State Management
Frontend uses Zustand store (`store/index.ts`) for app state: messages, sessions, loading state. Persisted in DuckDB via Tauri commands.

### Error Handling
- Rust: `utils/error.rs` defines `AppError` enum with categories
- Frontend: Errors are returned from Tauri commands as `Err(String)`

### Security
- Commands validated against whitelist/blacklist (`exec/security.rs`)
- API keys encrypted with `secrecy` crate
- Dangerous commands require user confirmation
- Path sandboxing available

## Code Style

**Rust:**
- 2021 edition enforced
- `#![forbid(unsafe_code)]` enabled
- Clippy pedantic/nursery lints (with allowances)
- Use `#[must_use]` for return values that should be checked
- Prefer `const fn` where possible

**TypeScript:**
- Strict mode enabled
- No `any` types (use `unknown` instead)
- Prefer interfaces over types for object shapes
- Use `cn()` utility for conditional classes

## Configuration

- Tauri config: `src-tauri/tauri.conf.json`
- User config: `~/.config/CEOClaw/config.toml` (auto-generated)
- Database: `~/.local/share/CEOClaw/ceo-claw.db`
- Logs: `~/.local/share/CEOClaw/logs/ceo-claw.log` (production)

## Common Tasks

### Adding a new Tauri command
1. Create function in appropriate `commands/*.rs` with `#[tauri::command]` macro
2. Register in `lib.rs` `invoke_handler!` macro
3. Add TypeScript types in `types/index.ts`
4. Call from frontend using `invoke('command_name', { args })`

### Adding a new database table
1. Add `CREATE TABLE` statement in `db/connection.rs::init_schema()`
2. Create model in `db/models.rs`
3. Add CRUD operations in `connection.rs`

### Adding a new UI component
1. Create in `src/components/`
2. Import and use in parent component
3. Use Tailwind classes for styling
4. Use Lucide icons

## Security Considerations

- Never log or serialize API keys
- All user commands must be validated before execution
- File operations must check permissions
- Use `secrecy::SecretString` for sensitive data
- Validate all user inputs

## Testing

Run Rust tests: `cargo test` (in src-tauri directory)
The project uses unit tests with `#[cfg(test)]` modules.

## Build Platform

Use `npm run tauri build` to create platform-specific bundles:
- Windows: `.exe` installer (NSIS)
- macOS: `.app` bundle and `.dmg`
- Linux: `.deb`, `.AppImage`