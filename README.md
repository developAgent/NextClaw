<div align="center">

![CEOClaw Logo](https://via.placeholder.com/150x150/1a1a1a/FFFFFF?text=CEOClaw)

# CEOClaw

**The Zero-Code Desktop Automation Platform**

[![Build Status](https://github.com/your-org/ceo-claw/workflows/Build%20CEOClaw/badge.svg)](https://github.com/your-org/ceo-claw/actions)
[![Code Quality](https://github.com/your-org/ceo-claw/workflows/Code%20Quality/badge.svg)](https://github.com/your-org/ceo-claw/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue.svg)](https://github.com/tauri-apps/tauri)

</div>

---

## 🚀 What is CEOClaw?

CEOClaw is a powerful desktop automation platform that empowers users to automate repetitive tasks through visual workflow creation, screen recording, and AI-powered element recognition. Built with Rust and React, CEOClaw provides an intuitive interface for creating and managing automation macros without coding.

**Inspired by clawX, nexu, and openclaw** - CEOClaw builds upon the best features of these automation tools while providing enhanced UI/UX and modern technology.

### ✨ Key Features

| Feature | Description |
|---------|-------------|
| 🎬 **Screen Recording** | Record mouse clicks, keyboard input, and text entry as reusable automation macros |
| 🔧 **Visual Workflow Builder** | Create complex automation flows with an intuitive drag-and-drop node editor |
| 📚 **Macro Library** | Organize, categorize, and reuse your automation scripts |
| ⚡ **Playback Execution** | Replay your recordings with configurable speed and precision |
| 🤖 **AI Element Recognition** | Smart element identification using OCR and computer vision for robust automation |
| ⏰ **Trigger System** | Schedule automations to run on time, events, or keyboard shortcuts |
| 🎯 **Visual Feedback** | See your automations in action with real-time visual indicators |
| 🌍 **Cross-Platform** | Native builds for Windows, macOS, and Linux |
| 🎨 **Modern Dark Theme** | Beautiful interface optimized for extended use |

---

## 🎯 Core Capabilities

### Recording & Replay
- Record mouse clicks, scrolls, drags, and keyboard input
- Capture screenshots at key moments for reference
- Pause, resume, and stop recordings at will
- Replay recordings at adjustable speeds
- Visual playback with real-time mouse tracking

### Visual Workflow Builder
- Drag-and-drop node interface
- Pre-built action nodes: Click, Type, Wait, Scroll, OCR, Loop, Condition, and more
- Visual connection between workflow steps
- Variables and data flow between nodes
- Nested workflows and reusable components

### Macro Management
- Organize macros by category
- Tag-based filtering and search
- Track usage statistics
- Export and share macros
- Import macros from clawX/openclaw formats

### AI-Powered Features
- Optical Character Recognition (OCR) for text-based automation
- Computer vision for UI element detection
- Smart adaptation to UI changes
- Automatic optimization of recorded actions

---

## 🖼️ Screenshots

<div align="center">
  <img src="https://via.placeholder.com/800x500/1a1a1a/FFFFFF?text=Dashboard" alt="Dashboard" width="45%" />
  <img src="https://via.placeholder.com/800x500/1a1a1a/FFFFFF?text=Workflow+Builder" alt="Workflow Builder" width="45%" />
</div>

---

## 🛠️ Tech Stack

### Backend (Rust 2021)

```
Tauri 2.0       ──  Cross-platform desktop framework
DuckDB          ──  High-performance embedded database
Tokio           ──  Async runtime for concurrent operations
Tracing         ──  Structured logging and telemetry
```

### Frontend (TypeScript + React)

```
React 18        ──  Modern UI with hooks and concurrent features
TypeScript      ──  Type-safe development
Vite            ──  Lightning-fast build tool
Tailwind CSS    ──  Utility-first styling
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

### Creating Your First Automation

1. **Launch CEOClaw** after installation
2. **Start Recording**:
   - Click "Record Actions" in the dashboard
   - Enter a name for your automation
   - Perform the actions you want to automate (clicks, typing, etc.)
   - Click "Stop" when finished
3. **Save as Macro**: Your recording is saved and ready to replay
4. **Replay**: Click the play button to execute your automation

### Building Workflows

1. **Open Workflow Builder**: Click "New Workflow" in the dashboard
2. **Add Nodes**: Drag action nodes from the sidebar (Click, Type, Wait, etc.)
3. **Connect Nodes**: Draw connections between nodes to define flow
4. **Configure Properties**: Set parameters for each node in the properties panel
5. **Test and Save**: Test your workflow and save it as a reusable macro

### Using the Macro Library

- Browse all your saved automations
- Search by name or filter by category
- Track usage statistics
- Export macros for sharing

### Example Automations

**Daily Report Generation**:
1. Click on spreadsheet application
2. Type "Ctrl+N" to create new file
3. Enter data from another source
4. Save with date stamp
5. Close application

**Email Processing**:
1. Click email icon
2. Wait for inbox to load
3. Click specific email
4. Extract data using OCR
5. Save to database

---

## ⚙️ Configuration

### Application Settings

Settings are stored in platform-specific locations:

| Platform | Config Path |
|----------|-------------|
| Linux | `~/.config/CEOClaw/config.toml` |
| macOS | `~/Library/Application Support/com.ceoclaw.app/config.toml` |
| Windows | `%APPDATA%\CEOClaw\config.toml` |

### Recording Configuration

```toml
[recording]
capture_screenshots = true
screenshot_frequency = 5
record_mouse_moves = true
max_duration_ms = 3600000
enable_ai_recognition = true
auto_optimize = true
```

### Playback Configuration

```toml
[playback]
default_speed = "normal"
loop_enabled = false
stop_on_error = true
show_visual_feedback = true
```

---

## 🔒 Security Architecture

CEOClaw implements security measures for automation:

```
┌─────────────────────────────────────────────┐
│           Automation Security Layer         │
├─────────────────────────────────────────────┤
│  User Confirmation for Critical Actions     │
│  Sandbox Restrictions for File Access       │
│  Playback Timeout Protection                │
├─────────────────────────────────────────────┤
│              Rust Runtime Layer             │
├─────────────────────────────────────────────┤
│  Memory Safety (Rust guarantees)            │
│  Type Safety (Rust 2021 edition)            │
│  No Unsafe Code (`#![forbid(unsafe_code)]`) │
├─────────────────────────────────────────────┤
│              Data Storage Layer             │
├─────────────────────────────────────────────┤
│  Local DuckDB Database                     │
│  Encrypted Configuration (if needed)       │
│  No Cloud Data Transmission                │
└─────────────────────────────────────────────┘
```

### Security Guarantees

- 🛡️ **No data leaves your machine** - All automation happens locally
- 🚫 **No unsafe Rust code** - Memory safety guaranteed
- 📝 **Full audit trail** - All recordings logged
- ⏱️ **Playback timeout** - Prevents runaway automations

---

## 🧪 Development

### Project Structure

```
ceo-claw/
├── .github/workflows/      # CI/CD configurations
├── src/                    # React frontend
│   ├── components/         # UI components
│   │   ├── Dashboard.tsx           # Main dashboard
│   │   ├── WorkflowBuilder.tsx     # Visual workflow editor
│   │   ├── MacroLibrary.tsx        # Macro management
│   │   ├── Recorder/
│   │   │   └── RecorderControl.tsx # Recording controls
│   │   └── SettingsModal.tsx       # Settings panel
│   ├── types/             # TypeScript type definitions
│   │   ├── index.ts               # Common types
│   │   └── automation.ts          # Automation types
│   └── utils/             # Utility functions
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── recorder/      # Recording system
│   │   ├── workflow/      # Workflow engine
│   │   ├── player/        # Playback execution
│   │   ├── commands/      # Tauri command handlers
│   │   ├── db/            # DuckDB database layer
│   │   └── telemetry/     # Logging setup
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
- [ClawX](https://github.com/clawsoftware/clawx) - For pioneering the AI-powered automation approach
- [OpenClaw](https://github.com/openclaw/openclaw) - For the open-source automation foundation
- [Nexu](https://github.com/nexu-project/nexu) - For innovative automation features
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