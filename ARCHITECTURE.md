# ARCHITECTURE.md

这份文件是系统的顶层地图。保持简短，只提供最关键的结构信息，把更深的内容指向其他文档。

## 系统形态

- 产品：FileScope
- 主用户流程：选择目录 → 输入关键字 → 搜索文件 → 查看结果 → 操作文件
- 运行面：desktop (Tauri 2)
- 产品行为详情：`docs/scan-flow.md`

## 领域地图

```
┌─────────────────────────────────────────────────────────┐
│                    Frontend (React)                      │
│  ┌───────────┐  ┌───────────┐  ┌──────────────────────┐ │
│  │  Sidebar   │  │ SearchBar │  │    ResultsView       │ │
│  │  Preset    │  │  搜索控制  │  │  Toolbar + Tree      │ │
│  │  Selector  │  │           │  │                      │ │
│  └───────────┘  └───────────┘  └──────────────────────┘ │
│  ┌──────────────────────────────────────────────────────┐│
│  │              Zustand Store (状态管理)                  ││
│  │  settings / scanProgress / appConfig / UI state       ││
│  └──────────────────────────────────────────────────────┘│
└────────────────────────┬────────────────────────────────┘
                         │ invoke (Tauri IPC)
┌────────────────────────┴────────────────────────────────┐
│                    Backend (Rust)                         │
│  ┌─────────────────────────────────────────────────────┐ │
│  │              lib.rs (Tauri 构建入口)                  │ │
│  │  .manage(ScanStore, PauseStore, CancelStore)         │ │
│  └─────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────┐ │
│  │              commands/ (Tauri 命令层)                 │ │
│  │  scan.rs    │ file_ops.rs │ system.rs               │ │
│  └─────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────┐ │
│  │              scanner.rs (扫描引擎)                    │ │
│  │  BFS 遍历 → Rayon 并行处理 → 回调通知                 │ │
│  └─────────────────────────────────────────────────────┘ │
│  ┌──────────────┐  ┌───────────────────────────────────┐ │
│  │  config.rs   │  │  types.rs (共享数据结构)            │ │
│  └──────────────┘  └───────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

## 分层模型

```
Types → Config → Scanner → Commands → Store → UI
```

| 层 | 职责 | 文件 |
|---|------|------|
| **Types** | 共享数据结构 | `src-tauri/src/types.rs` + `src/types/` |
| **Config** | 应用配置持久化 | `src-tauri/src/config.rs` |
| **Scanner** | 文件扫描、匹配算法 | `src-tauri/src/scanner.rs` |
| **Commands** | Tauri IPC 命令处理 | `src-tauri/src/commands/` |
| **Store** | 全局状态管理、业务逻辑 | `src/store/index.ts` |
| **UI** | 组件渲染、用户交互 | `src/components/` |

## 硬性依赖规则

- UI 层只依赖 Store（通过 Zustand）
- Store 通过 `invoke()` 调用 Commands
- Commands 依赖 Scanner、Config、Types
- Scanner 只依赖 Types，不依赖 Commands
- Types 无依赖，纯数据定义
- **禁止：** UI 层直接调用 Tauri 命令

## 核心数据流

### 扫描流程

```
用户点击搜索 → store.startScan() → invoke("start_scan")
→ scanner::scan_directory_with_callback()
  → collect_entries_bfs()    // 收集文件
  → rayon::par_chunks()      // 并行匹配
  → callback.on_result()     // 写入 ScanStore
→ 前端轮询 get_scan_progress → 更新 UI
```

### 大目录确认流程

```
BFS 遇到大目录 → should_process_entry() 返回 false
→ callback.on_confirmation_needed()
→ 前端轮询检测 → 系统通知 + 自动弹出确认面板
→ 用户点击"允许" → scan_sub_directory() → 结果合并
```

详细流程图见 `docs/scan-flow.md`

## 当前热点

- **BUG-001：扫描不产出结果** — 见 `docs/QUALITY.md`
- 模块化重构已完成，待验证功能完整性

## 变更检查

当你修改了会影响架构的代码：

1. 如果领域地图或分层模型变了，更新这份文件
2. 如果涉及扫描逻辑，更新 `docs/scan-flow.md`
3. 有新 bug 记到 `docs/QUALITY.md`
