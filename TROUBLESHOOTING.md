# CEOClaw 安装包使用指南

## 📦 安装包位置

### NSIS 安装程序（推荐）
```
E:\project\NextClaw\target\release\bundle\nsis\CEOClaw_0.1.0_x64-setup.exe
```

### MSI 安装包
```
E:\project\NextClaw\target\release\bundle\msi\CEOClaw_0.1.0_x64_en-US.msi
```

## 🔧 故障排除

### 如果安装后点击没反应

1. **检查应用是否运行**
   - 打开任务管理器，查看是否有 `ceo-claw.exe` 进程
   - 如果有进程在运行，请结束进程后重新启动

2. **检查日志文件**
   - 日志位置: `C:\Users\<用户名>\AppData\Roaming\ceoclaw\CEOClaw\logs\ceo-claw.log`
   - 如果有错误信息，请查看并反馈

3. **清除旧数据**
   - 如果之前的安装留下了旧的数据库文件，可能会导致启动问题
   - 数据库位置: `C:\Users\<用户名>\AppData\Roaming\ceoclaw\CEOClaw\data\ceo-claw.db`
   - 删除 `C:\Users\<用户名>\AppData\Roaming\ceoclaw` 整个目录
   - 重新启动应用，会自动创建新的数据库
   - **注意**: 数据库版本不兼容会导致 "Failed to create sessions index" 错误

4. **直接运行可执行文件**
   - 位置: `E:\project\NextClaw\target\release\ceo-claw.exe`
   - 双击直接运行，查看是否有错误提示

5. **检查端口占用**
   - Tauri 应用可能需要特定端口
   - 检查防火墙是否阻止了应用运行

### 如果窗口显示空白

1. **打开开发者工具**
   - 如果是开发版本，按 `F12` 打开开发者工具
   - 查看控制台是否有 JavaScript 错误

2. **检查前端资源**
   - 确保 `dist` 目录中包含所有必要的文件
   - 检查 `index.html` 是否正确加载

### 如果应用崩溃

1. **查看 Windows 事件日志**
   - 打开"事件查看器"
   - 查看"Windows 日志" → "应用程序"
   - 搜索 "CEOClaw" 相关的错误

2. **重新安装**
   - 卸载现有版本
   - 清理残留文件
   - 重新安装

## 🎯 功能测试

### 基本功能测试
1. 启动应用 - 窗口应该正常显示
2. 导航切换 - Chat 和 Settings 标签页应该可以切换
3. 窗口操作 - 最小化、最大化、关闭按钮应该正常

### 高级功能测试
1. Chat 功能 - 可以输入消息并查看回复
2. Settings - 可以查看和修改设置
3. API 配置 - 可以添加和测试 API 密钥

## 📞 获取帮助

如果问题仍然存在，请提供以下信息：
1. 错误提示信息
2. 日志文件内容
3. 操作系统版本
4. 截图或错误描述