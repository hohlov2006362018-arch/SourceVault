# Changelog

All notable changes to SourceVault are documented in this file. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/hohlov2006362018-arch/SourceVault/compare/HEAD...HEAD
