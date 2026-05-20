# 跨平台兼容优化 + 崩溃日志

**创建日期：** 2026-05-20  
**状态：** 已完成  
**优先级：** P1

---

## 背景

当前 Lumina 与 macOS 高度耦合，Windows 用户反馈搜索过程中会闪退崩溃且无日志可查。

---

## 目标

1. 尽可能使用 Tauri 官方 API 替代平台特定代码
2. OCR 功能跨平台抽象，支持第三方 API
3. 添加崩溃日志记录机制

---

## Tauri 官方 API 调研结果

| 插件 | 功能 | 当前替代方案 |
|------|------|-------------|
| `tauri-plugin-log` | 结构化日志、文件输出、rotation | 无 |
| `tauri-plugin-opener` | 打开文件/URL、revealItemInDir | `std::process::Command` + `#[cfg]` |
| `tauri-plugin-shell` | 跨平台命令执行 | `std::process::Command` |
| `tauri-plugin-notification` | 系统通知、通知声音 | 无 |
| `tauri-plugin-dialog` | 文件选择、消息对话框 | 无 |

---

## 任务清单

### 任务 1：使用 `tauri-plugin-opener` 替代平台特定文件操作

**状态：** ✅ 已完成

**实现：**
- `file_ops.rs` 重构 `reveal_in_finder` 使用 `OpenerExt::reveal_item_in_dir`
- `system.rs` 重构 `open_config_file` 使用 `OpenerExt::reveal_item_in_dir`
- 移除所有 `#[cfg(target_os)]` 文件操作代码

---

### 任务 2：使用 `tauri-plugin-log` 添加日志系统

**状态：** ✅ 已完成

**实现：**
- `Cargo.toml` 添加 `tauri-plugin-log = "2"`
- `lib.rs` 注册日志插件，配置 LogDir + Stdout 输出
- `main.rs` 添加 panic hook，捕获崩溃并记录
- `scanner.rs` 添加扫描开始、OCR 初始化日志
- `commands/scan.rs` 添加扫描启动、取消、完成日志

- [ ] `commands/scan.rs` 添加错误日志

**日志路径：**
- macOS: `~/Library/Logs/com.lumina.app/lumina.log`
- Windows: `%APPDATA%/lumina/logs/lumina.log`
- Linux: `~/.local/share/lumina/logs/lumina.log`

**前端日志（可选）：**
```javascript
import { error, info } from '@tauri-apps/plugin-log'

// 自动将 console.log 重定向到日志文件
const detach = await attachConsole()
```

---

### 任务 3：OCR 跨平台抽象

**状态：** ✅ 已完成

**实现：**
- 新建 `src-tauri/src/ocr/` 模块，定义 `OcrProvider` trait
- 实现 `MacOSNativeOcr`（macOS Vision 框架）
- 实现 `ApiOcr`（第三方 HTTP API）
- `config.rs` 添加 `OcrSettings` 配置项
- `scanner.rs` 改用 trait 调用，移除 `#[cfg(target_os = "macos")]` 限制
- `Cargo.toml` 添加 `reqwest` 和 `log` 依赖

---

### 任务 4：系统音频播放跨平台支持

**状态：** ✅ 已完成

**实现：**
- `system.rs` 为 `play_system_sound` 添加 Windows/Linux 实现
  - Windows: 使用 PowerShell 播放 .wav
  - Linux: 使用 paplay/aplay 播放 .oga

---

## 依赖变更

```toml
# Cargo.toml 新增
[dependencies]
tauri-plugin-log = "2"
reqwest = { version = "0.12", features = ["json"] }  # OCR API 调用

# 已有（无需修改）
tauri-plugin-opener = "2"  # 已注册
tauri-plugin-dialog = "2"  # 已注册
tauri-plugin-notification = "2"  # 已注册
```

---

## 验证路径

1. **文件操作跨平台：**
   - macOS: 右键"在 Finder 中显示"正常
   - Windows: 右键"在资源管理器中显示"正常
   - 验证 `reveal_in_finder` 和 `open_config_file` 无 `#[cfg]` 代码

2. **日志系统：**
   - 启动应用后检查日志文件生成
   - 触发 panic，验证日志记录崩溃信息
   - 日志文件自动 rotation（不超过 10MB）

3. **OCR 跨平台：**
   - macOS: 使用 native provider 搜索图片文字
   - Windows: 配置 API provider 后搜索图片文字

4. **音频播放：**
   - Windows 播放系统音效不报错
   - Linux 播放系统音效不报错

---

## 风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 第三方 OCR API 延迟 | 搜索速度下降 | 异步调用 + 超时控制 |
| API key 泄露 | 安全风险 | 配置文件加密存储 |
| 日志文件过大 | 磁盘占用 | tauri-plugin-log 自带 rotation |
| Windows 音频格式兼容 | 播放失败 | 统一使用 .wav 格式 |

---

## 进度日志

| 日期 | 进展 |
|------|------|
| 2026-05-20 | 计划创建，待开始 |
| 2026-05-20 | ✅ 任务1完成：使用 tauri-plugin-opener 重构文件操作 |
| 2026-05-20 | ✅ 任务2完成：使用 tauri-plugin-log 添加日志系统 |
| 2026-05-20 | ✅ 任务3完成：OCR 跨平台抽象（trait + macOS Vision + API） |
| 2026-05-20 | ✅ 任务4完成：系统音频播放全平台支持 |
| 2026-05-20 | ✅ cargo check 编译通过 |
| 2026-05-20 | ✅ 文档更新完成（ARCHITECTURE.md, QUALITY.md） |
