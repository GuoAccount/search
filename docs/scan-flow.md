# 搜索/扫描完整逻辑流程文档

## 概述

本项目是一个基于 Tauri 的文件搜索工具，采用前后端分离架构：
- **前端**：React + Zustand 状态管理
- **后端**：Rust + Tauri 命令系统
- **扫描引擎**：基于 BFS 遍历 + Rayon 并行处理

---

## 1. 主扫描流程

### 流程图

```
用户点击搜索 → 前端构建配置 → 调用后端命令 → 后端启动扫描 → 轮询进度 → 返回结果
```

### 详细步骤

#### 1.1 前端发起扫描

**文件**: `src/components/SearchBar/SearchBar.tsx:52-59`

```
用户点击"搜索"按钮 → 触发 startScan()
```

**文件**: `src/store/index.ts:98-148` - `startScan()`

1. 获取当前设置（scanPath, keyword, enabledPresets, customExtensions）
2. 验证必要参数（路径、关键字、扩展名不能为空）
3. 构建 `ScanConfig` 对象：
   - `path`: 扫描路径
   - `keyword`: 搜索关键字
   - `scan_types`: 根据配置生成（file_name, text_content, exif_data, ocr_text）
   - `file_extensions`: 文件扩展名列表
   - `exclude_patterns`: 排除模式（当前为空）
4. 设置前端状态：`isScanning: true`
5. 调用 Tauri 命令：`invoke("start_scan", { config })`
6. 启动轮询定时器（200ms 间隔）：`invoke("get_scan_progress", { scanId })`

#### 1.2 后端接收并启动扫描

**文件**: `src-tauri/src/commands/scan.rs:10-103` - `start_scan()`

1. 生成唯一 `scan_id`（UUID）
2. 初始化 `ScanProgress` 结构体
3. 将进度存入 `ScanStore`（全局状态存储）
4. 创建 `should_cancel` 和 `should_pause` 标志位
5. 加载应用配置 `AppConfig`
6. 在 `tokio::spawn` 中启动异步任务：
   - 创建 `ScanCallback` 回调结构体
   - 调用 `scanner::scan_directory_with_callback()`
   - 扫描完成后更新状态为 `"completed"`

#### 1.3 扫描引擎执行

**文件**: `src-tauri/src/scanner.rs:61-258` - `scan_directory_with_callback()`

1. **BFS 收集阶段**（第 89-93 行）：
   - 调用 `collect_entries_bfs()` 收集所有待处理文件
   - 使用广度优先搜索遍历目录树

2. **并行处理阶段**（第 118-251 行）：
   - 使用 Rayon 并行处理文件
   - 根据 CPU 核心数分块处理
   - 对每个文件执行多种匹配：
     - 文件名匹配（`file_name`）
     - 文本内容匹配（`text_content`）
     - EXIF 数据匹配（`exif_data`）
     - OCR 文字识别（`ocr_text`，仅 macOS）

3. **结果收集**（第 100-112 行）：
   - 使用 `mpsc::channel` 收集结果
   - 通过回调函数通知前端

#### 1.4 进度轮询与结果返回

**文件**: `src/store/index.ts:129-141`

1. 每 200ms 调用 `get_scan_progress` 获取最新进度
2. 更新前端 `scanProgress` 状态
3. 当 `status === "completed"` 或 `"cancelled"` 时停止轮询

---

## 2. 大目录处理流程

### 流程图

```
遇到大目录 → 触发确认 → 用户选择 → 子扫描 → 结果合并
```

### 详细步骤

#### 2.1 大目录检测

**文件**: `src-tauri/src/scanner.rs:303-366` - `should_process_entry()`

1. 在 BFS 遍历过程中，对每个目录进行检查
2. 检查条件（第 348-363 行）：
   - `ctx.ask_on_large_dir` 为 true
   - 调用 `count_entries_fast()` 统计目录直接子项数量
   - 如果数量超过 `threshold`（默认 1000），触发确认流程

3. 创建 `PendingConfirmation` 结构体：
   - `id`: 唯一标识
   - `path`: 目录路径
   - `entry_count`: 子项数量

4. 调用回调 `on_confirmation_needed` 通知后端
5. 将目录标记为跳过（`reason: "large_dir"`）

#### 2.2 后端通知前端

**文件**: `src-tauri/src/commands/scan.rs:75-82`

1. 回调函数将确认请求添加到 `ScanProgress.pending_confirmations`
2. 发送 Tauri 事件 `"confirmation-needed"` 到前端

#### 2.3 前端接收并展示确认面板

**文件**: `src/App.tsx:59-101`

1. 监听 `scanProgress?.pending_confirmations.length` 变化
2. 当有新的确认请求时：
   - 发送系统通知（带节流机制，5秒内最多一次）
   - 播放系统声音
   - 自动打开确认面板 `setShowConfirmPanel(true)`

**文件**: `src/components/modals/ConfirmPanel.tsx:5-77`

1. 显示待确认目录列表
2. 每个目录显示：
   - 路径
   - 子项数量
3. 提供操作按钮：
   - "允许"：触发子扫描
   - "拒绝"：跳过该目录
   - "全部允许"：批量处理

#### 2.4 用户响应确认

**文件**: `src/store/index.ts:198-304` - `respondConfirmation()`

1. 调用后端 `invoke("respond_confirmation", ...)` 传递：
   - `scanId`
   - `confirmationId`
   - `allow` (是否允许)
   - `remember` (是否记住选择)

2. **如果允许（allow=true）**：
   - 构建子扫描配置
   - 调用 `invoke("scan_sub_directory", ...)` 启动子扫描
   - 启动子扫描轮询定时器（200ms 间隔）
   - 将子扫描结果合并到主扫描结果中（去重处理）

3. **如果拒绝（allow=false）**：
   - 将目录添加到 `skipped_dirs` 列表

4. **如果记住选择（remember=true）**：
   - 更新后端配置文件
   - 刷新前端配置

#### 2.5 后端处理确认响应

**文件**: `src-tauri/src/commands/scan.rs:291-331` - `respond_confirmation()`

1. 从 `ScanProgress.pending_confirmations` 中移除该确认
2. 如果 `remember=true`：
   - 加载 `AppConfig`
   - 添加到 `scan_rules`（允许）或 `skip_rules`（拒绝）
   - 保存配置文件
3. 如果 `allow=false`：
   - 添加到 `skipped_dirs` 列表

#### 2.6 子扫描执行

**文件**: `src-tauri/src/commands/scan.rs:106-217` - `scan_sub_directory()`

1. 生成新的 `sub_scan_id`
2. 创建子扫描进度结构体（包含 `parent_scan_id`）
3. 启动子扫描任务
4. **关键：结果合并**（第 170-181 行）：
   - 子扫描的 `on_result` 回调同时更新：
     - 子扫描进度
     - 主扫描进度
5. 子扫描完成后更新子扫描状态为 `"completed"`

#### 2.7 前端结果合并

**文件**: `src/store/index.ts:258-293`

1. 子扫描轮询定时器检查子扫描进度
2. 将子扫描结果与主扫描结果合并：
   - 使用 `existingPaths` Set 去重
   - 只添加不重复的结果
3. 更新 `results_found` 计数

---

## 3. 暂停/恢复流程

### 3.1 暂停扫描

**前端**: `src/store/index.ts:166-180` - `pauseScan()`

1. 调用 `invoke("pause_scan", { scanId })`
2. 更新前端状态：`scanProgress.status = "paused"`

**后端**: `src-tauri/src/commands/scan.rs:249-268` - `pause_scan()`

1. 更新 `ScanProgress.status` 为 `"paused"`
2. 设置 `pause_flag` 为 `true`

**扫描引擎**: `src-tauri/src/scanner.rs:132-138`

```rust
while *should_pause.lock().unwrap() {
    std::thread::sleep(std::time::Duration::from_millis(100));
    // 暂停期间也检查取消标志
    if *should_cancel.lock().unwrap() {
        return;
    }
}
```

### 3.2 恢复扫描

**前端**: `src/store/index.ts:182-196` - `resumeScan()`

1. 调用 `invoke("resume_scan", { scanId })`
2. 更新前端状态：`scanProgress.status = "scanning"`

**后端**: `src-tauri/src/commands/scan.rs:270-289` - `resume_scan()`

1. 更新 `ScanProgress.status` 为 `"scanning"`
2. 设置 `pause_flag` 为 `false`

---

## 4. 取消流程

### 4.1 前端取消

**文件**: `src/store/index.ts:150-164` - `cancelScan()`

1. 调用 `invoke("cancel_scan", { scanId })`
2. 设置 `isScanning: false`
3. 清除轮询定时器

### 4.2 后端取消

**文件**: `src-tauri/src/commands/scan.rs:228-247` - `cancel_scan()`

1. 更新 `ScanProgress.status` 为 `"cancelled"`
2. 设置 `cancel_flag` 为 `true`

### 4.3 扫描引擎响应取消

**文件**: `src-tauri/src/scanner.rs:127-129`

```rust
if *should_cancel.lock().unwrap() {
    return;
}
```

在以下位置检查取消标志：
- BFS 遍历循环中（第 277-279 行）
- 并行处理每个文件前（第 127-129 行）
- 暂停等待循环中（第 135-137 行）

---

## 5. 关键数据结构

### 5.1 ScanConfig

**文件**: `src-tauri/src/types.rs:7-14`

```rust
pub struct ScanConfig {
    pub path: String,           // 扫描路径
    pub keyword: String,        // 搜索关键字
    pub scan_types: Vec<String>, // 扫描类型
    pub file_extensions: Vec<String>, // 文件扩展名
    pub exclude_patterns: Vec<String>, // 排除模式
}
```

### 5.2 ScanProgress

**文件**: `src-tauri/src/types.rs:29-40`

```rust
pub struct ScanProgress {
    pub scan_id: String,
    pub parent_scan_id: Option<String>,  // 子扫描关联的主扫描ID
    pub status: String,                  // scanning, paused, completed, cancelled
    pub files_scanned: u32,
    pub results_found: u32,
    pub current_path: String,
    pub results: Vec<ScanResult>,
    pub pending_confirmations: Vec<PendingConfirmation>,
    pub skipped_dirs: Vec<SkippedDir>,
}
```

### 5.3 PendingConfirmation

**文件**: `src-tauri/src/types.rs:16-21`

```rust
pub struct PendingConfirmation {
    pub id: String,
    pub path: String,
    pub entry_count: u64,
}
```

### 5.4 ScanResult

**文件**: `src-tauri/src/scanner.rs:23-33`

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

### 5.5 AppConfig

**文件**: `src-tauri/src/config.rs:6-18`

```rust
pub struct AppConfig {
    pub version: u32,
    pub scan: ScanSettings,
    pub skip_rules: Vec<String>,  // 跳过规则
    pub scan_rules: Vec<String>,  // 扫描规则（优先级最高）
}

pub struct ScanSettings {
    pub large_dir_threshold: u64,  // 大目录阈值（默认1000）
    pub ask_on_large_dir: bool,    // 是否询问大目录（默认true）
}
```

---

## 6. 涉及文件和函数汇总

| 文件 | 关键函数 | 作用 |
|------|----------|------|
| `src/store/index.ts` | `startScan()` | 启动扫描 |
| | `cancelScan()` | 取消扫描 |
| | `pauseScan()` | 暂停扫描 |
| | `resumeScan()` | 恢复扫描 |
| | `respondConfirmation()` | 响应确认请求 |
| | `allowAllConfirmations()` | 允许所有确认 |
| `src-tauri/src/commands/scan.rs` | `start_scan()` | 后端启动扫描 |
| | `scan_sub_directory()` | 子扫描 |
| | `get_scan_progress()` | 获取进度 |
| | `cancel_scan()` | 取消扫描 |
| | `pause_scan()` | 暂停扫描 |
| | `resume_scan()` | 恢复扫描 |
| | `respond_confirmation()` | 响应确认 |
| `src-tauri/src/scanner.rs` | `scan_directory_with_callback()` | 扫描主函数 |
| | `collect_entries_bfs()` | BFS 收集文件 |
| | `should_process_entry()` | 判断是否处理 |
| | `count_entries_fast()` | 快速统计目录项 |
| | `is_text_file()` | 判断文本文件 |
| | `is_image_file()` | 判断图片文件 |
| | `extract_exif()` | 提取 EXIF 数据 |
| | `perform_ocr()` | OCR 识别 |
| `src-tauri/src/types.rs` | 数据结构定义 | - |
| `src-tauri/src/config.rs` | `AppConfig` | 配置管理 |
| `src/App.tsx` | `useEffect` 监听 | 通知检测和自动弹出 |
| `src/components/SearchBar/SearchBar.tsx` | 搜索栏 UI | 用户交互入口 |
| `src/components/modals/ConfirmPanel.tsx` | 确认面板 UI | 确认交互 |

---

## 7. 当前可能存在的问题或遗漏

### 7.1 架构设计问题

1. **子扫描结果合并存在竞态条件**
   - 位置：`src/store/index.ts:264-282`
   - 问题：子扫描轮询时，`subProgress.results` 可能包含重复结果，前端去重逻辑依赖 `existingPaths` Set，但主扫描结果可能同时在更新
   - 建议：后端统一管理结果合并，前端只负责展示

2. **子扫描的取消和暂停独立于主扫描**
   - 位置：`src-tauri/src/commands/scan.rs:134-143`
   - 问题：子扫描有独立的 `cancel_flag` 和 `pause_flag`，取消主扫描不会自动取消子扫描
   - 建议：实现级联取消机制

3. **确认面板的 `remember` 参数未使用**
   - 位置：`src/components/modals/ConfirmPanel.tsx:56,65`
   - 问题：UI 中没有提供 "记住选择" 的选项，`respondConfirmation` 调用时 `remember` 始终为 `false`
   - 建议：添加复选框让用户选择是否记住

### 7.2 性能问题

4. **轮询间隔过短（200ms）**
   - 位置：`src/store/index.ts:141,293`
   - 问题：频繁的 IPC 调用可能影响性能
   - 建议：考虑使用 Tauri 的事件推送机制替代轮询

5. **BFS 收集所有文件后再处理**
   - 位置：`src-tauri/src/scanner.rs:89-93`
   - 问题：对于超大目录树，`collect_entries_bfs()` 可能消耗大量内存
   - 建议：改为流式处理，边收集边处理

6. **暂停使用忙等待（busy wait）**
   - 位置：`src-tauri/src/scanner.rs:132-138`
   - 问题：`while` 循环 + `sleep(100ms)` 消耗 CPU
   - 建议：使用 `Condvar` 或 `channel` 实现真正的阻塞等待

### 7.3 功能缺失

7. **没有实现 `allowAllConfirmations` 的批量后端接口**
   - 位置：`src/store/index.ts:306-313`
   - 问题：当前实现是循环调用 `respondConfirmation`，每次都会触发子扫描和配置更新
   - 建议：添加批量处理的后端命令

8. **子扫描的进度更新不反映到主扫描的 `files_scanned`**
   - 位置：`src-tauri/src/commands/scan.rs:183-188`
   - 问题：子扫描的 `files_scanned` 只更新子扫描进度，不累加到主扫描
   - 建议：同步更新主扫描的扫描计数

9. **缺少扫描错误处理**
   - 位置：`src-tauri/src/scanner.rs:140-143`
   - 问题：文件元数据读取失败时直接 `continue`，没有记录错误
   - 建议：添加错误收集机制，记录无法访问的文件

10. **OCR 功能仅限 macOS**
    - 位置：`src-tauri/src/scanner.rs:231-249`
    - 问题：使用 Swift 脚本执行 OCR，不支持其他平台
    - 建议：考虑跨平台 OCR 方案（如 Tesseract）

### 7.4 状态管理问题

11. **扫描状态恢复不完整**
    - 位置：`src/store/index.ts:133-137`
    - 问题：如果页面刷新，正在进行的扫描状态会丢失（轮询定时器丢失）
    - 建议：实现扫描状态持久化和恢复机制

12. **`ScanStore` 内存泄漏风险**
    - 位置：`src-tauri/src/types.rs:60`
    - 问题：完成的扫描记录不会自动清理，长时间运行会积累大量数据
    - 建议：实现扫描记录的过期清理机制

### 7.5 用户体验问题

13. **大目录确认缺乏上下文信息**
    - 位置：`src/components/modals/ConfirmPanel.tsx:48-49`
    - 问题：只显示子项数量，不显示目录完整路径和预计扫描时间
    - 建议：添加更多信息帮助用户决策

14. **没有提供扫描进度百分比**
    - 位置：`src-tauri/src/types.rs:29-40`
    - 问题：`ScanProgress` 没有总文件数，无法计算百分比
    - 建议：在 BFS 阶段估算总文件数

---

## 8. 优化建议

### 短期优化

1. 使用 Tauri 事件系统替代轮询
2. 实现批量确认接口
3. 添加 "记住选择" UI
4. 改进暂停机制（使用 Condvar）

### 中期优化

1. 流式处理文件（边收集边处理）
2. 实现子扫描级联取消
3. 添加扫描错误收集和展示
4. 实现扫描状态持久化

### 长期优化

1. 跨平台 OCR 支持
2. 增量扫描支持
3. 扫描结果缓存
4. 分布式扫描支持

---

## 附录：完整流程时序图

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   用户界面   │    │  Store层    │    │  Tauri命令   │    │  扫描引擎    │
└──────┬──────┘    └──────┬──────┘    └──────┬──────┘    └──────┬──────┘
       │                  │                  │                  │
       │  点击搜索        │                  │                  │
       │─────────────────>│                  │                  │
       │                  │  startScan()     │                  │
       │                  │─────────────────>│                  │
       │                  │                  │  start_scan()    │
       │                  │                  │─────────────────>│
       │                  │                  │                  │
       │                  │                  │  scan_id         │
       │                  │                  │<─────────────────│
       │                  │  scan_id         │                  │
       │                  │<─────────────────│                  │
       │                  │                  │                  │
       │                  │  启动轮询        │                  │
       │                  │─────────────────────────────────────>│
       │                  │                  │                  │
       │                  │  get_scan_progress (每200ms)        │
       │                  │─────────────────>│                  │
       │                  │                  │  ScanProgress    │
       │                  │<─────────────────│                  │
       │                  │                  │                  │
       │                  │  (遇到大目录)    │                  │
       │                  │                  │  PendingConfirmation
       │                  │<─────────────────│<─────────────────│
       │  显示确认面板    │                  │                  │
       │<─────────────────│                  │                  │
       │                  │                  │                  │
       │  用户确认允许    │                  │                  │
       │─────────────────>│                  │                  │
       │                  │  respond_confirmation               │
       │                  │─────────────────>│                  │
       │                  │                  │                  │
       │                  │  scan_sub_directory                 │
       │                  │─────────────────>│                  │
       │                  │                  │  子扫描执行      │
       │                  │                  │─────────────────>│
       │                  │                  │                  │
       │                  │  子扫描轮询      │                  │
       │                  │─────────────────────────────────────>│
       │                  │                  │                  │
       │                  │  合并结果        │                  │
       │                  │<─────────────────│<─────────────────│
       │  更新结果列表    │                  │                  │
       │<─────────────────│                  │                  │
       │                  │                  │                  │
       │  扫描完成        │                  │                  │
       │<─────────────────│<─────────────────│<─────────────────│
       │                  │                  │                  │
```
