<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="128" height="128" alt="Lumina Icon" />
</p>

<h1 align="center">Lumina</h1>

<p align="center">
  <em>파일을 비추다</em><br>
  <code>OCR</code> <code>전체 텍스트 검색</code> <code>EXIF</code> <code>Tauri</code> <code>macOS</code>
</p>

<p align="center">
  <strong>Tauri 2 + React로 구축된 고속 네이티브 파일 검색 및 관리 도구</strong>
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

## 기능

- **다중 모드 검색** — 파일명, 텍스트 내용, EXIF 메타데이터, OCR 문자 인식 (macOS) 지원
- **파일 타입 프리셋** — 문서, 코드, 이미지, 설정 파일 4가지 카테고리 빠른 필터
- **트리 뷰** — 계층 폴더 트리로 확장/축소 탐색
- **파일 미리보기** — 텍스트, 코드, 이미지 파일을 앱 내에서 미리보기
- **일괄 관리** — 여러 파일을 선택하여 휴지통으로 한 번에 이동
- **네이티브 성능** — Rust 기반 검색 엔진, 멀티스레드 스캔
- **테마 지원** — 라이트, 다크, 시스템 연동 3가지 테마 모드
- **키보드 단축키** — `Cmd/Ctrl+B` 사이드바 전환, `Cmd/Ctrl+Enter` 스캔 시작
- **사이드바 레이아웃** — 파일 타입 설정이 가능한 접이식 사이드바
- **Finder에서 보기** — 시스템 파일 관리자에서 바로 파일 위치 열기
- **크로스 플랫폼** — macOS, Windows, Linux 지원

## 검색 모드

| 모드 | 설명 |
|------|------|
| **파일명** | 파일명 및 폴더명에서 키워드 매칭 |
| **텍스트 내용** | 텍스트 기반 파일(코드, 문서, 설정) 내부 검색 |
| **EXIF 데이터** | 이미지 EXIF 메타데이터(카메라, 렌즈, GPS 등) 검색 |
| **OCR 텍스트** | Vision 프레임워크로 이미지에서 텍스트 추출 후 검색 (macOS 전용) |

## 시작하기

### 사전 요구사항

- [Node.js](https://nodejs.org/) >= 20
- [pnpm](https://pnpm.io/) >= 9
- [Rust](https://www.rust-lang.org/) >= 1.77
- **macOS**: Xcode 커맨드라인 도구
- **Windows**: Microsoft Visual Studio C++ 빌드 도구
- **Linux**: `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`

### 개발

```bash
# 의존성 설치
pnpm install

# 개발 서버 시작
pnpm tauri dev

# 프로덕션 빌드
pnpm tauri build
```

### Releases에서 설치

[GitHub Releases](https://github.com/GuoAccount/search/releases)에서 사용 중인 플랫폼의 설치 파일을 다운로드:

| 플랫폼 | 형식 |
|--------|------|
| macOS (Apple Silicon) | `.dmg`, `.app` |
| macOS (Intel) | `.dmg`, `.app` |
| Windows | `.msi`, `.nsis` |
| Linux | `.deb`, `.AppImage` |

## 기술 스택

| 계층 | 기술 |
|------|------|
| 프론트엔드 | React 19, TypeScript, Vite 7, Lucide Icons |
| 백엔드 | Rust, Tauri 2, Walkdir, Rayon, Kamadak-exif |
| 빌드 | pnpm, tauri-cli |
| CI/CD | GitHub Actions |

## 키보드 단축키

| 단축키 | 동작 |
|--------|------|
| `Cmd/Ctrl + B` | 사이드바 전환 |
| `Cmd/Ctrl + Enter` | 스캔 시작 |
| `Enter` | 검색 입력 제출 |

## 프로젝트 구조

```
search/
├── src/                    # React 프론트엔드
│   ├── App.tsx             # 메인 애플리케이션 컴포넌트
│   ├── App.css             # 애플리케이션 스타일
│   └── index.css           # 테마 변수 및 기본 스타일
├── src-tauri/              # Rust 백엔드
│   ├── src/
│   │   ├── lib.rs          # Tauri 명령 및 앱 엔트리
│   │   └── scanner.rs      # 파일 스캔 엔진
│   ├── resources/          # OCR 스크립트, 사운드 등
│   ├── Cargo.toml          # Rust 의존성
│   └── tauri.conf.json     # Tauri 설정
├── .github/workflows/      # CI/CD 릴리스 파이프라인
├── DESIGN.md               # Apple 스타일 디자인 시스템 문서
├── package.json
└── vite.config.ts
```

## 라이선스

MIT
