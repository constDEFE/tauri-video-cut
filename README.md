# VideoCut

Fast, lossless video cutting tool built with Tauri, Rust, and Preact.

> **Windows only** — designed to run exclusively on Windows systems.

---

## Backstory

TL;DR — the entire story behind this app is just me running out of free space on my SSDs.

Everyone remembers those moments where it's you, your company, and a funny thing — I love these. Jokes, memes, weird conversations, cringe moments — each of these is something it's great to someday look back on. Since this seed was planted in my mind, I started recording every session: every time my PC boots, I immediately start OBS and click record.

But high-quality recordings come at a price, and that price can be huge.

The problem with recording everything is that only 5–10% of an actual recording is anything worth keeping. Some might say there are already tools for this: OBS Replay, Nvidia Instant Replay, etc. They solve the space problem, but have drawbacks:

1. Fixed, preconfigured duration
2. Could overlap with previous if the duration since last save hasn't elapsed
3. Each time you want to save the moment, you have to manually open the app, overlay, or press some keyboard combination

Not quite convenient. I stuck with plain recordings.

By the time I had around 30GB of free space, I discovered [LosslessCut](https://github.com/mifi/lossless-cut) — saw its huge size (~600MB) and decided to build my own. It may not be perfect: may not support other systems or many video formats/codecs, may not work perfectly on specific hardware — but it solves my specific use case, does it relatively fast, and has a small footprint.

## Regarding AI

95% of the Rust code is written with AI — I'm a Frontend Engineer, not a Native Apps Developer. Despite that, the entire process was end-to-end orchestrated by me: diving into FFmpeg and mpv configurations, codecs, encoders, hardware acceleration methods, stripping binaries to a bare minimum, weighing each trade-off — all to achieve the end result I wanted.

## License

This project's original source code is licensed under the [**Apache License 2.0**](./LICENSE).

However, when building or distributing compiled binaries, the resulting software is governed by the [**GNU General Public License v3.0 (GPLv3)**](https://opensource.org/license/gpl-3.0) because the application dynamically/statically links to custom-built configurations of FFmpeg and mpv compiled with their GPL flags enabled, alongside various native system runtime libraries.

Complete legal compliance details and the full license texts for all compiled Rust dependencies, Node frontend packages, and bundled third-party binaries are organized in:

- [`THIRDPARTY_RUST.txt`](./THIRDPARTY_RUST.txt)
- [`THIRDPARTY_NODE.txt`](./THIRDPARTY_NODE.txt)
- [`THIRDPARTY_BINARIES.txt`](./THIRDPARTY_BINARIES.txt)

These files are included in both this repository and the release artifacts. In accordance with GPLv3, the complete build configuration and full source code remain openly available in this GitHub repository.

### Notice & Contact

All third-party attribution blocks and legal texts are compiled automatically via automated build scripts using dependency manifests and system package databases. If you identify any discrepancies, missing notices, or have any other legal or licensing concerns, reach out at [constant_defe@pm.me](mailto:constant_defe@pm.me) or open a GitHub Issue.

## Features

- **Lossless Cutting** — stream copy mode for instant cuts without re-encoding
- **Smart Cut** — intelligent keyframe-aware cutting to maintain quality
- **Multi-Segment Export** — cut multiple segments from a single video in one operation
- **Audio Track Selection** — choose which audio tracks to include in export
- **Embedded Player** — libmpv-powered video player with native-like experience
- **Hardware Acceleration** — automatic detection and use of hardware encoders/decoders (cuda, d3d11va, dxva2)
- **Waveform Visualization** — audio waveform rendered on the timeline
- **Session Management** — save and restore editing sessions
- **Logging** — log files saved at `%temp%/io.github.constdefe.tauri-video-cut`
- **Formats** — MP4, MKV, MOV, AVI, WebM

## Tech Stack

**Frontend**: Preact, TypeScript, Tailwind CSS v4, Zustand, Vite, React Router, Base UI, Sonner, clsx, tailwind-merge, nanoid, @tanstack/react-hotkeys, tauri-plugin-libmpv

**Backend**: Rust 2024, Tauri v2, FFmpeg/FFprobe (custom-built), libmpv, tokio, dashmap, windows-sys

**Dev Tools**: Bun, oxlint, oxfmt

## Architecture

### Frontend — Feature-Sliced Design (FSD)

The frontend follows [Feature-Sliced Design](https://feature-sliced.design/) with the following layers and slices:

```
src/
├── app/                    # App shell: router, global styles, root error boundary
├── entities/               # Domain entities
│   ├── config/             # Application configuration
│   ├── segments/           # Cut segments
│   ├── session/            # Editing session state
│   └── video/              # Video metadata, player hooks, waveform
├── features/               # User-facing features
│   ├── editor/             # Editor features
│   │   ├── playback/       # Play/pause, volume, audio track selection
│   │   ├── player/         # libmpv player integration
│   │   ├── segment-edit/   # Add/remove/split segments
│   │   ├── theme-switch/   # Light/dark theme toggle
│   │   ├── timeline/       # Timeline canvas rendering
│   │   └── waveform/       # Waveform controller
│   ├── export/             # Export form and execution
│   └── import/             # File import and session modal
├── widgets/                # Composite UI sections
│   ├── editor/             # Editor panel with preview and sidebar
│   └── export/             # Export form and progress widgets
├── pages/                  # Route-level pages
│   └── export/             # Export complete page
└── shared/                 # Cross-cutting utilities
    ├── lib/                # Shared libraries (mpv, theme)
    ├── types/              # Shared type definitions
    ├── ui/                 # Shared UI components and icons
    └── utils/              # Shared utilities
```

### Backend — Rust

The backend is organized into the following modules:

```
src-tauri/src/
├── commands/               # Tauri command handlers
│   ├── config.rs           # Configuration commands
│   ├── export.rs           # Export operation commands
│   ├── metadata.rs         # Video metadata commands
│   └── session.rs          # Session management commands
├── config/                 # Configuration model and storage
├── core/                   # Core business logic
│   ├── ffmpeg/             # FFmpeg operations
│   │   ├── executor.rs     # FFmpeg process execution
│   │   ├── hwaccel.rs      # Hardware acceleration detection
│   │   ├── keyframes.rs    # Keyframe extraction
│   │   ├── mp4_parser.rs   # MP4 container parsing
│   │   ├── probe.rs        # Video file probing
│   │   └── smart_cut.rs    # Smart cut algorithm
│   ├── system/             # System-level operations
│   │   └── job_object.rs   # Windows job objects for process management
│   └── waveform/           # Audio waveform generation
│       ├── cache.rs        # Waveform cache
│       ├── command.rs      # Waveform command handling
│       ├── engine.rs       # Waveform generation engine
│       ├── model.rs        # Waveform data model
│       └── registry.rs     # Waveform registry
├── session/                # Session model and storage
├── types/                  # Type definitions
├── utils/                  # Utilities (atomic writes, cleanup, DLL loading, paths)
├── error.rs                # Error types
├── logger.rs               # Logging setup
├── lib.rs                  # Library entry point
└── main.rs                 # Application entry point
```

### Binary Management

All binaries (FFmpeg, FFprobe, libmpv DLLs) are stored in `src-tauri/lib/`. Tauri bundles them as resources via `tauri.conf.json`. At runtime:

- **libmpv plugin** loads DLLs directly from bundled `lib/` directory
- **FFmpeg/FFprobe** accessed via `app_handle.path().resolve()`

### Export Modes

#### Smart Cut Algorithm

**When cut points are on keyframes:**
```
K1C1_______K2________K3________K4C1
```
1. Just stream copy

**When cut points not on keyframes:**
```
K1__C1__K2_______K3__C2__K4
```
1. Find closest keyframes outside cut boundaries (K1, K4)
2. Re-encode to the closest keyframes inside boundaries (K1-K2, K4-K3)
3. Stream copy first to last keyframes inside boundaries (K2-K3)
4. Concat
5. Trim to cut points (C1, C2)

#### Stream Copy Algorithm

**When cut points are on keyframes:**
```
K1C1_______K2________K3________K4C1
```
1. Just stream copy

**When cut points not on keyframes:**
```
K1__C1__K2_______K3__C2__K4
```
1. Find closest keyframes outside cut boundaries (K1, K4)
2. Stream copy (K1-K4)

## Prerequisites

### Development

- **Rust** 1.85+ (via rustup)
- **Node.js** 18+ or **Bun** (recommended)
- **FFmpeg/libmpv libraries** (Windows MSYS2)
- **MSYS2 MINGW64 terminal**

## Commands

### Frontend Development

```bash
bun run dev          # Start Vite dev server
bun run build        # Build frontend
bun run preview      # Preview production build
bun run lint         # Oxlint check
bun run lint:fix     # Oxlint fix
bun run format       # Oxfmt format
bun run format:check # Oxfmt check
bun run check        # Full check (TS + Oxlint + Oxfmt)
```

### Tauri

```bash
bun run tauri dev    # Run app in dev mode (devtools auto-open)
bun run tauri build  # Build production app
```

## Building from Source

1. Clone repo
2. Install prerequisites (Rust, Bun, MSYS2 libraries)
3. Run scripts in `scripts/` to build custom FFmpeg and libmpv binaries:
   - `1-setup-env.sh` — environment setup
   - `2-install-nvidia-headers.sh` — NVIDIA codec headers
   - `3-build-ff.sh` — build FFmpeg
   - `4-build-mpv.sh` — build libmpv
   - `5-harvest-all.sh` — collect and organize all binaries
4. Build the app: `bun run tauri build`
5. Installer found in `src-tauri/target/release/bundle/`

## Changelog

See [CHANGELOG.md](./CHANGELOG.md) for the full history.

- **[1.1.1]** — Reverted keyframe extraction by packets (frames proved worse performance)
- **[1.1.0]** — Segment export progress, auto devtools, scrollbar styling
- **[1.0.0]** — Initial release

## Repository

[github.com/constDEFE/tauri-video-cut](https://github.com/constDEFE/tauri-video-cut)
