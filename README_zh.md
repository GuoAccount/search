<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="128" height="128" alt="Lumina Icon" />
</p>

<h1 align="center">Lumina</h1>

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
- **文件预览** — 应用内预览文本、代码和图片文件
- **批量管理** — 勾选多个文件一键移到废纸篓
- **原生性能** — Rust 驱动的搜索引擎，多线程扫描
- **主题切换** — 浅色、深色、跟随系统三种主题模式
- **快捷键** — `Cmd/Ctrl+B` 切换侧边栏，`Cmd/Ctrl+Enter` 开始扫描
- **侧边栏布局** — 可折叠侧边栏，配置文件类型筛选
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
| 前端 | React 19、TypeScript、Vite 7、Lucide Icons |
| 后端 | Rust、Tauri 2、Walkdir、Rayon、Kamadak-exif |
| 构建 | pnpm、tauri-cli |
| CI/CD | GitHub Actions |

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Cmd/Ctrl + B` | 切换侧边栏 |
| `Cmd/Ctrl + Enter` | 开始扫描 |
| `Enter` | 提交搜索输入 |

## 项目结构

```
search/
├── src/                    # React 前端
│   ├── App.tsx             # 主应用组件
│   ├── App.css             # 应用样式
│   └── index.css           # 主题变量与基础样式
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── lib.rs          # Tauri 命令与应用入口
│   │   └── scanner.rs      # 文件扫描引擎
│   ├── resources/          # OCR 脚本、音效等资源
│   ├── Cargo.toml          # Rust 依赖
│   └── tauri.conf.json     # Tauri 配置
├── .github/workflows/      # CI/CD 发布流水线
├── DESIGN.md               # Apple 风格设计系统文档
├── package.json
└── vite.config.ts
```

## 开源协议

MIT
