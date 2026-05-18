# PLANS.md

执行计划规则 + 当前热点任务。

## 计划规则

- 跨越多会话、多子系统的工作必须有 plan
- `docs/exec-plans/active/`：当前计划
- `docs/exec-plans/completed/`：已完成计划
- 每个 plan 包含：目标、范围、验证路径、风险、进度日志

---

## 当前热点任务

### 🔴 P0: 搜索逻辑重构 [已完成]

将扫描引擎从"先收集后处理"改为"BFS 与搜索工作协程并发"架构。

**已完成：**
- [x] 重构 scanner.rs：BFS 线程 + 搜索工作协程
- [x] 重构 commands/scan.rs：通道管理，消除 scan_sub_directory
- [x] 新增 DirWork + ChannelStore 类型
- [x] 简化 store/index.ts：移除子扫描轮询和结果合并逻辑（约 80 行）
- [x] cargo check + npm run build 编译通过

---

### 🟡 P1: 模块化重构 [已完成]

前后端模块化重构，将巨型文件拆分为职责单一的模块。

**已完成：**
- [x] 后端：types.rs, commands/, lib.rs 精简
- [x] 前端：types/, constants/, utils/, store/, components/
- [x] Zustand 状态管理
- [x] CSS Modules
- [x] CancelStore 修复

---

### 🟡 P1: Bug修复冲刺 [进行中]

修复用户报告的多个关键bug。

**已完成：**
- [x] 展开按钮bug修复（文件路径→文件夹路径）
- [x] 分类tab平铺显示
- [x] 根目录文件搜索修复
- [x] 重复结果消除
- [x] 扫描完成状态修复
- [x] 格式计数修复
- [x] 扩展名筛选bug（文件名匹配未检查 ext_allowed）
- [x] 删除文件复活bug（move_to_trash 未清理 ScanStore）

**待验证：**
- [ ] 大目录确认流程
- [ ] OCR搜索图片文字功能

---

### 🟡 P1: 文档内容提取 [已完成]

为 docx/xlsx/pptx/pdf 实现文档内容搜索，并设为可配置开关（默认开启）。

**已完成：**
- [x] 后端 config.rs: 新增 ContentExtractionSettings（docx/xlsx/pdf/pptx 开关）
- [x] 后端 scanner.rs: 实现 extract_docx/xlsx/pptx/pdf_text，集成到 search_directory
- [x] 后端 config.rs: 加载时自动迁移（serde default + 写回缺失字段）
- [x] 后端 scanner.rs: pdf-extract panic 用 catch_unwind 兜底
- [x] 前端 SettingsPanel: 扫描选项卡新增文档内容提取切换开关
- [x] 前端 store: startScan 自动添加 "document_content" 到 scanTypes
- [x] 前端 types: AppConfig 新增 content_extraction 字段
- [x] 文件预览: 内容匹配的文档可预览提取文本（match_type !== filename 时显示预览按钮）
- [x] SettingsPanel UI: 用 CSS 变量重写，保存按钮改为圆角矩形
- [x] Cargo.toml: 新增 zip, quick-xml, pdf-extract 依赖
- [x] docs/scan-flow.md: 同步更新

---

### 🟢 P2: 功能完善 [待开始]

- [ ] 确认面板添加"记住选择"选项
- [ ] 批量确认接口优化
- [ ] 扫描状态持久化
- [ ] 子扫描级联取消
- [ ] ScanStore 内存清理
