# DESIGN.md

这份文件是设计文档体系的入口。把具体的设计决策放进 `docs/design-docs/`，这里只放路由和概要。

## 设计文档体系

设计决策跨会话、跨 reviewer 持久存在。详见：

- `docs/design-docs/index.md`：设计历史地图
- `docs/design-docs/core-beliefs.md`：核心运行信念

## 什么时候写设计文档

- 决策影响系统整体形状或多个领域
- 决策涉及取舍，需要记录为什么选 A 不选 B
- 决策会被后续 agent 反复碰到

## 维护规则

- 设计文档要保持简短，把实现细节留给代码和 plan。
- 过期的设计文档要么删除，要么明确标成 deprecated。
