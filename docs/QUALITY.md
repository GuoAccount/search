# QUALITY.md — Bug 清单与优化清单

## 🔴 严重 Bug

### BUG-001: 扫描不产出结果

**状态：** 已修复
**优先级：** P0
**影响：** 核心功能不可用
**修复方案：** 重构搜索逻辑为 BFS + 搜索工作协程并发架构，消除子扫描机制

**症状：**
- 扫描运行后只触发大目录确认事件
- 扫描结果始终为 0
- 即使允许扫描大目录，结果仍为 0

**排查清单：**
- [ ] 确认 `collect_entries_bfs` 返回的 entries 数量
- [ ] 确认 `process_entries` 中文件是否被处理
- [ ] 确认 `on_result` 回调是否被调用
- [ ] 确认 `get_scan_progress` 返回的 results 数量
- [ ] 检查 `file_extensions` 过滤是否过于严格
- [ ] 检查 `should_process_entry` 是否错误跳过文件

**相关文件：**
- `src-tauri/src/scanner.rs` — collect_entries_bfs, should_process_entry
- `src-tauri/src/commands/scan.rs` — start_scan, get_scan_progress
- `src/store/index.ts` — startScan, 轮询逻辑

---

### BUG-002: cancel_scan 不生效

**状态：** 已修复  
**优先级：** P1  
**修复方案：** 添加 CancelStore，cancel_scan 正确设置 should_cancel 标志

---

## 🟡 Bug 清单

### BUG-003: macOS 红绿灯位置在不同包类型下不一致

**状态：** 已修复  
**优先级：** P1  
**影响：** 正式包 UI 对齐问题

**症状：**
- 开发包和测试包左上角红绿灯（窗口控制按钮）位置一致
- 打包成正式包后红绿灯会往下移
- 导致与其他 UI 元素对不齐

**根本原因：**
- GitHub Actions 自动构建环境与本地构建环境差异
- 本地 `pnpm tauri build --bundles dmg` 构建正常
- GitHub Actions 使用 `tauri-apps/tauri-action@action-v0.6.2` 构建异常
- 可能是 tauri-action 版本或构建参数导致的窗口配置差异

**修复方案：**
- 参考 Tauri 2.0 官方文档的 GitHub Actions 配置
- 使用 `tauri-apps/tauri-action@v0` 替代 `action-v0.6.2`
- 简化配置，移除不必要的参数，使用官方推荐的 tagName/releaseName 配置
- 使用 `permissions: write-all` 修复 release 创建权限问题
- 使用 `ncipollo/release-action` 替代 `tauri-action` 创建 release

**排查清单：**
- [x] 检查 `src-tauri/tauri.conf.json` 中 window 相关配置
- [x] 对比 dev 和 production 构建的 window 属性
- [x] 检查是否使用了自定义 titlebar 或 transparent 属性
- [x] 验证 macOS 特定的窗口样式设置
- [x] 检查 tauri-action 版本和构建参数
- [x] 对比本地构建和 GitHub Actions 构建的差异
- [x] 修改 GitHub Actions 构建配置
- [x] 参考 Tauri 2.0 官方文档配置
- [x] 修复 tagName 配置，使用 github.ref_name
- [x] 添加 actions:write 权限修复 release 创建权限问题
- [x] 添加更多权限（issues:write, pull-requests:write, discussions:write）
- [x] 使用 softprops/action-gh-release 替代 tauri-action 创建 release
- [x] 参考其他 Tauri 2.0 项目配置（clash-verge-rev）
- [x] 使用 permissions:write-all 和 tauriScript:pnpm
- [x] 移除 tauriScript:pnpm，使用默认的 pnpm tauri 命令
- [x] 使用 ncipollo/release-action 替代 tauri-action 创建 release

**相关文件：**
- `src-tauri/tauri.conf.json` — window 配置
- `src-tauri/src/lib.rs` — 窗口创建逻辑
- `.github/workflows/release.yml` — GitHub Actions 构建配置

---

### BUG-004: 搜索大目录时内存占用过高且不释放

**状态：** 待分析  
**优先级：** P1  
**影响：** 大目录搜索可能导致内存溢出

**症状：**
- 搜索大文件夹目录时内存增加约 2GB
- 停止搜索后内存仍占用，不下降
- 搜索小文件夹时内存没有明显增加

**可能原因分析：**
1. **BFS 全量收集**：`collect_entries_bfs` 将所有目录条目收集到 Vec 中
2. **结果累积**：ScanStore 持续累积搜索结果，无上限
3. **Rust 内存管理**：Rayon 线程池持有数据引用，阻止释放
4. **前端状态**：Zustand store 持有大量结果对象引用

**优化方向：**
- 流式处理替代全量收集
- 结果分页或虚拟滚动
- 实现内存预算机制（超过阈值暂停收集）
- 完成后主动清理 ScanStore
- 前端结果列表虚拟化

**排查清单：**
- [ ] 使用 `heaptrack` 或 `valgrind` 分析 Rust 堆内存
- [ ] 检查 `scanner.rs` 中 Vec 增长模式
- [ ] 验证 `ChannelStore` 是否正确清理
- [ ] 检查前端 `results` 数组大小增长
- [ ] 测试停止搜索后的内存释放行为

**相关文件：**
- `src-tauri/src/scanner.rs` — collect_entries_bfs, search_directory
- `src-tauri/src/commands/scan.rs` — cancel_scan, ChannelStore 管理
- `src/store/index.ts` — results 状态管理

---

## 🟡 优化清单

### OPT-001: 超长行预览卡死

**状态：** 已修复  
**优先级：** P1  
**问题：** 压缩JS文件单行可达几十万字符，预览时渲染卡死  
**修复方案：** 后端截断每行到200字符（`MAX_LINE_LENGTH` / `MAX_MATCH_CONTEXT`）

**相关文件：**
- `src-tauri/src/commands/file_ops.rs` — read_file_preview
- `src-tauri/src/scanner.rs` — truncate_line

---

### OPT-002: 子扫描结果合并竞态

**问题：** 后端和前端都在合并子扫描结果，可能重复  
**建议：** 统一由后端管理结果合并

### OPT-002: 轮询间隔过短

**问题：** 200ms 轮询频繁 IPC 调用  
**建议：** 改用 Tauri 事件推送机制

### OPT-003: BFS 全量收集内存消耗

**问题：** 超大目录树会消耗大量内存  
**建议：** 改为流式处理

### OPT-004: ScanStore 内存泄漏

**问题：** 完成的扫描不会自动清理  
**建议：** 实现过期清理机制

### OPT-005: 确认面板缺少"记住选择"

**问题：** remember 参数未使用  
**建议：** 添加复选框让用户选择

### OPT-006: 跨平台兼容优化

**状态：** 已完成  
**优先级：** P1  
**问题：** 与 macOS 高度耦合，Windows 搜索闪退无日志  

**修复方案：**
1. 文件操作跨平台：使用 `tauri-plugin-opener` 替代 `#[cfg(target_os)]` + `std::process::Command`
2. 日志系统：使用 `tauri-plugin-log`，支持文件输出 + rotation + panic hook
3. OCR 跨平台：抽象 `OcrProvider` trait，支持 macOS Vision 和第三方 API
4. 音频播放：`play_system_sound` 添加 Windows/Linux 支持

**相关文件：**
- `src-tauri/src/commands/file_ops.rs` — reveal_in_finder (重构)
- `src-tauri/src/commands/system.rs` — open_config_file, play_system_sound (重构)
- `src-tauri/src/ocr/` — OCR 跨平台抽象 (新增)
- `src-tauri/src/config.rs` — OcrSettings (新增)
- `src-tauri/src/lib.rs` — 日志插件注册
- `src-tauri/src/main.rs` — panic hook

---

## 🟢 技术债

- 文件预览组件样式待完善
- 动态导入 `@tauri-apps/api/core` 应统一为静态导入
- scanner.rs 中 ScanType 枚举未使用（已改为字符串比较）
