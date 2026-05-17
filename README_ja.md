<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="128" height="128" alt="FileScope Icon" />
</p>

<h1 align="center">FileScope</h1>

<p align="center">
  <strong>Tauri 2 + React で構築された高速ネイティブファイル検索・管理ツール</strong>
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

## 機能一覧

- **マルチモード検索** — ファイル名、テキスト内容、EXIF メタデータ、OCR 文字認識（macOS）に対応
- **ファイルタイププリセット** — ドキュメント、コード、画像、設定ファイルの4カテゴリでクイックフィルター
- **ツリービュー** — 階層フォルダツリーで展開/折りたたみナビゲーション
- **ファイルプレビュー** — テキスト、コード、画像ファイルをアプリ内でプレビュー
- **一括管理** — 複数ファイルを選択してゴミ箱へ一括移動
- **ネイティブパフォーマンス** — Rust パワードの検索エンジン、マルチスレッドスキャン
- **テーマサポート** — ライト、ダーク、システム連携の3種類のテーマ
- **キーボードショートカット** — `Cmd/Ctrl+B` サイドバー切替、`Cmd/Ctrl+Enter` スキャン開始
- **サイドバーレイアウト** — ファイルタイプ設定の折りたたみ可能サイドバー
- **Finderで表示** — システムファイルマネージャーで直接ファイル位置を開く
- **クロスプラットフォーム** — macOS、Windows、Linux に対応

## 検索モード

| モード | 説明 |
|--------|------|
| **ファイル名** | ファイル名とフォルダ名のキーワードマッチ |
| **テキスト内容** | テキストベースファイル（コード、ドキュメント、設定）内を検索 |
| **EXIFデータ** | 画像のEXIFメタデータ（カメラ、レンズ、GPS等）を検索 |
| **OCRテキスト** | Vision フレームワークで画像からテキストを抽出して検索（macOS のみ） |

## クイックスタート

### 前提条件

- [Node.js](https://nodejs.org/) >= 20
- [pnpm](https://pnpm.io/) >= 9
- [Rust](https://www.rust-lang.org/) >= 1.77
- **macOS**: Xcode コマンドラインツール
- **Windows**: Microsoft Visual Studio C++ ビルドツール
- **Linux**: `libgtk-3-dev`、`libwebkit2gtk-4.1-dev`、`libappindicator3-dev`、`librsvg2-dev`

### 開発

```bash
# 依存関係のインストール
pnpm install

# 開発サーバーの起動
pnpm tauri dev

# プロダクションビルド
pnpm tauri build
```

### Releases からインストール

[GitHub Releases](https://github.com/GuoAccount/search/releases) からお使いのプラットフォームのインストーラーをダウンロード：

| プラットフォーム | フォーマット |
|-----------------|-------------|
| macOS (Apple Silicon) | `.dmg`、`.app` |
| macOS (Intel) | `.dmg`、`.app` |
| Windows | `.msi`、`.nsis` |
| Linux | `.deb`、`.AppImage` |

## 技術スタック

| レイヤー | 技術 |
|---------|------|
| フロントエンド | React 19、TypeScript、Vite 7、Lucide Icons |
| バックエンド | Rust、Tauri 2、Walkdir、Rayon、Kamadak-exif |
| ビルド | pnpm、tauri-cli |
| CI/CD | GitHub Actions |

## キーボードショートカット

| ショートカット | アクション |
|---------------|-----------|
| `Cmd/Ctrl + B` | サイドバーの切替 |
| `Cmd/Ctrl + Enter` | スキャン開始 |
| `Enter` | 検索入力の送信 |

## プロジェクト構成

```
search/
├── src/                    # React フロントエンド
│   ├── App.tsx             # メインアプリケーションコンポーネント
│   ├── App.css             # アプリケーションスタイル
│   └── index.css           # テーマ変数と基本スタイル
├── src-tauri/              # Rust バックエンド
│   ├── src/
│   │   ├── lib.rs          # Tauri コマンドとアプリエントリ
│   │   └── scanner.rs      # ファイルスキャンエンジン
│   ├── resources/          # OCR スクリプト、サウンド等
│   ├── Cargo.toml          # Rust 依存関係
│   └── tauri.conf.json     # Tauri 設定
├── .github/workflows/      # CI/CD リリースパイプライン
├── DESIGN.md               # Apple スタイルデザインシステム
├── package.json
└── vite.config.ts
```

## ライセンス

MIT
