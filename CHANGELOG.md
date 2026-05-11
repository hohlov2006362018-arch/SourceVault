# Changelog

All notable changes to SourceVault are documented in this file. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] — 2026-05-11

### Changed

- Release profile tuned to reduce machine-learning antivirus false positives:
  - `lto = false`, `codegen-units = 16`, `strip = "none"`, `panic = "unwind"` (default Rust).
  - All exe sections now look like normal `cargo build --release` output rather than a
    densely packed, symbol-stripped artifact that ML models tend to flag.
- Both binaries now ship with a complete Windows `VERSIONINFO` block
  (`FileDescription`, `ProductName`, `OriginalFilename`, `InternalName`, `CompanyName`,
  `LegalCopyright`, `FileVersion`, `ProductVersion`) and the application icon, so the
  metadata heuristics that look for empty-fields binaries are satisfied.

### Added

- `crates/sourcevault-cli/build.rs`: embed VERSIONINFO + icon into `sourcevault.exe`.

## [0.1.0] — 2026-05-11

### Added

- Initial scaffolding of the Rust workspace (`sourcevault-core`, `sourcevault-cli`,
  `sourcevault-gui`).
- Read support for `.vpk` (v1 + v2, multi-part), `.pak`, `.wad` (WAD2/WAD3), `.xzp` (v1/v6),
  `.gcf` (v1/v3/v5/v6) and `.sga` (v4/v5).
- Write support for `.vpk` v1 single-file and `.pak`.
- WinRAR-style GUI with tree view, entry list, preview pane and drag-and-drop.
- Multilingual UI (8 languages out of the box).
- Multilingual repository README (8 languages).
- `sourcevault` CLI binary with `list`, `info`, `extract`, `pack` and `formats` subcommands.
- Inno Setup installer with shell integration (file associations, *Open with* context-menu
  entry) and a clean uninstaller.
- GitHub Actions pipelines for clippy, formatting, multi-arch builds and release packaging.

[Unreleased]: https://github.com/hohlov2006362018-arch/SourceVault/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/hohlov2006362018-arch/SourceVault/releases/tag/v0.1.1
[0.1.0]: https://github.com/hohlov2006362018-arch/SourceVault/releases/tag/v0.1.0
