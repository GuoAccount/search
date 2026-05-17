# AGENTS.md

## 构建发布规范

### 版本号更新

每次构建发布前，必须更新以下三个文件中的版本号：

1. `package.json` - `"version": "x.x.x"`
2. `src-tauri/tauri.conf.json` - `"version": "x.x.x"`
3. `src-tauri/Cargo.toml` - `version = "x.x.x"`

### 发布流程

1. 更新上述三个文件的版本号
2. 提交代码：`git commit -m "chore: bump version to x.x.x"`
3. 推送代码：`git push`
4. 删除旧标签（如需要）：`git tag -d vx.x.x && git push origin :refs/tags/vx.x.x`
5. 创建新标签：`git tag vx.x.x`
6. 推送标签：`git push origin vx.x.x`

### 版本号规范

- 使用语义化版本：`主版本.次版本.修订号`
- 示例：`2.0.2`
