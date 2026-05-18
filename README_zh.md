<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="128" height="128" alt="Lumina Icon" />
</p>

<h1 align="center">Lumina</h1>

<p align="center">
  <em>照亮你的文件</em><br>
  🔍 OCR · 📄 全文搜索 · 🏷️ EXIF · ⚡ Tauri · 🍎 macOS
</p>

<p align="center">
  <strong>基于 Tauri 2 + React 构建的高速原生文件搜索与管理工具</strong>
</p>

<p align="center">
  <a href="README.md">English</a> · <a href="README_zh.md">中文</a> · <a href="README_ja.md">日本語</a> · <a href="README_ko.md">한국어</a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blueviolet" alt="Platform" />
  <img src="https://img.shields.io/badge/tauri-2-orange" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/react-19-blue" alt="React 19" />
  <img src="https://img.shields.io/badge/rust-2021-orange" alt="Rust" />
  <img src="https://img.shields.io/github/v/release/GuoAccount/search" alt="GitHub Release" />
</p>

---

## 功能特性

- **多模式搜索** — 支持文件名搜索、文本内容搜索、EXIF 元数据搜索、OCR 文字识别（macOS）
- **文件类型预设** — 快速筛选文档、代码、图片、配置文件四大类别
- **树形视图** — 层级文件夹树，支持展开/折叠导航
- **文件预览** — 应用内预览文本、代码和图片文件，带上下文高亮
- **OCR 高亮** — OCR 匹配图片上显示可视化的边界框叠加
- **批量管理** — 勾选多个文件一键移到废纸篓
- **原生性能** — BFS + Rayon 线程池并发扫描，结果实时流式返回
- **大目录处理** — 超大目录扫描前弹出确认对话框，支持"记住选择"规则
- **跳过目录日志** — 查看哪些目录被跳过以及原因
- **持久化配置** — 大目录阈值、扫描/跳过规则、显示偏好等跨会话保存
- **主题切换** — 浅色、深色、跟随系统三种主题模式
- **快捷键** — `Cmd/Ctrl+B` 切换侧边栏，`Cmd/Ctrl+Enter` 开始扫描
- **侧边栏布局** — 可折叠侧边栏，含文件类型预设、扫描路径和 OCR 开关
- **设置面板** — 配置扫描阈值、显示选项和文件类型扩展名规则
- **访达中打开** — 直接在系统文件管理器中定位文件
- **跨平台** — 支持 macOS、Windows、Linux

## 搜索模式

| 模式 | 说明 |
|------|------|
| **文件名** | 匹配文件名和文件夹名中的关键字 |
| **文本内容** | 在文本类文件（代码、文档、配置）中搜索 |
| **EXIF 数据** | 搜索图片 EXIF 元数据（相机、镜头、GPS 等） |
| **OCR 文字** | 使用 Vision 框架提取图片中的文字并搜索（仅 macOS） |

## 快速开始

### 环境要求

- [Node.js](https://nodejs.org/) >= 20
- [pnpm](https://pnpm.io/) >= 9
- [Rust](https://www.rust-lang.org/) >= 1.77
- **macOS**：Xcode 命令行工具
- **Windows**：Microsoft Visual Studio C++ 构建工具
- **Linux**：`libgtk-3-dev`、`libwebkit2gtk-4.1-dev`、`libappindicator3-dev`、`librsvg2-dev`

### 开发调试

```bash
# 安装依赖
pnpm install

# 启动开发服务器
pnpm tauri dev

# 生产构建
pnpm tauri build
```

### 从 Releases 安装

从 [GitHub Releases](https://github.com/GuoAccount/search/releases) 下载对应平台的安装包：

| 平台 | 格式 |
|------|------|
| macOS (Apple Silicon) | `.dmg`、`.app` |
| macOS (Intel) | `.dmg`、`.app` |
| Windows | `.msi`、`.nsis` |
| Linux | `.deb`、`.AppImage` |

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | React 19、TypeScript、Vite 7、Zustand 5、Lucide Icons |
| 后端 | Rust、Tauri 2、Walkdir、Rayon、Kamadak-exif、Trash |
| 构建 | pnpm、tauri-cli |
| CI/CD | GitHub Actions |

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Cmd/Ctrl + B` | 切换侧边栏 |
| `Cmd/Ctrl + Enter` | 开始扫描 |
| `Enter` | 提交搜索输入 |

## 搜索引擎架构

扫描引擎采用并发的 **BFS + Rayon 线程池** 设计：

1. **BFS 线程** 遍历目录树，按大小对目录分类
2. **Dispatcher 线程** 从通道读取工作项，分发到 Rayon 线程池
3. **Rayon 线程池**（CPU 密集型）并行执行文件匹配——文件名、文本内容、EXIF、OCR
4. **Result/Progress 处理线程** 将结果实时流式返回前端

超过阈值（默认 1000 项）的大目录会触发确认对话框，可选"记住选择"规则系统。

## 项目结构

```
search/
├── src/                          # React 前端
│   ├── App.tsx                   # 主应用组件
│   ├── App.css                   # 应用样式
│   ├── index.css                 # 主题变量与基础样式
│   ├── main.tsx                  # React 入口
│   ├── components/
│   │   ├── Sidebar/              # 可折叠侧边栏（预设 + 路径 + OCR 开关）
│   │   ├── SearchBar/            # 搜索输入与扫描控制
│   │   ├── ResultsView/          # 树形视图文件结果
│   │   ├── ScanProgress/         # 扫描进度指示器
│   │   ├── PresetSelector/       # 文件类型预设配置
│   │   ├── EmptyState/           # 空状态占位
│   │   └── modals/
│   │       ├── ConfirmPanel.tsx   # 大目录确认对话框
│   │       ├── SkippedDirsPanel.tsx # 跳过目录查看
│   │       ├── SettingsPanel.tsx  # 应用设置面板
│   │       ├── FilePreviewModal.tsx # 文本/代码文件预览
│   │       └── ImagePreviewModal.tsx # 图片预览（含 OCR 高亮）
│   ├── store/
│   │   └── index.ts              # Zustand 状态管理
│   ├── types/
│   │   └── index.ts              # TypeScript 类型定义
│   ├── constants/
│   │   └── presets.ts            # 文件类型预设配置
│   ├── utils/
│   │   ├── storage.ts            # 设置持久化
│   │   └── file.ts               # 文件工具函数
│   └── assets/                   # 静态资源
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── lib.rs                # Tauri 构建器、插件、命令注册
│   │   ├── main.rs               # 应用入口
│   │   ├── types.rs              # 共享数据结构
│   │   ├── config.rs             # 持久化配置管理
│   │   ├── scanner.rs            # 文件扫描引擎（BFS + Rayon）
│   │   └── commands/
│   │       ├── mod.rs            # 命令模块声明
│   │       ├── scan.rs           # 扫描生命周期命令
│   │       ├── file_ops.rs       # 文件操作（预览、废纸篓、访达中打开）
│   │       └── system.rs         # 系统命令（音效、配置文件）
│   ├── resources/                # OCR 脚本、音效等资源
│   ├── Cargo.toml                # Rust 依赖
│   └── tauri.conf.json           # Tauri 配置
├── docs/                         # 架构与产品文档
├── .github/workflows/            # CI/CD 发布流水线
├── DESIGN.md                     # Apple 风格设计系统文档
├── ARCHITECTURE.md               # 系统架构概览
├── package.json
└── vite.config.ts
```

## 开源协议

MIT
