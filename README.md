<div align="center">

![CEOClaw Logo](https://via.placeholder.com/150x150/1a1a1a/FFFFFF?text=CEOClaw)

# CEOClaw

**The Zero-Code AI Assistant for Desktop Automation**

[![Build Status](https://github.com/your-org/ceo-claw/workflows/Build%20CEOClaw/badge.svg)](https://github.com/your-org/ceo-claw/actions)
[![Code Quality](https://github.com/your-org/ceo-claw/workflows/Code%20Quality/badge.svg)](https://github.com/your-org/ceo-claw/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue.svg)](https://github.com/tauri-apps/tauri)

</div>

---

## 🚀 What is CEOClaw?

CEOClaw is a cross-platform desktop AI assistant that empowers non-technical users to accomplish complex tasks through natural language conversations. Built with Rust and React, CEOClaw provides a secure, efficient, and intuitive interface for AI-powered automation.

### ✨ Key Features

| Feature | Description |
|---------|-------------|
| 🤖 **AI-Powered Conversations** | Intelligent interactions powered by Anthropic Claude models |
| 💻 **Command Execution** | Safely execute shell commands with comprehensive security validations |
| 📁 **File Operations** | Browse, read, analyze, and write files through AI assistance |
| 🗣️ **Multi-Session Management** | Multiple chat sessions with persistent conversation history |
| 🔒 **Secure Storage** | Encrypted API key storage in local DuckDB database |
| 🌙 **Modern Dark Theme** | Eye-friendly interface optimized for extended use |
| ⚡ **Blazing Fast** | Rust backend ensures lightning-fast performance |
| 🌍 **Cross-Platform** | Native builds for Windows, macOS, and Linux |

---

## 🖼️ Screenshots

<div align="center">
  <img src="https://via.placeholder.com/800x500/1a1a1a/FFFFFF?text=Chat+Interface" alt="Chat Interface" width="45%" />
  <img src="https://via.placeholder.com/800x500/1a1a1a/FFFFFF?text=Settings+Panel" alt="Settings Panel" width="45%" />
</div>

---

## 🛠️ Tech Stack

### Backend (Rust 2021)

```
Tauri 2.0       ──  Cross-platform desktop framework
DuckDB          ──  High-performance embedded database
Tokio           ──  Async runtime for concurrent operations
Tracing         ──  Structured logging and telemetry
Anthropic SDK   ──  Claude API integration
Secrecy         ──  Secure secret handling
Shlex           ──  Shell command parsing
```

### Frontend (TypeScript + React)

```
React 18        ──  Modern UI with hooks and concurrent features
TypeScript      ──  Type-safe development
Vite            ──  Lightning-fast build tool
Tailwind CSS    ──  Utility-first styling
Zustand         ──  Lightweight state management
Lucide React    ──  Beautiful icon library
```

---

## 📋 Prerequisites

- **Rust**: 1.70 or higher (2021 edition)
  ```bash
  # Install Rust via rustup
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh  # macOS/Linux
  # Download rustup-init.exe from https://rustup.rs/           # Windows
  ```

- **Node.js**: 18.14.1 or higher
  ```bash
  # Install Node.js
  # Download from https://nodejs.org/
  ```

- **Platform-Specific Tools**:

  **Linux:**
  ```bash
  sudo apt-get install libwebkit2gtk-4.0-dev build-essential \
    curl wget file libssl-dev libayatana-appindicator3-dev librsvg2-dev
  ```

  **macOS:**
  ```bash
  xcode-select --install  # Xcode Command Line Tools
  ```

  **Windows:**
  - Microsoft C++ Build Tools
  - WebView2 (usually included with Windows 10+)

---

## 🚦 Quick Start

### Installation

```bash
# 1. Clone the repository
git clone https://github.com/your-org/ceo-claw.git
cd ceo-claw

# 2. Install dependencies
npm install

# 3. Start development server
npm run tauri:dev
```

The app will open automatically with hot reload enabled.

### Building for Production

```bash
# Build for your current platform
npm run tauri:build

# Build for specific platforms
npm run tauri:build -- --target x86_64-pc-windows-msvc      # Windows x64
npm run tauri:build -- --target aarch64-apple-darwin       # macOS ARM64
npm run tauri:build -- --target x86_64-apple-darwin        # macOS Intel
npm run tauri:build -- --target x86_64-unknown-linux-gnu   # Linux x64
```

Built artifacts will be in `src-tauri/target/release/bundle/`.

---

## 📖 Usage Guide

### First-Time Setup

1. **Launch CEOClaw** after installation
2. **Configure API Key**:
   - Press `Ctrl+,` (or `Cmd+,` on macOS) to open Settings
   - Enter your Anthropic API key (`sk-ant-...`)
   - Click "Save API Key"
3. **Select Model**: Choose between Claude Sonnet, Opus, or Haiku
4. **Start Chatting**: Create a new session and begin!

### Example Conversations

```
You: List all files in the current directory
CEOClaw: [Executes ls -la]
CEOClaw: Here are the files in your current directory:
        drwxr-xr-x  user  staff  128 Apr  7 10:00 .
        drwxr-xr-x  user  staff   64 Apr  7 09:00 ..
        -rw-r--r--  user  staff  256 Apr  7 10:00 file.txt

You: Read the contents of file.txt
CEOClaw: [Reads file and displays content]
CEOClaw: The file contains: "Hello, World!"
```

### Security Features

- ✅ **Command Validation**: Whitelist/blacklist enforcement
- ✅ **Confirmation Prompts**: Dangerous commands require approval
- ✅ **Path Sandboxing**: Restrict file operations to safe directories
- ✅ **Timeout Protection**: Commands automatically terminate after timeout
- ✅ **Audit Logging**: All actions logged for review

---

## ⚙️ Configuration

### Application Settings

Settings are stored in platform-specific locations:

| Platform | Config Path |
|----------|-------------|
| Linux | `~/.config/CEOClaw/config.toml` |
| macOS | `~/Library/Application Support/com.ceoclaw.app/config.toml` |
| Windows | `%APPDATA%\CEOClaw\config.toml` |

### Configuration Options

```toml
[api]
claude_model = "claude-3-sonnet-20240229"
request_timeout_secs = 120
max_retries = 3

[commands]
timeout_secs = 300
allow_shell = true
whitelist = ["ls", "cat", "echo"]
blacklist = ["rm -rf /", "dd if=/dev/zero"]
sandbox_path = "/safe/directory"
require_confirmation = true

[ui]
theme = "dark"
font_size = 14
show_timestamps = true
max_history = 1000
```

---

## 🔒 Security Architecture

CEOClaw implements defense-in-depth security:

```
┌─────────────────────────────────────────────┐
│              Application Layer              │
├─────────────────────────────────────────────┤
│  Command Whitelist/Blacklist Validation     │
│  Path Sandbox Enforcement                   │
│  User Confirmation Prompts                  │
├─────────────────────────────────────────────┤
│              Rust Runtime Layer             │
├─────────────────────────────────────────────┤
│  Memory Safety (Rust guarantees)            │
│  Type Safety (Rust 2021 edition)            │
│  No Unsafe Code (`#![forbid(unsafe_code)]`) │
├─────────────────────────────────────────────┤
│              Data Storage Layer             │
├─────────────────────────────────────────────┤
│  Encrypted API Keys (secrecy crate)         │
│  Local DuckDB Database                     │
│  No Cloud Data Transmission                │
└─────────────────────────────────────────────┘
```

### Security Guarantees

- 🛡️ **No data leaves your machine** - All processing happens locally (except API calls to Claude)
- 🔑 **API keys encrypted** - Stored using secure encryption
- 🚫 **No unsafe Rust code** - Memory safety guaranteed
- 📝 **Full audit trail** - All commands logged
- ⏱️ **Timeout protection** - Prevents runaway commands

---

## 🧪 Development

### Project Structure

```
ceo-claw/
├── .github/workflows/      # CI/CD configurations
├── src/                    # React frontend
│   ├── components/         # UI components
│   │   ├── ChatInterface.tsx
│   │   ├── Sidebar.tsx
│   │   ├── MessageBubble.tsx
│   │   ├── InputArea.tsx
│   │   ├── CommandResult.tsx
│   │   └── SettingsModal.tsx
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
├── package.json           # Node dependencies
└── README.md              # This file
```

### Available Scripts

```bash
npm run tauri:dev      # Start development server
npm run tauri:build    # Build for production
npm run dev            # Start Vite dev server only
npm run build          # Build frontend only
npm run lint           # Run ESLint
npm run lint:fix       # Auto-fix ESLint issues
```

### Rust Development

```bash
cd src-tauri

# Run tests
cargo test

# Check code
cargo clippy

# Format code
cargo fmt

# Build debug
cargo build

# Build release
cargo build --release
```

---

## 🤝 Contributing

We welcome contributions! Please read our [Contributing Guidelines](CONTRIBUTING.md) before submitting PRs.

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test` and `npm run lint`)
5. Commit with conventional commits (`feat: add amazing feature`)
6. Push to your fork (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Code Style

- **Rust**: Follow `cargo fmt` and `cargo clippy` guidelines
- **TypeScript**: Follow ESLint rules
- **Commits**: Use [Conventional Commits](https://www.conventionalcommits.org/) format

---

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- [Tauri](https://tauri.app/) - For the amazing desktop framework
- [Anthropic](https://www.anthropic.com/) - For Claude AI
- [DuckDB](https://duckdb.org/) - For the embedded database
- All contributors who help make CEOClaw better

---

## 📞 Support

- 🐛 [Report Issues](https://github.com/your-org/ceo-claw/issues)
- 💡 [Feature Requests](https://github.com/your-org/ceo-claw/discussions)
- 📖 [Documentation](https://github.com/your-org/ceo-claw/wiki)
- 💬 [Discussions](https://github.com/your-org/ceo-claw/discussions)

---

<div align="center">

**Built with ❤️ using Rust, Tauri, and React**

[⬆ Back to Top](#ceoclaw)

</div>