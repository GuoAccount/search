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

## 🟡 优化清单

### OPT-001: 子扫描结果合并竞态

**问题：** 后端和前端都在合并子扫描结果，可能重复  
**建议：** 统一由后端管理结果合并

### OPT-002: 轮询间隔过短

**问题：** 200ms 轮询频繁 IPC 调用  
**建议：** 改用 Tauri 事件推送机制

### OPT-003: BFS 全量收集内存消耗

**问题：** 超大目录树会消耗大量内存  
**建议：** 改为流式处理

### OPT-004: 暂停使用忙等待

**问题：** while + sleep(100ms) 消耗 CPU  
**建议：** 使用 Condvar

### OPT-005: ScanStore 内存泄漏

**问题：** 完成的扫描不会自动清理  
**建议：** 实现过期清理机制

### OPT-006: 确认面板缺少"记住选择"

**问题：** remember 参数未使用  
**建议：** 添加复选框让用户选择

---

## 🟢 技术债

- 所有组件的 CSS Modules 待完善（部分组件样式不完整）
- 动态导入 `@tauri-apps/api/core` 应统一为静态导入
- scanner.rs 中 ScanType 枚举未使用（已改为字符串比较）
