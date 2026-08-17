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

Not quite convenient and so I decided to stick with plain recordings.

By the time I had around 30GB of free space, I discovered [LosslessCut](https://github.com/mifi/lossless-cut) — saw its huge size (~600MB) and decided to build my own. It may not be perfect: may not support other systems or many video formats/codecs, may not work perfectly on specific hardware — but it solves my specific use case, does it relatively fast, and has a small footprint.

## Regarding AI

95% of the Rust code is written with AI — I'm a Frontend Engineer, not a Native Apps Developer. Despite that, the entire process was end-to-end orchestrated by me: diving into FFmpeg and mpv configurations, codecs, encoders, hardware acceleration methods, stripping binaries to a bare minimum, weighing each trade-off — all to achieve the end result I wanted.

## License

This project's original source code is licensed under the [**Apache License 2.0**](./LICENSE).

However, when building or distributing compiled binaries, the resulting software is governed by the [**GNU General Public License v2.0 (GPLv2)**](https://opensource.org/license/gpl-2-0/) because the application dynamically/statically links to custom-built configurations of FFmpeg and mpv compiled with their GPL flags enabled, alongside various native system runtime libraries. See [`LICENSE.DISTRIBUTION`](./LICENSE.DISTRIBUTION) and [`SOURCE-OFFER.md`](./SOURCE-OFFER.md).

Complete legal compliance details and the full license texts for all compiled Rust dependencies, Node frontend packages, MSYS2 system packages, and bundled third-party binaries are organized in:

- [`NOTICE.md`](./NOTICE.md) — consolidated legal notices
- [`legal/`](./legal) — generated attributions (Rust, frontend, MSYS2, FFmpeg, libmpv) and build environment
- [`LICENSES/`](./LICENSES) — full texts of the referenced licenses

These files are included in both this repository and the release artifacts. In accordance with GPLv2 Section 3, the complete build configuration and full source code of all GPL components remain openly available in this GitHub repository.

### Notice & Contact

All third-party attribution blocks and legal texts are compiled automatically via automated build scripts using dependency manifests and system package databases. If you identify any discrepancies, missing notices, or have any other legal or licensing concerns, reach out at [constant_defe@pm.me](mailto:constant_defe@pm.me) or open a GitHub Issue.

## Features

- **Lossless Cutting** — stream copy mode for instant cuts without re-encoding
- **Smart Cut** — intelligent keyframe-aware cutting to maintain quality
- **Multi-Segment Export** — cut multiple segments from a single video in one operation
- **Audio Track Selection** — choose which audio tracks to include in export
- **Embedded Player** — libmpv-powered video player with native-like experience
- **Waveform Visualization** — audio waveform rendered on the timeline
- **Session Management** — save and restore editing sessions
- **Export Cancellation** — cancel an in-progress export at any time
- **Zoom** — persistent UI zoom scaling
- **Logging** — log files saved at `%temp%/io.github.constdefe.tauri-video-cut`
- **Formats** — MP4, MKV, MOV, AVI, WebM

## Tech Stack

**Frontend**: Preact, TypeScript, Tailwind CSS v4, Zustand, Vite, Base UI, Sonner, clsx, tailwind-merge, nanoid, @tanstack/react-hotkeys, tauri-plugin-libmpv-api, Tauri plugins (dialog, fs, opener, shell, prevent-default)

**Backend**: Rust 2024 (1.97.1+), Tauri v2, FFmpeg 9.0.1/FFprobe (custom-built), libmpv 0.41.0, tokio, dashmap, windows-sys

**Dev Tools**: Bun, oxlint, oxfmt

## Architecture

### Frontend — Feature-Sliced Design (FSD)

The frontend follows [Feature-Sliced Design](https://feature-sliced.design/) with the following layers and slices:

```
src/
├── app/                    # App shell: router, global styles, root error boundary
├── entities/               # Domain entities
│   ├── config/             # Application configuration (theme, zoom)
│   ├── segments/           # Cut segments
│   ├── session/            # Editing session state
│   ├── video/              # Video metadata and player hooks
│   └── waveform/           # Waveform state and job sequencing
├── features/               # User-facing features
│   ├── editor/             # Editor features
│   │   ├── export-controls/ # Export and re-import entry controls (hotkeys)
│   │   ├── playback/        # Play/pause, volume, audio track selection
│   │   ├── player/          # libmpv player integration
│   │   ├── segment-edit/    # Add/remove/split segments
│   │   ├── theme-switch/    # Light/dark theme toggle
│   │   ├── timeline/        # Timeline canvas rendering (DPR-aware)
│   │   └── waveform/        # Waveform controller
│   ├── export/             # Export form, execution, progress, completion output
│   └── import/             # File import and session modal
├── widgets/                # Composite UI sections
│   └── editor/             # Editor panel with preview and sidebar
├── pages/                  # Route-level pages (editor, import, export flow)
└── shared/                 # Cross-cutting utilities
    ├── lib/                # Shared libraries (mpv client, custom router, theme, zoom)
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
│   │   ├── executor.rs     # FFmpeg process execution (ProcessManager)
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
├── utils/                  # Utilities (atomic writes, cleanup, subprocess helpers, DLL loading, fs, paths)
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

**When both cut points are on keyframes:**

```
K1C1_______K2________K3________K4C1
```

1. Just stream copy

**When a cut point is off a keyframe, and valid intermediate keyframes exist (K2 < K3):**

```
K1__C1__K2_______K3__C2
```

- `K1` — last keyframe before `C1` · `K2` — first keyframe after `C1` · `K3` — last keyframe before `C2`

1. Head _(if `C1` off keyframe)_: decode from `K1`, trim to `C1`, re-encode `C1→K2` (video only)
2. Middle: stream copy `K2→K3` (video only)
3. Tail _(if `C2` off keyframe)_: decode from `K3`, re-encode `K3→C2`, forcing the first frame to a keyframe (video only)
4. Concat video parts
5. Validate each concat boundary with a strict decode; if the re-encoded parts are not parameter-compatible with the source, fall back to FullEncode for the whole segment
6. Mux the stream-copied audio from the source over the final video — audio is never re-encoded, so no a/v drift

Re-encoding uses the source codec's profile/level/pixel format/color range as encoder hints so the output stays concat-compatible with the copied middle.

**When no valid intermediate keyframes exist (e.g. the segment fits inside a single GOP):**

1. **FullEncode** — re-encode the entire `C1→C2` (decode starting from `K1`), audio stream-copied

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

- **Rust** 1.97.1+ (via rustup)
- **Node.js** 18+ or **Bun** (recommended)
- **MSYS2 UCRT64 toolchain and libraries** (Windows)
- **MSYS2 UCRT64 terminal**

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

### Legal

```bash
bun run legal        # Generate third-party license attributions into legal/
```

## Building from Source

1. Clone repo
2. Install prerequisites (Rust, Bun, MSYS2 libraries)
3. Run scripts in `scripts/build/` (from the MSYS2 UCRT64 terminal) to build custom FFmpeg and libmpv binaries:
   - `1-setup-env.sh` — environment setup
   - `2-build-ff.sh` — build FFmpeg
   - `3-build-mpv.sh` — build libmpv
   - `4-harvest-all.sh` — collect and organize all binaries
4. Build the app: `bun run tauri build`
5. Installer found in `src-tauri/target/release/bundle/`

## Changelog

See [CHANGELOG.md](./CHANGELOG.md) for the full history.

- **[1.1.1]** — Reverted keyframe extraction by packets (frames proved worse performance)
- **[1.1.0]** — Segment export progress, auto devtools, scrollbar styling
- **[1.0.0]** — Initial release

## Repository

[github.com/constDEFE/tauri-video-cut](https://github.com/constDEFE/tauri-video-cut)
