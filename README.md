# VideoCut

Fast, lossless video cutting tool built with Tauri, Rust, and Preact.

## Backstory

TL;DR - the entire story behind this app is just me running out of free space on my ssds

Everyone remembers those moments where it's you, your company, and a funny thing - I love these. Jokes, memes, wierd
conversations, cringe moments - each of these are things it's great to someday look back on. And ever since this thought
came to my mind I literally became a living CCTV - every time my pc boots I immediately start OBS and click on a record button.
But high quality recordings come at a price and this price could be really huge

The problem with me recording everything is that only 5-10% of an actual recording is something happening. Some could
say there's already tools for this: OBS' Replay, Nvidia's Instant Replay, etc. (I believe there's more). And while these
do fundamentally solve space problem, there are drawbacks:

1. Fixed preconfigured duration
2. Could overlap with previous if the duration since last save hasn't elapsed
3. Each time you want to save the moment you have to manually either open the app, overlay, whatever,
   and click save, or press some keyboard combination

Not quite convenient for me and so I stuck with plain recordings

By the time I had around 30Gb of total free space on my machine, I discovered [LosslessCut](https://github.com/mifi/lossless-cut) tool.
I saw its huge size (~600Mb) and decided to build my own. It may not be perfect: may not support other system than windows, many video
formats/codecs, may not work perfectly on specific hardware - but it does solve my specific use-case, does it relatively fast, and has
small fingerprint on a system

## Features

- **Lossless Cutting**: Stream copy mode for instant cuts without re-encoding
- **Smart Cut**: Intelligent keyframe-aware cutting to maintain quality
- **Multi-Segment Export**: Cut multiple segments from single video in one operation
- **Audio Track Selection**: Choose which audio tracks to include in export
- **Embedded Player**: libmpv-powered video player - native-like experience
- **Hardware Acceleration**: Automatic detection and use of hardware encoders/decoders (cuda, d3d11va, dxva2)
- **Logging**: Log files saved at `%temp%/io.github.constdefe.tauri-video-cut`
- **Multiple Formats**: Supports MP4, MKV, MOV, AVI, WebM

## Technology Stack

**Frontend**: Preact, TypeScript, Tailwind CSS, Zustand, Vite, tauri-plugin-libmpv \
**Backend**: Rust, Tauri, FFmpeg/FFprobe, libmpv

## Architecture

### Binary Management

All binaries (FFmpeg, FFprobe, libmpv DLLs) stored in `src-tauri/lib/`. Tauri bundles them as resources via `tauri.conf.json`. At runtime:

- **libmpv plugin** loads DLLs directly from bundled `lib/` directory
- **FFmpeg/FFprobe** accessed via `app_handle.path().resolve()`

### Export Modes

#### Smart Cut Algorithm

When cut points are on keyframes: \
K1C1\_\_\_\_\_K2\_\_\_\_\_K3\_\_\_\_\_K4C1

1. Just stream copy

When cut points not on keyframes: \
K1\_\_C1\_\_K2\_\_\_\_\_K3\_\_C2\_\_K4

1. Find closest keyframes outside cut boundaries (K1, K4)
2. Re-encode to the closest keyframes inside boundaries (K1-K2, K4-K3)
3. Stream copy first to last keyframes inside boundaries (K2-K3)
4. Concat
5. Trim to cut points (C1, C2)

#### Stream Copy Algorithm

When cut points are on keyframes: \
K1C1\_\_\_\_\_K2\_\_\_\_\_K3\_\_\_\_\_K4C1

1. Just stream copy

When cut points not on keyframes: \
K1\_\_C1\_\_K2\_\_\_\_\_K3\_\_C2\_\_K4

1. Find closest keyframes outside cut boundaries (K1, K4)
2. Stream copy (K1-K4)

## Prerequisites

### Development

- **Rust** 1.85+ (via rustup)
- **Node.js** 18+ or **Bun** (recommended)
- **FFmpeg/libmpv libraries** (Windows MSYS2):
- **MSYS2 MINGW64 terminal**

## Project Structure

```
tauri-video-cut/
├── scripts/                # Shell scripts to setup environment
├── src/                    # Frontend (Preact/TS)
│   ├── components/         # UI components
│   ├── hooks/              # React hooks
│   ├── stores/             # Zustand stores
│   └── utils/              # Helper functions
├── src-tauri/              # Backend (Rust)
│   ├── src/
│   │   ├── commands/       # Tauri command handlers
│   │   │   ├── export.rs   # Segment export logic
│   │   │   └── metadata.rs # Video metadata extraction
│   │   ├── ffmpeg/         # FFmpeg integration
│   │   │   ├── executor.rs # Command execution
│   │   │   ├── hwaccel.rs  # Hardware acceleration
│   │   │   ├── keyframes.rs# Keyframe analysis
│   │   │   └── probe.rs    # Video probing
│   │   ├── config.rs       # App configuration
│   │   ├── error.rs        # Error types
│   │   ├── models.rs       # Data structures
│   │   └── lib.rs          # Main entry + logging setup
│   │   └── logger.rs       # Logging setup
│   ├── lib/                # FFmpeg/libmpv binaries (bundled as resources)
│   └── tauri.conf.json     # Tauri config
└── package.json
```

## Commands

### Frontend Development

```bash
bun run dev          # Start Vite dev server
bun run build        # Build frontend
bun run preview      # Preview production build
bun run lint         # ESLint check
bun run lint:fix     # ESLint fix
bun run format       # Prettier format
bun run format:check # Prettier check
bun run check        # Full check (TS + ESLint + Prettier)
```

### Tauri

```bash
bun run tauri dev    # Run app in dev mode
bun run tauri build  # Build production app
```

## Building from Source

1. Clone repo
2. Install prerequisites (Rust, Bun, MSYS2 libraries)
3. Run scripts in `scripts` to build binaries
4. Installer in `src-tauri/target/release/bundle/`