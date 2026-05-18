<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="128" height="128" alt="Lumina Icon" />
</p>

<h1 align="center">Lumina</h1>

<p align="center">
  <em>Illuminate your files</em><br>
  🔍 OCR · 📄 全文搜索 · 🏷️ EXIF · ⚡ Tauri · 🍎 macOS
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
- **File preview** — In-app preview for text, code, and image files with context highlighting
- **OCR highlight** — Visual bounding box overlay on OCR-matched images
- **Batch management** — Select and move multiple files to trash with one click
- **Native performance** — BFS + Rayon thread pool for concurrent scanning, real-time result streaming
- **Large directory handling** — Confirmation dialog for directories exceeding threshold, with "remember" rules
- **Skipped directories log** — Review which directories were skipped and why
- **Persistent settings** — Large dir threshold, scan/skip rules, display preferences, saved across sessions
- **Theme support** — Light, Dark, and System theme modes
- **Keyboard shortcuts** — `Cmd/Ctrl+B` toggle sidebar, `Cmd/Ctrl+Enter` start scan
- **Sidebar layout** — Collapsible sidebar with file type presets, scan path, and OCR toggle
- **Settings panel** — Configure scan threshold, display options, and file type extension rules
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
| Frontend | React 19, TypeScript, Vite 7, Zustand 5, Lucide Icons |
| Backend | Rust, Tauri 2, Walkdir, Rayon, Kamadak-exif, Trash |
| Build | pnpm, tauri-cli |
| CI/CD | GitHub Actions |

## Search Engine Architecture

The scan engine uses a concurrent **BFS + Rayon thread pool** design:

1. **BFS thread** traverses the directory tree, classifying directories by size
2. **Dispatcher thread** feeds work items from a channel into the Rayon thread pool
3. **Rayon thread pool** (CPU-bound) performs file matching in parallel — filename, text content, EXIF, and OCR
4. **Result/Progress handler thread** streams results back to the frontend in real-time

Large directories (>1000 entries by default) trigger a confirmation dialog before scanning, with an optional "remember" rule system.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl + B` | Toggle sidebar |
| `Cmd/Ctrl + Enter` | Start scan |
| `Enter` | Submit search input |

## Project Structure

```
search/
├── src/                          # React frontend
│   ├── App.tsx                   # Main application component
│   ├── App.css                   # Application styles
│   ├── index.css                 # Theme variables and base styles
│   ├── main.tsx                  # React entry point
│   ├── components/
│   │   ├── Sidebar/              # Collapsible sidebar with presets
│   │   ├── SearchBar/            # Search input and scan controls
│   │   ├── ResultsView/          # Tree view with file results
│   │   ├── ScanProgress/         # Scan progress indicator
│   │   ├── PresetSelector/       # File type preset configuration
│   │   ├── EmptyState/           # Empty state placeholder
│   │   └── modals/
│   │       ├── ConfirmPanel.tsx   # Large directory confirmation dialog
│   │       ├── SkippedDirsPanel.tsx # Skipped directories review
│   │       ├── SettingsPanel.tsx  # Application settings
│   │       ├── FilePreviewModal.tsx # Text/code file preview
│   │       └── ImagePreviewModal.tsx # Image preview with OCR highlights
│   ├── store/
│   │   └── index.ts              # Zustand state management
│   ├── types/
│   │   └── index.ts              # TypeScript type definitions
│   ├── constants/
│   │   └── presets.ts            # File type presets configuration
│   ├── utils/
│   │   ├── storage.ts            # Settings persistence
│   │   └── file.ts               # File utility functions
│   └── assets/                   # Static assets
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── lib.rs                # Tauri builder, plugin setup, command registration
│   │   ├── main.rs               # Application entry point
│   │   ├── types.rs              # Shared data structures
│   │   ├── config.rs             # Persistent configuration management
│   │   ├── scanner.rs            # File scanning engine (BFS + Rayon)
│   │   └── commands/
│   │       ├── mod.rs            # Command module declarations
│   │       ├── scan.rs           # Scan lifecycle commands
│   │       ├── file_ops.rs       # File operations (preview, trash, reveal)
│   │       └── system.rs         # System commands (sound, config file)
│   ├── resources/                # OCR scripts, sounds, etc.
│   ├── Cargo.toml                # Rust dependencies
│   └── tauri.conf.json           # Tauri configuration
├── docs/                         # Architecture and product documentation
├── .github/workflows/            # CI/CD release pipeline
├── DESIGN.md                     # Apple-style design system documentation
├── ARCHITECTURE.md               # System architecture overview
├── package.json
└── vite.config.ts
```

## License

MIT
