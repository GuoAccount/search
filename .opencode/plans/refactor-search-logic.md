# 重构搜索逻辑

**状态：** 已完成
**优先级：** P0
**开始时间：** 2026-05-17

## 目标

将扫描引擎从"先收集后处理"改为"BFS 与搜索并发"架构，消除子扫描机制，实现结果实时展示。

## 当前架构问题

1. **串行瓶颈**：`collect_entries_bfs()` 先收集 ALL 文件路径到 Vec，然后才开始并行处理。大目录树下 BFS 阶段耗时长，用户看不到任何结果。
2. **子扫描机制复杂**：用户确认大目录后，前端需要启动子扫描、轮询子扫描进度、合并子扫描结果（约 80 行复杂逻辑）。
3. **结果不能实时展示**：BFS 阶段没有结果产出，只有 BFS 完成后 rayon 才开始处理。

## 新架构设计

### 线程模型

```
BFS Thread (std::thread)
  │
  ├── work_tx ──> work_rx ──> Tokio spawn_blocking 搜索工作池
  │                              │
  │                              ├── result_tx ──> result_rx ──> Result Handler Thread
  │                              └── progress_tx ──> progress_rx ──> Progress Handler Thread
  │
  └── confirm_tx ──> confirm_rx ──> Confirmation Handler Thread
                                        │
                                        └── 用户确认后 ──> work_tx (重新注入搜索通道)
```

### 核心数据流

```
BFS 遍历目录树
  │
  ├─ 目录 ≤ 阈值 ──> work_tx 发送目录路径 ──> 搜索工作协程处理该目录下文件
  │                                               ├─ 匹配结果 ──> result_tx ──> 实时展示
  │                                               └─ 进度更新 ──> progress_tx
  │
  ├─ 目录 > 阈值 且 ask_on_large_dir=true ──> confirm_tx ──> 前端确认面板
  │                                               └─ 用户允许 ──> work_tx (重新注入)
  │
  └─ 目录 > 阈值 且 ask_on_large_dir=false ──> 跳过，记录 skipped_dirs
```

### 关键设计决策

| 决策 | 选择 | 原因 |
|------|------|------|
| 搜索并发模型 | Tokio spawn_blocking | 与 Tauri runtime 集成自然 |
| 嵌套大目录策略 | 继续检查阈值 | 子目录仍可能超阈值 |
| 工作项粒度 | 仅目录路径 | BFS 只发路径，搜索工作协程自己 list 文件 |

## 涉及文件

| 文件 | 改动类型 | 说明 |
|------|----------|------|
| `src-tauri/src/scanner.rs` | **重写** | BFS 线程 + 搜索工作函数 |
| `src-tauri/src/commands/scan.rs` | **重写** | 通道管理，消除 scan_sub_directory |
| `src-tauri/src/types.rs` | **小改** | 新增 DirWork 结构体 |
| `src-tauri/src/lib.rs` | **小改** | 注册 ChannelStore |
| `src/store/index.ts` | **简化** | 移除子扫描逻辑（删约 80 行） |
| `docs/scan-flow.md` | **重写** | 更新流程文档 |
| `docs/QUALITY.md` | **更新** | 标记 BUG-001 已修复 |
| `ARCHITECTURE.md` | **更新** | 更新核心数据流图 |

## 验证路径

1. `cd src-tauri && cargo check` — Rust 编译通过
2. `npm run build` — 前端编译通过
3. 运行应用，扫描有文件的目录 → 结果正常显示
4. 扫描包含大目录的目录 → 确认面板弹出
5. 点击"允许" → 大目录的结果实时出现在列表中
6. 点击"拒绝" → 目录被跳过
7. 取消/暂停/恢复 → 正常工作
