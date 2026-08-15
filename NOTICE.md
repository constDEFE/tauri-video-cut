# Legal Notices

## License

The source code of tauri-video-cut is licensed under the **Apache License 2.0**.
See [`LICENSE`](./LICENSE).

Distributed binary releases bundle GPL-licensed components and are provided
under the **GNU General Public License v2.0**.
See [`LICENSE.DISTRIBUTION`](./LICENSE.DISTRIBUTION) and [`SOURCE-OFFER.md`](./SOURCE-OFFER.md).

## Third-Party Software

This application bundles FFmpeg, mpv/libmpv, and various codec/rendering
libraries. Full attribution and license texts are provided in:

- [`legal/generated/THIRD_PARTY_LICENSES.md`](./legal/generated/THIRD_PARTY_LICENSES.md)
- the [`LICENSES/`](./LICENSES/) directory

### GPL Components (Upstream Source)

- **FFmpeg**: [n9.0.1](https://github.com/FFmpeg/FFmpeg/tree/n9.0.1) (Built with `--enable-gpl`)
- **mpv**: [v0.41.0](https://github.com/mpv-player/mpv/tree/v0.41.0)
- **libmpv-wrapper**: [v0.1.1](https://github.com/nini22P/libmpv-wrapper/tree/v0.1.1)

## Hardware Note

Bundled FFmpeg binaries are compiled with `--cpu=x86-64-v3` and require an
AVX2-capable CPU. They will not run on older processors.
