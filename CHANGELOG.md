# Changelog

## [1.2.1] - 2026-08-27

### Fixed

- Fix verbose terminal window when generating waveform ([e237b1d](https://github.com/constDEFE/tauri-video-cut/commit/e237b1d0cbbd10cb49a1198d280ffdd5053bac91)) ([@constDEFE](https://github.com/constDEFE))

## [1.2.0] - 2026-08-18

### Added

- Add editing sessions: save and restore editing state ([98b5499](https://github.com/constDEFE/tauri-video-cut/commit/98b549909e81f41c352c59295f911dae13c77e99), [c88ee9c](https://github.com/constDEFE/tauri-video-cut/commit/c88ee9cf823ee2bde3f6999f126daa29b32d5216)) ([@constDEFE](https://github.com/constDEFE))
- Add persistent UI zoom scaling via `zoomScale` config field with timeline DPR support ([959e8c4](https://github.com/constDEFE/tauri-video-cut/commit/959e8c4f66ff45e0f6a012d9bddbc037cea978f0)) ([@constDEFE](https://github.com/constDEFE))
- Add button to cancel an in-progress export ([9f51727](https://github.com/constDEFE/tauri-video-cut/commit/9f51727c2c45cc70243d3dac0f25777f1e92132a)) ([@constDEFE](https://github.com/constDEFE))
- Add audio waveforms: generation, streaming, resuming, and cancellation across backend and frontend ([ef03c83](https://github.com/constDEFE/tauri-video-cut/commit/ef03c831917d0a3a0ff31afacdc65c633529c9fe)) ([@constDEFE](https://github.com/constDEFE))
- Add RMS+Peak (envelope) waveform rendering ([bd867dc](https://github.com/constDEFE/tauri-video-cut/commit/bd867dc4ea514722a865e18dee4a45a17d983dd2), [6b1e253](https://github.com/constDEFE/tauri-video-cut/commit/6b1e253f39e743ad750779bc6fe572b206fe6200), [454cd9e](https://github.com/constDEFE/tauri-video-cut/commit/454cd9e2fe89bbc62bc3d690fe1111a825e8002c), [616d118](https://github.com/constDEFE/tauri-video-cut/commit/616d118572cbc358121c29ba0831eb55b8754ec4), [ab58cf0](https://github.com/constDEFE/tauri-video-cut/commit/ab58cf022dcdcfad29856690904deb6a7cea4715)) ([@constDEFE](https://github.com/constDEFE))
- Add ProcessManager for managing subprocesses created by the Tauri window, such as FFmpeg and FFprobe ([bd6a278](https://github.com/constDEFE/tauri-video-cut/commit/bd6a27856496ab0d29dd135327f359fe396bbe46)) ([@constDEFE](https://github.com/constDEFE))
- Add atomic writes for config storage ([0b75992](https://github.com/constDEFE/tauri-video-cut/commit/0b759927d4508f42c341220a3591fec13233cd24)) ([@constDEFE](https://github.com/constDEFE))
- Add `temp_segment` cleanup on exit during export ([1f995b8](https://github.com/constDEFE/tauri-video-cut/commit/1f995b8a39021fb3d8ee54380c309b547340d79e)) ([@constDEFE](https://github.com/constDEFE))
- Add FullEncode mode for segments without intermediate keyframes, with boundary decode validation fallback ([0617252](https://github.com/constDEFE/tauri-video-cut/commit/061725269928599eb2777c44637b28ce9616bc44)) ([@constDEFE](https://github.com/constDEFE))
- Add proper licensing: `LICENSE`, `LICENSE.DISTRIBUTION`, `NOTICE.md`, `SOURCE-OFFER.md`, and full license texts under `LICENSES/` ([f205eee](https://github.com/constDEFE/tauri-video-cut/commit/f205eee77c42cc4da95557d110259c01ebab6a6e)) ([@constDEFE](https://github.com/constDEFE))
- Add scripts for automatic license acquisition for distributed packages and dependencies, GitHub release workflow, and on-main-push license check workflow ([f205eee](https://github.com/constDEFE/tauri-video-cut/commit/f205eee77c42cc4da95557d110259c01ebab6a6e)) ([@constDEFE](https://github.com/constDEFE))

### Changed

- Overhaul backend and frontend structure ([ef03c83](https://github.com/constDEFE/tauri-video-cut/commit/ef03c831917d0a3a0ff31afacdc65c633529c9fe)) ([@constDEFE](https://github.com/constDEFE))
- Replace logging solution with `tracing` crates and update logging across the backend ([ef03c83](https://github.com/constDEFE/tauri-video-cut/commit/ef03c831917d0a3a0ff31afacdc65c633529c9fe), [b6ebee1](https://github.com/constDEFE/tauri-video-cut/commit/b6ebee163a7909011f1a269928990fd8f218e6ed)) ([@constDEFE](https://github.com/constDEFE))
- Migrate build scripts from MINGW64 to UCRT64, prioritize AVX2 support, and collect fewer binaries ([ef03c83](https://github.com/constDEFE/tauri-video-cut/commit/ef03c831917d0a3a0ff31afacdc65c633529c9fe)) ([@constDEFE](https://github.com/constDEFE))
- Refactor keyframe lookup functions to use binary search ([660d8a6](https://github.com/constDEFE/tauri-video-cut/commit/660d8a60a793c3f99efee15ed97c40617d36a898)) ([@constDEFE](https://github.com/constDEFE))
- Improve metadata probing by reading only the first 10 packets instead of the entire file, with fallback for corrupted MKV/MP4 files ([eb4f202](https://github.com/constDEFE/tauri-video-cut/commit/eb4f202ca8c117a20542c8f31de4ed4852ab36aa)) ([@constDEFE](https://github.com/constDEFE))
- Switch keyframes extraction output from JSON to CSV and parallelize `probe_video` and `get_keyframes` calls ([1acd6b3](https://github.com/constDEFE/tauri-video-cut/commit/1acd6b39d4e451015cac8a2f482d6e5540b0de1c)) ([@constDEFE](https://github.com/constDEFE))
- Replace `react-router` with a custom router solution ([54c3d05](https://github.com/constDEFE/tauri-video-cut/commit/54c3d05682177ef4c783bb329620849f88623c6e)) ([@constDEFE](https://github.com/constDEFE))
- Disable browser default shortcuts via Tauri plugin ([2b60348](https://github.com/constDEFE/tauri-video-cut/commit/2b60348381eabf2f90ce8d7a62f2f3ec6558fbe4)) ([@constDEFE](https://github.com/constDEFE))
- Replace reactive `theme` state with a global `window` reference ([c3efb1c](https://github.com/constDEFE/tauri-video-cut/commit/c3efb1c13b9d7eb6e3f168827c5ce9c9d37c8217)) ([@constDEFE](https://github.com/constDEFE))
- Refactor smart-cut to use `,6` timestamp precision, validate concat boundaries with a strict decode, and hint encoders with source codec parameters ([0617252](https://github.com/constDEFE/tauri-video-cut/commit/061725269928599eb2777c44637b28ce9616bc44)) ([@constDEFE](https://github.com/constDEFE))
- Bump FFmpeg to v9.0.1 and migrate the codebase from FFmpeg v7 ([874d4d8](https://github.com/constDEFE/tauri-video-cut/commit/874d4d8bcf0328ab67cf44b1a140ae9286f74580)) ([@constDEFE](https://github.com/constDEFE))
- Bind libmpv to v0.41.0 ([874d4d8](https://github.com/constDEFE/tauri-video-cut/commit/874d4d8bcf0328ab67cf44b1a140ae9286f74580)) ([@constDEFE](https://github.com/constDEFE))
- Bump Rust to 1.97.1 ([7ee7081](https://github.com/constDEFE/tauri-video-cut/commit/7ee708171b68c1fa1686495a91c4557074db4980)) ([@constDEFE](https://github.com/constDEFE))

### Removed

- Remove ETA feature as it's practically impossible to estimate export time accurately for multiple segments ([0617252](https://github.com/constDEFE/tauri-video-cut/commit/061725269928599eb2777c44637b28ce9616bc44), [5873de3](https://github.com/constDEFE/tauri-video-cut/commit/5873de3c742209effebaec89d0d7095b14ef679f)) ([@constDEFE](https://github.com/constDEFE))
- Remove hardware encoder support due to compatibility issues, worse output quality at the same parameters, and maintenance burden ([4172655](https://github.com/constDEFE/tauri-video-cut/commit/41726550d7655e96bddbac23a94d97d468b68905)) ([@constDEFE](https://github.com/constDEFE))

### Fixed

- Ease backward seek smoothness ([c9f9122](https://github.com/constDEFE/tauri-video-cut/commit/c9f91228af3682f6517c125940b0d36c088aab40)) ([@constDEFE](https://github.com/constDEFE))
- Fix a/v sync issues and broken output file duration in smart-cut exports ([0617252](https://github.com/constDEFE/tauri-video-cut/commit/061725269928599eb2777c44637b28ce9616bc44)) ([@constDEFE](https://github.com/constDEFE))

## [1.1.1] - 2026-06-15

### Fixed

- Revert extraction of keyframes by packets as frames proved worse performance despite using less RAM ([d015dfb](https://github.com/constDEFE/tauri-video-cut/commit/d015dfbc6e339e953d7c61494fb41543d8602673)) ([@constDEFE](https://github.com/constDEFE))

## [1.1.0] - 2026-06-14

### Added

- Add detailed status on segments export progress ([c2235d0](https://github.com/constDEFE/tauri-video-cut/commit/c2235d0ceeae1e300248f463e63636a711eb1423), [2a2ec90](https://github.com/constDEFE/tauri-video-cut/commit/2a2ec90e7af620f704276978ef641e834d4527a4)) ([@constDEFE](https://github.com/constDEFE))
- Open devtools automatically when using `bun run tauri dev` ([76255c5](https://github.com/constDEFE/tauri-video-cut/commit/76255c55caeac53a52ba9f351288b1d90a7808ac)) ([@constDEFE](https://github.com/constDEFE))
- Add scrollbar styling ([949527f](https://github.com/constDEFE/tauri-video-cut/commit/949527f7aa15c754a89da7984cd6c8bd4d7719e2)) ([@constDEFE](https://github.com/constDEFE))

### Changed

- Extract keyframes by frames, not packets - supposed to improve performance ([e51c3b6](https://github.com/constDEFE/tauri-video-cut/commit/e51c3b666495fcdfb06d3ceb0880d43dd3343fde)) ([@constDEFE](https://github.com/constDEFE))
- Simplify theme initialization ([949527f](https://github.com/constDEFE/tauri-video-cut/commit/949527f7aa15c754a89da7984cd6c8bd4d7719e2)) ([@constDEFE](https://github.com/constDEFE))
- Disable audio track select when no tracks are present ([e9abd65](https://github.com/constDEFE/tauri-video-cut/commit/e9abd65a46e4cdf96f7cd647e96c804fe1f5e3c4)) ([@constDEFE](https://github.com/constDEFE))
- Extract player-to-state synchronization logic into usePlayer hook in `features/editor/player` ([e9abd65](https://github.com/constDEFE/tauri-video-cut/commit/e9abd65a46e4cdf96f7cd647e96c804fe1f5e3c4)) ([@constDEFE](https://github.com/constDEFE))
- Break Complete page into using CompleteOutput widget ([f51eb2d](https://github.com/constDEFE/tauri-video-cut/commit/f51eb2d582df0f000d990f50cb6ba883af615017)) ([@constDEFE](https://github.com/constDEFE))

### Fixed

- Fix extraction of audio track names when file is not .mp4, .mp4a, .mov ([e9abd65](https://github.com/constDEFE/tauri-video-cut/commit/e9abd65a46e4cdf96f7cd647e96c804fe1f5e3c4)) ([@constDEFE](https://github.com/constDEFE))
- Fix extraction of audio track names of .mp4, .mp4a, .mov files by manually parsing headers ([e9abd65](https://github.com/constDEFE/tauri-video-cut/commit/e9abd65a46e4cdf96f7cd647e96c804fe1f5e3c4)) ([@constDEFE](https://github.com/constDEFE))
- Fix preserving of audio track names during export by adding explicit mapping of audio tracks along with metadata preserving arguments to export cli ([e9abd65](https://github.com/constDEFE/tauri-video-cut/commit/e9abd65a46e4cdf96f7cd647e96c804fe1f5e3c4)) ([@constDEFE](https://github.com/constDEFE))
- Fix "F" hotkey taking precedence when focusing text fields ([6fd71eb](https://github.com/constDEFE/tauri-video-cut/commit/6fd71eb85ff46ac84843fc1e285ca45d9fa0c662)) ([@constDEFE](https://github.com/constDEFE))
- Remove unsupported file extension from advertising ([0264e80](https://github.com/constDEFE/tauri-video-cut/commit/0264e8039804c53062b230003c8df9f1054845c2)) ([@constDEFE](https://github.com/constDEFE))

## [1.0.0] - 2026-06-09

_Initial release._

[1.2.1]: https://github.com/constdefe/tauri-video-cut/releases/tag/1.2.1
[1.2.0]: https://github.com/constdefe/tauri-video-cut/releases/tag/1.2.0
[1.1.1]: https://github.com/constdefe/tauri-video-cut/releases/tag/1.1.1
[1.1.0]: https://github.com/constdefe/tauri-video-cut/releases/tag/1.1.0
[1.0.0]: https://github.com/constdefe/tauri-video-cut/releases/tag/1.0.0
