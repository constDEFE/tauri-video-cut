# Changelog

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

[1.1.1]: https://github.com/owner/name/releases/tag/v1.1.1
[1.1.0]: https://github.com/owner/name/releases/tag/v1.1.0
[1.0.0]: https://github.com/owner/name/releases/tag/v1.0.0
