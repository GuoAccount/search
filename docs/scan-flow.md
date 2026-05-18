# 搜索/扫描完整逻辑流程文档

## 概述

本项目是一个基于 Tauri 的文件搜索工具，采用前后端分离架构：
- **前端**：React + Zustand 状态管理
- **后端**：Rust + Tauri 命令系统
- **扫描引擎**：BFS 线程遍历 + Rayon 线程池并发搜索

---

## 1. 主扫描流程

### 架构总览

```
BFS 线程 ──> work_tx ──> work_rx ──> Dispatcher 线程 ──> Rayon 线程池
   │                                                       │
   ├─ ≤阈值目录: 发送 DirWork                              ├─ search_directory()
   ├─ >阈值目录: 发送确认请求                              ├─ 结果 → result_tx → Result Handler
   └─ 跳过规则: 记录 skipped_dirs                          └─ 进度 → progress_tx → Progress Handler
```

### 详细步骤

#### 1.1 前端发起扫描

**文件**: `src/store/index.ts` - `startScan()`

1. 获取当前设置（scanPath, keyword, enabledPresets, customExtensions）
2. 验证必要参数（路径、关键字、扩展名不能为空）
3. 构建 `ScanConfig` 对象
4. 设置前端状态：`isScanning: true`
5. 调用 Tauri 命令：`invoke("start_scan", { config })`
6. 启动轮询定时器（200ms 间隔）：`invoke("get_scan_progress", { scanId })`

#### 1.2 后端接收并启动扫描

**文件**: `src-tauri/src/commands/scan.rs` - `start_scan()`

1. 生成唯一 `scan_id`（UUID）
2. 初始化 `ScanProgress` 结构体，存入 `ScanStore`
3. 创建 `should_cancel` 标志位
4. 创建 `work_tx/work_rx` 通道，将 `work_tx` 存入 `ChannelStore`
5. 加载 `AppConfig`
6. 在 `tokio::spawn` 中启动异步任务：
   - 创建 `ScanCallback` 回调结构体
   - 调用 `scanner::scan_directory(config, app_config, callback, work_tx, work_rx)`
   - 扫描完成后从 `ChannelStore` 移除通道，更新状态为 `"completed"`

#### 1.3 扫描引擎执行

**文件**: `src-tauri/src/scanner.rs` - `scan_directory()`

扫描引擎启动 4 个并发组件：

**① BFS 线程**（`bfs_scan()`）：
- 使用 `VecDeque` 做广度优先遍历
- 对每个目录调用 `count_entries_fast()` 统计直接子项
- 分类处理：
  - 隐藏文件/目录 → 跳过
  - 匹配 `exclude_patterns` → 跳过
  - 匹配 `scan_rules` → 强制扫描（最高优先级）
  - 匹配 `skip_rules` → 强制跳过，记录 `skipped_dirs`
  - 子项数 ≤ 阈值 → 发送 `DirWork` 到 `work_tx`，继续 BFS
  - 子项数 > 阈值 且 `ask_on_large_dir=true` → 发送 `PendingConfirmation`
  - 子项数 > 阈值 且 `ask_on_large_dir=false` → 跳过

**② Dispatcher 线程**：
- 从 `work_rx` 读取 `DirWork`
- 使用 `rayon::spawn()` 分发到 Rayon 线程池
- 跟踪活跃任务数，任务完成时通知

**③ Rayon 线程池**（`search_directory()`）：
- 默认线程数 = CPU 核心数（自动限制并发，避免线程爆炸）
- 对目录下的每个直接文件执行匹配：
  - 文件名匹配（始终执行，不受扩展名过滤影响）
  - 文本内容匹配（仅限允许的扩展名）
  - EXIF 数据匹配（仅限图片扩展名）
  - OCR 文字识别（仅限 macOS + 图片扩展名）
- 匹配结果通过 `result_tx` 发送
- 进度更新通过 `progress_tx` 发送

**④ Result/Progress Handler 线程**：
- 从 `result_rx` 读取结果，调用 `on_result` 回调写入 `ScanStore`
- 从 `progress_rx` 读取进度，调用 `on_progress` 回调更新 `ScanStore`

#### 1.4 进度轮询与结果返回

**文件**: `src/store/index.ts` - `startScan()`

1. 每 200ms 调用 `get_scan_progress` 获取最新进度
2. 更新前端 `scanProgress` 状态
3. 结果实时出现在 UI 列表中（BFS 边遍历，搜索边处理，结果边返回）
4. 当 `status === "completed"` 或 `"cancelled"` 时停止轮询

---

## 2. 大目录处理流程

### 流程图

```
BFS 遇到大目录 → 发送确认请求 → 前端展示确认面板
→ 用户允许 → respond_confirmation → work_tx.send(DirWork) → 搜索工作协程自动处理
→ 用户拒绝 → 记录 skipped_dirs
```

### 详细步骤

#### 2.1 大目录检测

**文件**: `src-tauri/src/scanner.rs` - `bfs_scan()` → `enqueue_dir()`

1. BFS 遍历过程中，对每个目录调用 `count_entries_fast()` 统计直接子项
2. 如果 `ask_on_large_dir=true` 且子项数 > `threshold`（默认 1000）：
   - 创建 `PendingConfirmation`（id, path, entry_count）
   - 调用 `on_confirmation_needed` 回调
   - 调用 `on_dir_skipped` 回调（reason: "large_dir"）
   - **不发送 DirWork，不继续 BFS 遍历该目录**

#### 2.2 后端通知前端

**文件**: `src-tauri/src/commands/scan.rs` - `start_scan()` 中的 `on_confirmation_needed` 回调

1. 将 `PendingConfirmation` 添加到 `ScanProgress.pending_confirmations`
2. 发送 Tauri 事件 `"confirmation-needed"` 到前端

#### 2.3 前端接收并展示确认面板

**文件**: `src/App.tsx`

1. 监听 `scanProgress?.pending_confirmations.length` 变化
2. 当有新的确认请求时：
   - 发送系统通知（带节流机制，5秒内最多一次）
   - 播放系统声音
   - 自动打开确认面板 `setShowConfirmPanel(true)`

**文件**: `src/components/modals/ConfirmPanel.tsx`

1. 显示待确认目录列表（路径 + 子项数量）
2. 提供操作按钮：
   - "允许" → `respondConfirmation(id, true, false)`
   - "拒绝" → `respondConfirmation(id, false, false)`
   - "全部允许" → 批量调用

#### 2.4 用户响应确认

**文件**: `src/store/index.ts` - `respondConfirmation()`

1. 调用后端 `invoke("respond_confirmation", { scanId, confirmationId, allow, remember })`
2. 更新本地状态（移除确认项，添加跳过目录）
3. **不再启动子扫描** — 后端通过 `work_tx` 直接注入工作项

#### 2.5 后端处理确认响应

**文件**: `src-tauri/src/commands/scan.rs` - `respond_confirmation()`

1. 从 `ScanProgress.pending_confirmations` 中移除该确认
2. 如果 `remember=true`：
   - 加载 `AppConfig`
   - 添加到 `scan_rules`（允许）或 `skip_rules`（拒绝）
   - 保存配置文件
3. 如果 `allow=true`：
   - 从 `ChannelStore` 获取 `work_tx`
   - 发送 `DirWork { path }` 到通道
   - **搜索工作协程自动处理，结果实时出现在主扫描进度中**
4. 如果 `allow=false`：
   - 添加到 `skipped_dirs` 列表

---

## 3. 取消流程

### 3.1 前端取消

**文件**: `src/store/index.ts` - `cancelScan()`

1. 调用 `invoke("cancel_scan", { scanId })`
2. 设置 `isScanning: false`
3. 清除轮询定时器

### 3.2 后端取消

**文件**: `src-tauri/src/commands/scan.rs` - `cancel_scan()`

1. 更新 `ScanProgress.status` 为 `"cancelled"`
2. 设置 `cancel_flag` 为 `true`

### 3.3 扫描引擎响应取消

取消检查点：
- BFS 遍历循环中（每个目录处理前）
- 搜索工作协程中（每个文件处理前）

取消后：
- BFS 线程退出循环，`work_tx` 被 drop
- Dispatcher 线程检测到通道关闭，退出
- Rayon 线程池中的任务检查 `cancel_flag` 后退出
- Result/Progress Handler 线程在通道关闭后退出

---

## 4. 关键数据结构

### 4.1 ScanConfig

**文件**: `src-tauri/src/types.rs`

```rust
pub struct ScanConfig {
    pub path: String,
    pub keyword: String,
    pub scan_types: Vec<String>,
    pub file_extensions: Vec<String>,
    pub exclude_patterns: Vec<String>,
}
```

### 4.2 DirWork

**文件**: `src-tauri/src/types.rs`

```rust
pub struct DirWork {
    pub path: PathBuf,  // 待处理的目录路径
}
```

### 4.3 ChannelStore

**文件**: `src-tauri/src/types.rs`

```rust
pub type ChannelStore = Arc<Mutex<HashMap<String, Sender<DirWork>>>>;
```

存储每个扫描的 `work_tx`，供 `respond_confirmation` 使用。

### 4.4 ScanProgress

**文件**: `src-tauri/src/types.rs`

```rust
pub struct ScanProgress {
    pub scan_id: String,
    pub parent_scan_id: Option<String>,
    pub status: String,                  // scanning, completed, cancelled
    pub files_scanned: u32,
    pub results_found: u32,
    pub current_path: String,
    pub results: Vec<ScanResult>,
    pub pending_confirmations: Vec<PendingConfirmation>,
    pub skipped_dirs: Vec<SkippedDir>,
}
```

### 4.5 PendingConfirmation

**文件**: `src-tauri/src/types.rs`

```rust
pub struct PendingConfirmation {
    pub id: String,
    pub path: String,
    pub entry_count: u64,
}
```

### 4.6 ScanResult

**文件**: `src-tauri/src/scanner.rs`

```rust
pub struct ScanResult {
    pub file_path: String,
    pub file_name: String,
    pub match_type: String,      // filename, content, exif, ocr
    pub match_line: Option<u32>,
    pub match_context: Option<String>,
    pub file_size: u64,
    pub file_extension: String,
    pub is_dir: bool,
}
```

### 4.7 AppConfig

**文件**: `src-tauri/src/config.rs`

```rust
pub struct AppConfig {
    pub version: u32,
    pub scan: ScanSettings,
    pub skip_rules: Vec<String>,
    pub scan_rules: Vec<String>,
}

pub struct ScanSettings {
    pub large_dir_threshold: u64,  // 默认 1000
    pub ask_on_large_dir: bool,    // 默认 true
}
```

---

## 5. 涉及文件和函数汇总

| 文件 | 关键函数 | 作用 |
|------|----------|------|
| `src/store/index.ts` | `startScan()` | 启动扫描 |
| | `cancelScan()` | 取消扫描 |
| | `respondConfirmation()` | 响应确认（通过 work_tx 注入工作项） |
| | `allowAllConfirmations()` | 允许所有确认 |
| `src-tauri/src/commands/scan.rs` | `start_scan()` | 创建通道，启动扫描 |
| | `get_scan_progress()` | 获取进度 |
| | `cancel_scan()` | 取消扫描 |
| | `respond_confirmation()` | 响应确认，通过 ChannelStore 发送 DirWork |
| `src-tauri/src/scanner.rs` | `scan_directory()` | 扫描主函数，协调各组件 |
| | `bfs_scan()` | BFS 线程，遍历目录树并分类 |
| | `enqueue_dir()` | 分类目录：≤阈值发 DirWork，>阈值发确认 |
| | `search_directory()` | 搜索工作函数，处理单个目录的文件 |
| | `count_entries_fast()` | 快速统计目录直接子项 |
| | `is_hidden()` | 判断隐藏文件/目录 |
| | `matches_rules()` | 规则匹配 |
| | `is_text_file()` | 判断文本文件 |
| | `is_image_file()` | 判断图片文件 |
| | `extract_exif()` | 提取 EXIF 数据 |
| | `perform_ocr()` | OCR 识别 |
| `src-tauri/src/types.rs` | `DirWork` | 搜索工作项 |
| | `ChannelStore` | 通道存储 |
| | `ScanProgress` | 扫描进度 |
| `src-tauri/src/config.rs` | `AppConfig` | 配置管理 |
| `src/App.tsx` | `useEffect` 监听 | 通知检测和自动弹出确认面板 |
| `src/components/SearchBar/SearchBar.tsx` | 搜索栏 UI | 用户交互入口 |
| `src/components/modals/ConfirmPanel.tsx` | 确认面板 UI | 确认交互 |

---

## 6. 已解决的问题

### 6.1 架构改进

| 问题 | 旧方案 | 新方案 |
|------|--------|--------|
| 串行瓶颈 | BFS 收集所有文件后再处理 | BFS 与搜索并发，边遍历边处理 |
| 子扫描复杂性 | 前端管理子扫描、轮询、结果合并（约 80 行） | 通过 work_tx 直接注入，无需子扫描 |
| 线程爆炸 | 每个工作项 `std::thread::spawn` | Rayon 线程池（CPU 核心数限制） |
| 结果延迟 | BFS 完成后才开始返回结果 | 实时返回，BFS 期间就能看到结果 |

### 6.2 仍存在的问题

1. **轮询间隔 200ms** — 频繁 IPC 调用，可考虑 Tauri 事件推送
2. **确认面板缺少"记住选择"** — `remember` 参数未在 UI 中暴露
3. **ScanStore 内存泄漏** — 完成的扫描记录不会自动清理

---

## 附录：新架构时序图

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌──────────────┐
│   用户界面   │    │  Store层    │    │  Tauri命令   │    │   扫描引擎    │
└──────┬──────┘    └──────┬──────┘    └──────┬──────┘    └──────┬───────┘
       │                  │                  │                   │
       │  点击搜索        │                  │                   │
       │─────────────────>│                  │                   │
       │                  │  startScan()     │                   │
       │                  │─────────────────>│                   │
       │                  │                  │  创建通道         │
       │                  │                  │  start_scan()     │
       │                  │                  │──────────────────>│
       │                  │                  │                   │
       │                  │                  │     BFS 线程启动   │
       │                  │                  │     Dispatcher 启动│
       │                  │                  │                   │
       │                  │  get_scan_progress (每200ms)         │
       │                  │─────────────────>│                   │
       │                  │                  │  ScanProgress     │
       │                  │<─────────────────│                   │
       │  实时更新结果    │                  │                   │
       │<─────────────────│                  │                   │
       │                  │                  │                   │
       │                  │  (遇到大目录)    │                   │
       │                  │                  │  PendingConfirmation
       │                  │<─────────────────│<──────────────────│
       │  显示确认面板    │                  │                   │
       │<─────────────────│                  │                   │
       │                  │                  │                   │
       │  用户确认允许    │                  │                   │
       │─────────────────>│                  │                   │
       │                  │  respond_confirmation                │
       │                  │─────────────────>│                   │
       │                  │                  │  work_tx.send()   │
       │                  │                  │──────────────────>│
       │                  │                  │                   │
       │                  │                  │  Rayon 处理目录   │
       │                  │                  │  结果实时返回     │
       │                  │  get_scan_progress                   │
       │                  │─────────────────>│                   │
       │                  │  ScanProgress (含新结果)             │
       │                  │<─────────────────│                   │
       │  更新结果列表    │                  │                   │
       │<─────────────────│                  │                   │
       │                  │                  │                   │
       │  扫描完成        │                  │                   │
       │<─────────────────│<─────────────────│<──────────────────│
```
