# ARCHITECTURE.md

这份文件是系统的顶层地图。保持简短，只提供最关键的结构信息，把更深的内容指向其他文档。

## 系统形态

- 产品：Lumina — 照亮你的文件
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
│  │  .manage(ScanStore, CancelStore, ChannelStore)        │ │
│  │  .plugin(tauri_plugin_log, tauri_plugin_opener, ...)   │ │
│  └─────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────┐ │
│  │              commands/ (Tauri 命令层)                 │ │
│  │  scan.rs    │ file_ops.rs │ system.rs               │ │
│  └─────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────┐ │
│  │              scanner.rs (扫描引擎)                    │ │
│  │  BFS 线程 → work_tx → Rayon 线程池 → 结果实时返回    │ │
│  └─────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────┐ │
│  │              ocr/ (OCR 跨平台抽象)                    │ │
│  │  mod.rs (trait) │ macos.rs (Vision) │ api.rs (HTTP)  │ │
│  └─────────────────────────────────────────────────────┘ │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐ │
│  │  config.rs   │  │  types.rs    │  │  scanner.rs   │ │
│  └──────────────┘  └──────────────┘  └───────────────┘ │
└─────────────────────────────────────────────────────────┘
```

## 分层模型

```
Types → Config → OCR → Scanner → Commands → Store → UI
```

| 层 | 职责 | 文件 |
|---|------|------|
| **Types** | 共享数据结构 | `src-tauri/src/types.rs` + `src/types/` |
| **Config** | 应用配置持久化 | `src-tauri/src/config.rs` |
| **OCR** | OCR 跨平台抽象 | `src-tauri/src/ocr/` (trait + macOS Vision + API) |
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
- **禁止：** 对每个工作项 `std::thread::spawn`（会导致线程爆炸，系统崩溃）。必须使用线程池（rayon）或固定数量的工作线程

## 核心数据流

### 扫描流程

```
用户点击搜索 → store.startScan() → invoke("start_scan")
→ 创建 work_tx/work_rx 通道，存入 ChannelStore
→ scanner::scan_directory()
  → BFS 线程：遍历目录树
    ≤ 阈值目录 → work_tx → 搜索工作协程 → 匹配文件 → 结果实时写入 ScanStore
    > 阈值目录 → 前端确认面板 → 用户允许 → work_tx 重新注入
  → 前端轮询 get_scan_progress → 实时更新 UI
```

### 大目录确认流程

```
BFS 遇到大目录 → count_entries_fast() > threshold
→ on_confirmation_needed() → 写入 ScanStore.pending_confirmations
→ 前端轮询检测 → 系统通知 + 自动弹出确认面板
→ 用户点击"允许" → respond_confirmation → work_tx.send(DirWork)
→ 搜索工作协程自动处理 → 结果实时展示
```

详细流程图见 `docs/scan-flow.md`

## 当前热点

- 搜索逻辑已重构为 BFS + 搜索工作协程并发架构
- 2026-05-17: 修复多个关键bug（根目录搜索、重复结果、进度条卡住、格式计数）
- 2026-05-18: 删除暂停机制，简化代码结构
- 2026-05-18: 新增文档内容提取（docx/xlsx/pptx/pdf），可配置开关，默认开启
- 2026-05-18: 文件预览统一为 match_type 驱动（内容匹配可预览，文件名匹配不显示预览按钮）
- 2026-05-20: 跨平台兼容优化
  - 使用 `tauri-plugin-opener` 替代平台特定文件操作
  - 使用 `tauri-plugin-log` 添加日志系统 + panic hook
  - OCR 跨平台抽象（trait + macOS Vision + 第三方 API）
  - 系统音频播放全平台支持
- 技术债：dispatch线程使用recv_timeout轮询，未来可考虑事件驱动

## 变更检查

当你修改了会影响架构的代码：

1. 如果领域地图或分层模型变了，更新这份文件
2. 如果涉及扫描逻辑，更新 `docs/scan-flow.md`
3. 有新 bug 记到 `docs/QUALITY.md`
