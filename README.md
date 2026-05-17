<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="128" height="128" alt="Lumina Icon" />
</p>

<h1 align="center">Lumina</h1>

<p align="center">
  <em>Illuminate your files</em>
</p>

<p align="center">
  <strong>A fast, native file search and management tool built with Tauri 2 + React</strong>
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

## Features

- **Multi-mode search** — Search by file name, text content, EXIF metadata, or OCR (macOS)
- **File type presets** — Quick filter by Document, Code, Image, or Config categories
- **Tree view** — Hierarchical folder tree with expand/collapse navigation
- **File preview** — In-app preview for text, code, and image files
- **Batch management** — Select and move multiple files to trash with one click
- **Native performance** — Rust-powered search engine with multi-threaded scanning
- **Theme support** — Light, Dark, and System theme modes
- **Keyboard shortcuts** — `Cmd/Ctrl+B` toggle sidebar, `Cmd/Ctrl+Enter` start scan
- **Sidebar layout** — Collapsible sidebar with file type configuration
- **Reveal in Finder** — Open file location directly in system file manager
- **Cross-platform** — Runs on macOS, Windows, and Linux

## Search Modes

| Mode | Description |
|------|-------------|
| **File Name** | Match keywords against file and folder names |
| **Text Content** | Search inside text-based files (code, documents, configs) |
| **EXIF Data** | Search image EXIF metadata (camera, lens, GPS, etc.) |
| **OCR Text** | Extract and search text from images using Vision framework (macOS only) |

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) >= 20
- [pnpm](https://pnpm.io/) >= 9
- [Rust](https://www.rust-lang.org/) >= 1.77
- **macOS**: Xcode Command Line Tools
- **Windows**: Microsoft Visual Studio C++ Build Tools
- **Linux**: `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`

### Development

```bash
# Install dependencies
pnpm install

# Start development server
pnpm tauri dev

# Build for production
pnpm tauri build
```

### Install from Releases

Download the latest installer for your platform from [GitHub Releases](https://github.com/GuoAccount/search/releases):

| Platform | Formats |
|----------|---------|
| macOS (Apple Silicon) | `.dmg`, `.app` |
| macOS (Intel) | `.dmg`, `.app` |
| Windows | `.msi`, `.nsis` |
| Linux | `.deb`, `.AppImage` |

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | React 19, TypeScript, Vite 7, Lucide Icons |
| Backend | Rust, Tauri 2, Walkdir, Rayon, Kamadak-exif |
| Build | pnpm, tauri-cli |
| CI/CD | GitHub Actions |

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl + B` | Toggle sidebar |
| `Cmd/Ctrl + Enter` | Start scan |
| `Enter` | Submit search input |

## Project Structure

```
search/
├── src/                    # React frontend
│   ├── App.tsx             # Main application component
│   ├── App.css             # Application styles
│   └── index.css           # Theme variables and base styles
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── lib.rs          # Tauri commands and app entry
│   │   └── scanner.rs      # File scanning engine
│   ├── resources/          # OCR scripts, sounds, etc.
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
├── .github/workflows/      # CI/CD release pipeline
├── DESIGN.md               # Apple-style design system documentation
├── package.json
└── vite.config.ts
```

## License

MIT
