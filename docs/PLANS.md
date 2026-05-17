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

### 🟢 P2: 功能完善 [待开始]

- [ ] 确认面板添加"记住选择"选项
- [ ] 批量确认接口优化
- [ ] 扫描状态持久化
- [ ] 子扫描级联取消
- [ ] ScanStore 内存清理
