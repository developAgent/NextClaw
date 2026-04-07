# Contributing to CEOClaw

Thank you for your interest in contributing to CEOClaw! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

- Rust 1.70+ with 2021 edition
- Node.js 18.14.1+
- Git

### Setting Up

1. Fork and clone the repository
2. Install dependencies:
   ```bash
   npm install
   cd src-tauri
   cargo build
   ```
3. Run development server:
   ```bash
   npm run tauri:dev
   ```

## Code Style

### Rust

- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy`
- Follow 2021 edition conventions
- Document public APIs with `///` doc comments
- Use `#[must_use]` for return values that should be checked

### TypeScript/React

- Use ESLint: `npm run lint`
- Use Prettier for formatting (if configured)
- Use TypeScript strict mode
- Prefer functional components with hooks
- Use `cn()` utility for conditional classes

### Commit Messages

Follow conventional commits:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting)
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks

Example: `feat: add file search functionality`

## Testing

### Rust Tests

```bash
cd src-tauri
cargo test
cargo test --release
```

### TypeScript Tests

```bash
npm test  # when tests are added
npm run lint
```

## Pull Request Process

1. Create a branch from `main` or `develop`
2. Make your changes following the code style guidelines
3. Add tests if applicable
4. Ensure all tests pass
5. Run `cargo clippy` and `npm run lint` - fix any warnings
6. Commit with conventional commit messages
7. Push to your fork
8. Open a pull request

The PR checklist workflow will automatically add a checklist to your PR.

## Project Structure

```
ceo-claw/
├── src/                    # React frontend
│   ├── components/         # React components
│   ├── store/             # Zustand state management
│   ├── types/             # TypeScript types
│   └── utils/             # Utility functions
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── ai/            # Claude API integration
│   │   ├── commands/      # Tauri command handlers
│   │   ├── db/            # DuckDB database
│   │   ├── exec/          # Command execution
│   │   └── utils/         # Shared utilities
│   └── tauri.conf.json    # Tauri configuration
```

## Adding Features

### Adding a New Tauri Command

1. Create function in appropriate `commands/*.rs` with `#[tauri::command]` macro
2. Register in `lib.rs` invoke_handler
3. Add TypeScript types in `types/index.ts`
4. Call from frontend using `invoke('command_name', { args })`
5. Add error handling and tests

### Adding a New UI Component

1. Create in `src/components/`
2. Use Tailwind CSS for styling
3. Use Lucide React for icons
4. Follow existing component patterns

## Security Considerations

- Never log or serialize API keys
- Validate all user inputs
- Use `secrecy::SecretString` for sensitive data
- Test command execution security
- Review file operation permissions

## Questions?

Feel free to open an issue for questions or discussions!