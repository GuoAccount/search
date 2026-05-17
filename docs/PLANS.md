# PLANS.md

执行计划规则 + 当前热点任务。

## 计划规则

- 跨越多会话、多子系统的工作必须有 plan
- `docs/exec-plans/active/`：当前计划
- `docs/exec-plans/completed/`：已完成计划
- 每个 plan 包含：目标、范围、验证路径、风险、进度日志

---

## 当前热点任务

### 🔴 P0: 扫描不产出结果 [进行中]

**问题：** 扫描运行后只触发大目录确认事件，不产出任何搜索结果。

**症状：**
- 点击搜索后，"待确认"按钮出现
- 确认面板弹出，显示大目录
- 但扫描结果始终为 0
- 即使允许扫描大目录，结果仍为 0

**排查方向：**
- [ ] `collect_entries_bfs` 返回的 entries 数量
- [ ] `should_process_entry` 是否错误跳过文件
- [ ] `on_result` 回调是否被调用
- [ ] `get_scan_progress` 返回的 results 数据

详见 `docs/QUALITY.md` BUG-001

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
