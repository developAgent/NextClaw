# CEOClaw 实现路线图 - 100% clawX 功能

## 阶段 1: 核心功能（立即实现）

### 1.1 屏幕捕获系统
- **文件**: `src-tauri/src/screen/mod.rs`
- **功能**:
  - 全屏截图
  - 区域截图
  - 窗口截图
  - 屏幕监控
  - 多显示器支持

### 1.2 鼠标键盘模拟
- **文件**: `src-tauri/src/input/mod.rs`
- **功能**:
  - 鼠标移动
  - 鼠标点击（左/右/中）
  - 鼠标拖拽
  - 鼠标滚动
  - 键盘按键
  - 文本输入

### 1.3 录制后端完善
- **文件**: `src-tauri/src/recorder/mod.rs` (已有，需要完善)
- **添加**:
  - 钩子系统
  - 事件监听
  - 实时录制

### 1.4 触发系统
- **文件**: `src-tauri/src/trigger/mod.rs`
- **功能**:
  - 时间触发器
  - 快捷键触发器
  - 事件触发器
  - 条件触发器

## 阶段 2: UI 增强（短期实现）

### 2.1 拖拽编辑器
- **文件**: `src/components/WorkflowBuilder.tsx` (已有，需要完善)
- **添加**:
  - 节点拖拽
  - 连线绘制
  - 画布缩放
  - 撤销/重做

### 2.2 屏幕选择器
- **文件**: `src/components/ScreenSelector.tsx`
- **功能**:
  - 区域选择
  - 窗口选择
  - 预览显示

### 2.3 触发器管理
- **文件**: `src/components/TriggerManager.tsx`
- **功能**:
  - 触发器列表
  - 触发器编辑
  - 触发器测试

### 2.4 导入导出
- **文件**: `src/components/ImportExport.tsx`
- **功能**:
  - 格式选择
  - 文件处理
  - 进度显示

## 阶段 3: 高级功能（长期实现）

### 3.1 AI 元素识别
- **文件**: `src-tauri/src/ai/element.rs`
- **功能**:
  - OCR 识别
  - 元素检测
  - 图像匹配
  - 自适应

### 3.2 系统托盘
- **文件**: `src-tauri/src/tray/mod.rs` (已有，需要完善)
- **添加**:
  - 快捷菜单
  - 状态指示
  - 通知显示

### 3.3 全局快捷键
- **文件**: `src-tauri/src/hotkeys/global.rs`
- **功能**:
  - 全局注册
  - 冲突检测
  - 快捷键录制

### 3.4 插件系统
- **文件**: `src-tauri/src/plugin/mod.rs`
- **功能**:
  - 插件加载
  - API 暴露
  - 沙箱执行

---

## 立即开始实现：屏幕捕获系统

### 技术选型
- **Windows**: `windows-rs` crate
- **macOS**: `core-graphics` crate
- **Linux**: `x11` or `wayland` crates

### 依赖添加
```toml
[dependencies]
# Windows
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
    "Win32_Foundation",
    "Win32_Graphics_Gdi",
    "Win32_UI_WindowsAndMessaging",
    "Win32_System_LibraryLoader",
] }

# macOS
[target.'cfg(target_os = "macos")'.dependencies]
core-graphics = "0.23"
cocoa = "0.25"

# Linux
[target.'cfg(target_os = "linux")'.dependencies]
x11 = { version = "2.21", features = ["xlib", "xrandr"] }
```

---

## 立即开始实现：鼠标键盘模拟

### 技术选型
- **Windows**: `enigo` crate
- **macOS**: `core-graphics` crate
- **Linux**: `x11` crate

### 依赖添加
```toml
[dependencies]
enigo = "0.2"
```

---

## 实现优先级

### P0 - 必须实现（核心功能）
1. ✅ 数据库集成
2. ✅ 基础录制系统
3. ✅ 基础回放系统
4. ⬜ 屏幕捕获
5. ⬜ 鼠标键盘模拟
6. ⬜ 拖拽编辑器
7. ⬜ 导入导出

### P1 - 重要功能（增强体验）
1. ⬜ 触发系统
2. ⬜ 全局快捷键
3. ⬜ 系统托盘
4. ⬜ 视觉反馈
5. ⬜ 高级编辑功能

### P2 - 可选功能（锦上添花）
1. ⬜ AI 元素识别
2. ⬜ 插件系统
3. ⬜ 云同步
4. ⬜ 协作功能
5. ⬜ 导出视频

---

## 当前状态

### 已完成
- ✅ 项目架构设计
- ✅ 数据库集成
- ✅ 基础录制系统（数据结构）
- ✅ 基础回放系统（数据结构）
- ✅ 工作流编辑器（UI框架）
- ✅ 宏管理（UI）
- ✅ 设置面板

### 进行中
- ⬜ 屏幕捕获系统
- ⬜ 鼠标键盘模拟
- ⬜ 触发系统

### 待开始
- ⬜ 拖拽编辑器实现
- ⬜ 导入导出功能
- ⬜ AI 元素识别
- ⬜ 系统托盘
- ⬜ 全局快捷键

---

**目标**: 确保 100% 实现 clawX 的所有功能，并在 UI/UX 上超越。