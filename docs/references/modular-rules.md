# 模块化开发规则

## 通用

- 前端和后端都必须模块化：单一职责、高内聚、低耦合。
- 禁止过度设计。足够简单到能理解，不要引入不必要的抽象层。
- 每个组件/模块只做一件事，做好一件事。

## 前端（React）

- 每个组件独立目录，配套 CSS Module（`ComponentName.tsx` + `ComponentName.module.css`）。
- 组件只从 Zustand store 读取状态，不直接调用 Tauri IPC（`invoke`）。
- Store 是唯一调用 `invoke` 的层，UI 层只通过 Store 方法触发后端操作。
- 跨组件共享状态走 Store，不走 props drilling 或 context。
- 工具函数（纯函数）放在 `src/utils/`，类型定义放在 `src/types/`，常量放在 `src/constants/`。

## 后端（Rust）

- `commands/` 目录下每个命令文件只负责一个领域（`scan.rs`、`file_ops.rs`、`system.rs`）。
- 共享数据结构放在 `types.rs`，只依赖标准库和 serde。
- `scanner.rs` 只依赖 `types.rs`，不依赖 `commands/`。
- `config.rs` 独立管理配置读写，不耦合业务逻辑。
- 禁止在命令处理函数中直接编写业务逻辑——委托给 `scanner.rs` 等专用模块。
