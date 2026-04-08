# CEOClaw - Rust + Tauri 实现方案

## 目标

用 Rust + Tauri 技术栈 100% 复制 ClawX 的所有功能。

## 技术栈

**后端（Rust）**:
- Tauri 2.0
- Tokio（异步运行时）
- Reqwest（HTTP 客户端）
- Serde（序列化）
- anyhow（错误处理）
- DuckDB（数据库）
- 密钥管理（系统密钥链）

**前端（TypeScript）**:
- React 19
- Zustand（状态管理）
- Tailwind CSS
- shadcn/ui（组件库）

---

## 核心模块

### 1. 聊天模块（Chat）

**前端**:
- `src/pages/Chat.tsx` - 聊天界面
- `src/components/chat/MessageBubble.tsx` - 消息气泡
- `src/components/chat/InputArea.tsx` - 输入区域
- `src/stores/chat.ts` - 聊天状态

**后端（Rust）**:
- `src-tauri/src/chat/mod.rs` - 聊天逻辑
- `src-tauri/src/commands/chat.rs` - Tauri 命令

### 2. Agent 模块

**前端**:
- `src/pages/Agents.tsx` - Agent 管理界面
- `src/components/agents/AgentCard.tsx` - Agent 卡片
- `src/components/agents/AgentForm.tsx` - Agent 表单
- `src/stores/agents.ts` - Agent 状态

**后端（Rust）**:
- `src-tauri/src/agents/mod.rs` - Agent 逻辑
- `src-tauri/src/commands/agents.rs` - Tauri 命令

### 3. Channel 模块

**前端**:
- `src/pages/Channels.tsx` - Channel 管理界面
- `src/components/channels/ChannelCard.tsx` - Channel 卡片
- `src/components/channels/ChannelForm.tsx` - Channel 表单
- `src/stores/channels.ts` - Channel 状态

**后端（Rust）**:
- `src-tauri/src/channels/mod.rs` - Channel 逻辑
- `src-tauri/src/channels/providers/` - 各个 Provider 实现
  - `openai.rs`
  - `anthropic.rs`
  - `moonshot.rs`
  - `custom.rs`

### 4. Cron 模块

**前端**:
- `src/pages/Cron.tsx` - Cron 管理界面
- `src/components/cron/JobCard.tsx` - 任务卡片
- `src/components/cron/CronForm.tsx` - Cron 表单
- `src/stores/cron.rs` - Cron 状态

**后端（Rust）**:
- `src-tauri/src/cron/mod.rs` - Cron 逻辑
- `src-tauri/src/cron/scheduler.rs` - 任务调度器
- `src-tauri/src/commands/cron.rs` - Tauri 命令

### 5. Skills 模块

**前端**:
- `src/pages/Skills.tsx` - Skills 管理界面
- `src/components/skills/SkillCard.tsx` - Skill 卡片
- `src/components/skills/SkillMarket.tsx` - 技能市场
- `src/stores/skills.rs` - Skills 状态

**后端（Rust）**:
- `src-tauri/src/skills/mod.rs` - Skills 逻辑
- `src-tauri/src/skills/manager.rs` - Skills 管理器
- `src-tauri/src/skills/executor.rs` - Skills 执行器
- `src-tauri/src/commands/skills.rs` - Tauri 命令

### 6. Models 模块

**前端**:
- `src/pages/Models.tsx` - Models 管理界面
- `src/components/models/ModelCard.tsx` - Model 卡片
- `src/components/models/ProviderForm.tsx` - Provider 表单
- `src/stores/models.ts` - Models 状态

**后端（Rust）**:
- `src-tauri/src/models/mod.rs` - Models 逻辑
- `src-tauri/src/models/provider.rs` - Provider 抽象
- `src-tauri/src/commands/models.rs` - Tauri 命令

### 7. Settings 模块

**前端**:
- `src/pages/Settings.tsx` - 设置界面
- `src/components/settings/SettingsTabs.tsx` - 设置标签页
- `src/stores/settings.ts` - 设置状态

**后端（Rust）**:
- `src-tauri/src/settings/mod.rs` - 设置逻辑
- `src-tauri/src/settings/secure_storage.rs` - 安全存储
- `src-tauri/src/commands/settings.rs` - Tauri 命令

### 8. Setup 模块

**前端**:
- `src/pages/Setup.tsx` - 设置向导

**后端（Rust）**:
- `src-tauri/src/setup/mod.rs` - 初始化逻辑

---

## 数据模型

### Agent
```rust
pub struct Agent {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
    pub created_at: String,
    pub updated_at: String,
}
```

### Channel
```rust
pub struct Channel {
    pub id: String,
    pub provider_type: ProviderType,
    pub name: String,
    pub config: ChannelConfig,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub enum ProviderType {
    OpenAI,
    Anthropic,
    Moonshot,
    Custom,
}
```

### Message
```rust
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: MessageRole,
    pub content: String,
    pub metadata: Option<MessageMetadata>,
    pub created_at: String,
}

pub enum MessageRole {
    User,
    Assistant,
    System,
}
```

### Cron Job
```rust
pub struct CronJob {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub agent_id: String,
    pub cron_expression: String,
    pub message: String,
    pub enabled: bool,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
```

### Skill
```rust
pub struct Skill {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub enabled: bool,
    pub config: Option<SkillConfig>,
    pub installed_at: String,
}
```

---

## API 集成

### OpenAI
```rust
pub struct OpenAIProvider {
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: String,
}
```

### Anthropic
```rust
pub struct AnthropicProvider {
    pub api_key: String,
    pub model: String,
    pub version: String,
}
```

---

## 实现优先级

### Phase 1: 基础框架
1. ✅ Tauri 项目初始化
2. ⬜ 数据库设计（DuckDB）
3. ⬜ 基础 UI 框架（路由、布局）
4. ⬜ 状态管理（Zustand）

### Phase 2: 核心 Chat
5. ⬜ OpenAI Provider 集成
6. ⬜ Anthropic Provider 集成
7. ⬜ 聊天界面
8. ⬜ 消息存储

### Phase 3: Agents & Channels
9. ⬜ Agent 管理
10. ⬜ Channel 管理
11. ⬜ 多 Provider 支持

### Phase 4: Cron & Skills
12. ⬜ 定时任务调度
13. ⬜ Skills 系统
14. ⬜ Skill 市场

### Phase 5: 高级功能
15. ⬜ 设置系统
16. ⬜ 设置向导
17. ⬜ 系统集成（托盘、通知）
18. ⬜ 国际化

---

## 立即开始

1. 清理当前错误的方向代码
2. 重新设计数据库 Schema
3. 实现 OpenAI Provider
4. 构建聊天界面
5. 逐步实现其他模块