# 代码清理与优化 - 2026-05-18

## 目标

删除暂停机制，重构代码，优化预览性能，添加关键字高亮和配置项。

## 范围

- [x] 删除暂停机制（PauseStore、pause_scan、resume_scan、check_pause）
- [x] 修复超长行预览卡死问题（截取关键字前后各100字符）
- [x] 修复UTF-8中文截断panic
- [x] 搜索结果和预览中关键字标红显示
- [x] 添加匹配上下文长度配置（设置 → 显示）
- [x] 优化系统通知逻辑（60秒节流，显示总数）
- [x] 取消待确认面板自动弹出

## 验证路径

- [x] `cargo check` 编译通过，无警告
- [x] `pnpm run build` 编译通过
- [x] 搜索压缩JS文件不卡死
- [x] 中文关键字截取不panic

## 涉及文件

| 文件 | 改动 |
|------|------|
| `src-tauri/src/scanner.rs` | 删除should_pause、check_pause；添加extract_context |
| `src-tauri/src/commands/scan.rs` | 删除pause_scan、resume_scan |
| `src-tauri/src/commands/file_ops.rs` | read_file_preview添加keyword、context_length参数 |
| `src-tauri/src/types.rs` | 删除PauseStore |
| `src-tauri/src/lib.rs` | 移除PauseStore注册 |
| `src-tauri/src/config.rs` | DisplaySettings添加match_context_length |
| `src/store/index.ts` | 删除pauseScan、resumeScan |
| `src/types/index.ts` | ScanProgress.status移除paused；AppConfig添加match_context_length |
| `src/components/SearchBar/SearchBar.tsx` | 删除暂停/恢复按钮 |
| `src/components/ResultsView/ResultsTree.tsx` | 添加关键字高亮 |
| `src/components/modals/FilePreviewModal.tsx` | 添加关键字高亮 |
| `src/components/modals/SettingsPanel.tsx` | 添加匹配上下文长度配置项 |
| `src/App.tsx` | 优化通知逻辑，取消自动弹出确认面板 |

## 完成状态

已完成，可收工。
