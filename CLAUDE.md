# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CEOClaw (NextClaw) 是一个跨平台桌面应用程序，结合了 AI 助手和自动化功能。使用 Rust (Tauri 2.0) 和 React/TypeScript 构建，支持：
- 多 AI 提供商集成（Anthropic Claude, OpenAI, Ollama）
- 屏幕录制和自动化回放
- 基于 WASM 的可扩展技能系统
- 对话会话管理和代理配置
- 定时任务、热键支持、系统托盘集成

## 开发命令

```bash
# 启动 Tauri 开发服务器（前端 + 后端）
npm run tauri:dev

# 构建生产版本
npm run tauri:build

# 仅开发前端
npm run dev          # 仅启动 Vite 开发服务器
npm run build        # 仅构建前端
npm run preview      # 预览前端构建

# 代码质量检查
npm run lint         # ESLint 检查
npm run lint:fix     # ESLint 自动修复

# Rust 开发（在 src-tauri 目录下）
cd src-tauri
cargo test           # 运行测试
cargo clippy         # 代码检查
cargo fmt            # 格式化代码
cargo build          # 构建 debug 版本
cargo build --release # 构建 release 版本
```

## 系统要求

- Rust 1.70+ (2021 edition)
- Node.js >=18.14.1
- 平台特定工具：
  - **Linux**: libwebkit2gtk-4.0-dev, build-essential, libssl-dev, libayatana-appindicator3-dev, librsvg2-dev, OpenCV 4
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Microsoft C++ Build Tools, WebView2

## 项目架构

### 核心模块结构

```
src-tauri/src/
├── ai/                 # AI 提供商集成（client, parser, prompt）
├── agents/             # AI 代理管理
├── channels/           # 频道系统（多账户支持）
├── commands/           # Tauri 命令处理器
│   ├── agents.rs       # 代理 CRUD 操作
│   ├── chat.rs         # 聊天会话管理
│   ├── anthropic.rs    # Anthropic API 集成
│   ├── ollama.rs       # Ollama 本地模型集成
│   ├── cron.rs         # 定时任务
│   ├── hotkey.rs       # 全局热键
│   ├── wasm.rs         # WASM 技能执行
│   └── ...
├── db/                 # SQLite 数据库层
│   ├── connection.rs   # 数据库连接和 schema 初始化
│   ├── models.rs       # 数据模型
│   └── schema.rs       # Schema 定义
├── skills/             # WASM 技能系统
│   ├── host.rs         # WASM 宿主环境
│   ├── runtime.rs      # WASM 运行时
│   ├── sandbox.rs      # 沙箱安全
│   └── permissions.rs  # 权限管理
├── providers/          # AI 提供商实现
│   ├── anthropic.rs
│   └── openai.rs
├── exec/               # 命令执行引擎
│   ├── shell.rs        # Shell 命令执行
│   └── security.rs     # 安全验证
└── ...
```

### 技术栈

**后端 (Rust 2021):**
- **Tauri 2.0** - 桌面应用框架
- **rusqlite** - SQLite 嵌入式数据库（不是 DuckDB）
- **Tokio** - 异步运行时
- **wasmtime** - WASM 运行时（用于技能系统）
- **Tracing** - 结构化日志
- **secrecy** - API 密钥安全处理
- **opencv** - 计算机视觉（OCR、元素识别）
- **screenshots** - 屏幕捕获
- **inputbot** - 输入模拟（自动化回放）

**前端 (TypeScript):**
- React 18 + Vite
- Tailwind CSS
- Zustand - 状态管理
- Lucide React - 图标库
- react-i18next - 国际化
- react-router-dom - 路由

### 关键设计模式

**Tauri 命令注册:**
1. 在 `src-tauri/src/commands/*.rs` 中创建命令函数，使用 `#[tauri::command]` 宏
2. 在 `src-tauri/src/lib.rs` 的 `invoke_handler!` 中注册命令
3. 命令必须返回 `Result<T>` 以便前端处理错误

**数据库访问:**
- 使用 SQLite (rusqlite)，连接通过 `Arc<tokio::sync::Mutex<Connection>>` 管理
- Schema 在 `db/connection.rs::init_schema()` 中初始化
- 核心表：agents (代理), sessions (会话), messages (消息), channels (频道)
- 所有操作使用预编译语句防止 SQL 注入

**WASM 技能系统:**
- 技能使用 WASM 编写，运行在沙箱环境中
- `skills/host.rs` 提供宿主函数（文件访问、网络请求等）
- `skills/permissions.rs` 控制技能的权限范围
- SDK 位于 `sdk/ceo-claw-sdk/`

**AI 提供商抽象:**
- `providers/` 目录包含不同 AI 提供商的实现
- 支持流式响应（使用 `streaming.rs`）
- API 密钥通过 `secrecy::SecretString` 保护，存储在系统 keyring 中

## 代码规范

**Rust:**
- 2021 edition，强制执行 `#![forbid(unsafe_code)]`
- 启用 Clippy pedantic/nursery lints（部分允许例外）
- 使用 `tracing` 而不是 `println!` 进行日志记录
- 错误处理使用 `utils/error.rs` 中的 `AppError` 枚举

**TypeScript:**
- Strict mode 启用
- 禁止使用 `any`，使用 `unknown` 代替
- 组件使用函数式组件和 Hooks
- 使用 `clsx` 或 `tailwind-merge` 处理动态类名

## 配置文件位置

**平台相关路径:**
- **Linux**: `~/.config/CEOClaw/`, `~/.local/share/CEOClaw/`
- **macOS**: `~/Library/Application Support/com.ceoclaw.app/`
- **Windows**: `%APPDATA%\CEOClaw\`

**关键文件:**
- 配置: `config.toml`
- 数据库: `ceo-claw.db`
- 日志: `logs/ceo-claw.log`

## 常见开发任务

### 添加新的 Tauri 命令
1. 在 `src-tauri/src/commands/` 中创建或编辑相应模块
2. 添加 `#[tauri::command]` 函数
3. 在 `commands/mod.rs` 中导出模块
4. 在 `lib.rs` 的 `invoke_handler!` 中注册命令
5. 从前端使用 `invoke('command_name', { args })` 调用

### 添加新的数据库表
1. 在 `db/connection.rs::init_schema()` 中添加 `CREATE TABLE` 语句
2. 在 `db/models.rs` 中定义对应的 Rust 结构体
3. 实现 CRUD 操作

### 开发 WASM 技能
1. 参考 `sdk/examples/hello-world/` 示例
2. 使用 `ceo-claw-sdk` 创建新技能
3. 编译为 WASM: `cargo build --target wasm32-wasip1 --release`
4. 通过 `commands/wasm.rs` 加载和执行

## 安全注意事项

- API 密钥使用 `secrecy::SecretString` 包装，永远不要直接序列化或打印
- 用户输入在 `exec/security.rs` 中验证后才能执行
- WASM 技能在沙箱中运行，权限受限
- 文件操作需要路径验证，防止目录遍历攻击
- 所有数据库查询使用参数化语句

## 构建和发布

```bash
# 构建当前平台的安装包
npm run tauri:build

# 跨平台构建（需要配置目标平台工具链）
npm run tauri:build -- --target x86_64-pc-windows-msvc      # Windows
npm run tauri:build -- --target x86_64-apple-darwin          # macOS Intel
npm run tauri:build -- --target aarch64-apple-darwin         # macOS Apple Silicon
npm run tauri:build -- --target x86_64-unknown-linux-gnu     # Linux
```

构建产物位于 `src-tauri/target/release/bundle/`