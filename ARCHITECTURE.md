# CEOClaw 产品架构设计 v2.0

## 核心功能清单（对标 openclaw、clawX、nexu）

### 1. 多 AI 模型支持
- [x] Anthropic Claude (claude-3-opus, claude-3-sonnet, claude-3-haiku)
- [x] OpenAI GPT (gpt-4, gpt-4-turbo, gpt-3.5-turbo)
- [x] Google Gemini (gemini-pro, gemini-pro-vision)
- [ ] 更多模型可扩展

### 2. 通道配置系统
- [ ] 多通道管理（通道 = API 提供商 + 模型 + 配置）
- [ ] 通道优先级和负载均衡
- [ ] 通道健康检测
- [ ] 失败自动切换
- [ ] 通道使用统计

### 3. 插件系统
- [ ] 插件架构（基于 Rust dylib）
- [ ] 插件市场集成
- [ ] 插件热加载/卸载
- [ ] 预设插件：
  - [ ] 命令执行插件
  - [ ] 文件操作插件
  - [ ] 网络请求插件
  - [ ] 代码分析插件
  - [ ] 翻译插件

### 4. 快捷键系统
- [ ] 全局快捷键
- [ ] 会话内快捷键
- [ ] 快捷键自定义
- [ ] 快捷键冲突检测

### 5. 系统托盘集成
- [ ] 系统托盘图标
- [ ] 右键菜单
- [ ] 最小化到托盘
- [ ] 托盘通知

### 6. UI/UX 增强
- [ ] 深色/浅色主题
- [ ] 自定义主题颜色
- [ ] 窗口透明度
- [ ] 毛玻璃效果
- [ ] 动画过渡
- [ ] 响应式布局

### 7. 会话管理
- [ ] 多会话并行
- [ ] 会话分组
- [ ] 会话搜索
- [ ] 会话导出/导入
- [ ] 会话同步（可选）

### 8. 命令执行系统
- [ ] 命令沙盒
- [ ] 命令历史
- [ ] 命令别名
- [ ] 命令验证
- [ ] 危险命令确认

### 9. 文件操作
- [ ] 文件浏览
- [ ] 文件编辑
- [ ] 文件预览
- [ ] 拖拽上传
- [ ] 文件版本管理

### 10. 高级功能
- [ ] 语音输入
- [ ] 语音输出
- [ ] 代码高亮
- [ ] Markdown 支持
- [ ] LaTeX 公式
- [ ] 代码执行

## 技术架构

### 前端架构
```
src/
├── main.tsx              # 应用入口
├── App.tsx               # 根组件
├── components/
│   ├── Sidebar/          # 侧边栏
│   ├── Chat/             # 聊天界面
│   ├── Channel/          # 通道配置
│   ├── Plugin/           # 插件管理
│   ├── Settings/         # 设置面板
│   ├── Tray/             # 系统托盘
│   └── Hotkeys/          # 快捷键配置
├── store/
│   ├── channel.ts        # 通道状态管理
│   ├── plugin.ts         # 插件状态管理
│   ├── session.ts        # 会话状态管理
│   └── settings.ts       # 设置状态管理
├── hooks/
│   ├── useChannel.ts     # 通道 Hook
│   ├── usePlugin.ts      # 插件 Hook
│   ├── useHotkey.ts      # 快捷键 Hook
│   └── useTray.ts        # 托盘 Hook
└── types/
    ├── channel.ts        # 通道类型
    ├── plugin.ts         # 插件类型
    └── config.ts         # 配置类型
```

### 后端架构
```
src-tauri/src/
├── main.rs               # 应用入口
├── lib.rs                # 库入口
├── channels/             # 通道管理
│   ├── mod.rs
│   ├── manager.rs        # 通道管理器
│   ├── claude.rs         # Claude 通道
│   ├── openai.rs         # OpenAI 通道
│   └── gemini.rs         # Gemini 通道
├── plugins/              # 插件系统
│   ├── mod.rs
│   ├── manager.rs        # 插件管理器
│   ├── loader.rs         # 插件加载器
│   └── api.rs            # 插件 API
├── commands/             # Tauri 命令
│   ├── mod.rs
│   ├── channel.rs        # 通道命令
│   ├── plugin.rs         # 插件命令
│   ├── hotkey.rs         # 快捷键命令
│   └── tray.rs           # 托盘命令
├── hotkeys/              # 快捷键系统
│   ├── mod.rs
│   ├── manager.rs        # 快捷键管理器
│   └── registry.rs       # 快捷键注册
├── tray/                 # 系统托盘
│   ├── mod.rs
│   └── manager.rs        # 托盘管理器
├── ui/                   # UI 控制
│   ├── mod.rs
│   ├── theme.rs          # 主题管理
│   └── window.rs         # 窗口管理
└── db/                   # 数据库
    ├── mod.rs
    ├── connection.rs     # 连接管理
    ├── schema.rs         # 数据库架构
    └── models.rs         # 数据模型
```

## 数据库架构

```sql
-- 通道表
CREATE TABLE channels (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    provider TEXT NOT NULL,        -- claude, openai, gemini
    model TEXT NOT NULL,
    api_key TEXT,
    api_base TEXT,
    priority INTEGER DEFAULT 0,
    enabled INTEGER DEFAULT 1,
    health_status TEXT DEFAULT 'unknown',
    last_used INTEGER,
    created_at TEXT,
    updated_at TEXT
);

-- 插件表
CREATE TABLE plugins (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    author TEXT,
    description TEXT,
    enabled INTEGER DEFAULT 1,
    config TEXT,
    installed_at TEXT,
    updated_at TEXT
);

-- 快捷键表
CREATE TABLE hotkeys (
    id TEXT PRIMARY KEY,
    action TEXT NOT NULL,
    key_combination TEXT NOT NULL,
    enabled INTEGER DEFAULT 1,
    created_at TEXT,
    updated_at TEXT
);

-- 主题配置表
CREATE TABLE themes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    mode TEXT NOT NULL,             -- dark, light, auto
    accent_color TEXT,
    window_opacity REAL,
    blur_enabled INTEGER DEFAULT 0,
    custom_css TEXT,
    is_default INTEGER DEFAULT 0,
    created_at TEXT
);
```

## 配置文件结构

```toml
[app]
name = "CEOClaw"
version = "0.1.0"

[ui]
theme = "dark"  # dark, light, auto
accent_color = "#6366f1"
window_opacity = 0.95
blur_enabled = true

[channels]
default_channel = "claude-3-sonnet"
failover_enabled = true
health_check_interval = 300

[plugins]
auto_update = true
plugin_sources = [
    "https://plugins.ceoclaw.com"
]

[hotkeys]
toggle_window = "CmdOrCtrl+Shift+C"
new_chat = "CmdOrCtrl+N"
focus_input = "CmdOrCtrl+L"

[tray]
minimize_to_tray = true
show_notifications = true
```

## 开发计划

### Phase 1: 核心重构（1-2周）
- [ ] 重新设计数据库架构
- [ ] 实现通道管理系统
- [ ] 实现基础插件架构
- [ ] 实现快捷键系统
- [ ] 实现系统托盘

### Phase 2: UI/UX 增强（1-2周）
- [ ] 重新设计主界面
- [ ] 实现主题系统
- [ ] 实现通道配置 UI
- [ ] 实现插件管理 UI
- [ ] 实现快捷键配置 UI

### Phase 3: 高级功能（2-3周）
- [ ] 实现多模型支持
- [ ] 实现插件市场
- [ ] 实现语音功能
- [ ] 实现代码执行
- [ ] 实现文件版本管理

### Phase 4: 测试和优化（1周）
- [ ] 全面功能测试
- [ ] 性能优化
- [ ] 用户体验优化
- [ ] 文档编写

## 对标竞品功能矩阵

| 功能 | OpenClaw | ClawX | Nexu | CEOClaw |
|------|----------|-------|------|---------|
| 多模型支持 | ✅ | ✅ | ✅ | 🔄 |
| 插件系统 | ✅ | ✅ | ✅ | 🔄 |
| 通道配置 | ✅ | ✅ | ✅ | 🔄 |
| 快捷键 | ✅ | ✅ | ✅ | 🔄 |
| 系统托盘 | ✅ | ✅ | ✅ | 🔄 |
| 主题切换 | ✅ | ✅ | ✅ | 🔄 |
| 跨平台 | ✅ | ✅ | ✅ | ✅ |
| 原生性能 | ❌ | ❌ | ❌ | ✅ |

✅ 已实现 | 🔄 进行中 | ❌ 未实现

**CEOClaw 核心优势：**
1. Rust 原生性能（相比 Electron 更轻更快）
2. 更好的资源占用
3. 更强的安全性
4. 更丰富的功能
5. 更现代的 UI/UX