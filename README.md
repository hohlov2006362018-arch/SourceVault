<!--
  When you change this file, please also update the translated copies:
    README.ru.md  README.uk.md  README.de.md  README.fr.md
    README.es.md  README.zh.md  README.ja.md
-->

# SourceVault

**🇬🇧 English** ·
[🇷🇺 Русский](README.ru.md) ·
[🇺🇦 Українська](README.uk.md) ·
[🇩🇪 Deutsch](README.de.md) ·
[🇫🇷 Français](README.fr.md) ·
[🇪🇸 Español](README.es.md) ·
[🇨🇳 中文](README.zh.md) ·
[🇯🇵 日本語](README.ja.md)

> Free, native Rust archiver for Valve Source engine archive formats — a modern fusion of
> **WinRAR**, **7-Zip**, **Windows Explorer** and **GCFScape**, scoped exclusively to the formats
> Valve has shipped since GoldSrc.

![ci](https://github.com/hohlov2006362018-arch/SourceVault/actions/workflows/ci.yml/badge.svg)
![release](https://img.shields.io/github/v/release/hohlov2006362018-arch/SourceVault?include_prereleases&sort=semver)
![license](https://img.shields.io/badge/license-MIT-blue)

---

## Supported formats

| Extension | Description                                              | Read | Write |
|-----------|----------------------------------------------------------|:----:|:-----:|
| `.vpk`    | Valve Pak v1 / v2 (single-file and multi-part `_dir.vpk`) |  ✔  |   ✔   |
| `.pak`    | Quake / GoldSrc / early Source pack                       |  ✔  |   ✔   |
| `.gcf`    | Steam Game Cache File (legacy, pre-SteamPipe)             |  ✔  |   –   |
| `.sga`    | Relic SGA v4 / v5 (Dawn of War, Company of Heroes)         |  ✔  |   –   |
| `.wad`    | id Software / GoldSrc texture WAD (WAD2 / WAD3)           |  ✔  |   –   |
| `.xzp`    | Xbox HL2 zip archive (xZip v1 / v6)                       |  ✔  |   –   |

Every reader streams from disk — none of them load the full archive into memory.

## Highlights

- **Independent.** No runtime, no Java, no .NET. A single ~5 MiB statically-linked Windows
  executable. Open and use.
- **Multi-architecture.** Pre-built binaries for `x86_64`, `i686` (32-bit) and `aarch64` Windows.
- **Full Windows shell integration.** The installer registers `.vpk / .pak / .gcf / .sga / .wad /
  .xzp` and adds a *Open with SourceVault* context-menu entry.
- **Multilingual UI** — English, Русский, Українська, Deutsch, Français, Español, 中文, 日本語.
  Switch on-the-fly from the *Language* menu; new translations are drop-in `.json` files.
- **Multilingual repository** — every README, CHANGELOG and CONTRIBUTING is mirrored.
- **GUI + CLI.** `sourcevault-gui.exe` is the WinRAR-style window; `sourcevault.exe` is a
  scriptable command-line companion.
- **Modern Rust.** `#[forbid(unsafe_code)]` in the parser crate, no UPX, no obfuscation, fully
  reproducible builds via `cargo` + GitHub Actions.

## Installation

### Installer (recommended)

Download `SourceVault-x.y.z-setup.exe` from the [latest release](https://github.com/hohlov2006362018-arch/SourceVault/releases/latest).

The installer:

- Installs `SourceVault.exe` and `sourcevault.exe` under `%ProgramFiles%\SourceVault`.
- Registers file associations for every supported extension.
- Adds *Open with SourceVault* to the right-click *Open With* list.
- Creates Start menu shortcuts.
- Provides a clean uninstaller in *Apps & Features*.

### Portable

Download `SourceVault-x.y.z-portable-<arch>.zip`, extract anywhere, run `SourceVault.exe`.
No registry changes are made — perfect for USB sticks and sandboxed environments.

## Command-line examples

```pwsh
# Inspect an archive
sourcevault info path\to\pak01_dir.vpk

# List entries (with sizes and CRC32)
sourcevault list -l path\to\pak01_dir.vpk

# Extract everything to a folder
sourcevault extract -o C:\extracted path\to\pak01_dir.vpk

# Extract only `materials/dev/` to a folder
sourcevault extract -p materials/dev/ -o C:\extracted path\to\pak01_dir.vpk

# Pack a directory back into a fresh VPK v1
sourcevault pack -o my_mod.vpk .\my_mod
```

## Building from source

Requires **Rust 1.74+** (`rustup default stable`) and, on Windows, the MSVC toolchain.

```pwsh
cargo build --release
.\target\release\sourcevault.exe formats
.\target\release\sourcevault-gui.exe
```

Run all tests:

```pwsh
cargo test --workspace
```

## Platform support

| Target                          | GUI | CLI | Tested |
|---------------------------------|:---:|:---:|:------:|
| Windows 11 / 10 / 8.1 / 8 (x64) | ✔  | ✔  |   ✔   |
| Windows 7 SP1 (x64)             | ✔  | ✔  |   ✔   |
| Windows Vista SP2 / 7 (x86)     | ✔  | ✔  |   △¹   |
| Windows XP SP3 (x86)            | –² | ✔  |   △¹   |
| Windows 11 ARM64                | ✔  | ✔  |   ✔   |

¹ Best-effort. The Rust stable toolchain officially supports Windows 7+. XP/Vista work for the CLI
binary via the legacy MSVC linker, but the GPU-accelerated GUI runtime (`eframe`/`glow`) requires
Windows 7+ with OpenGL 3.0.
² On XP, prefer the CLI. The GUI requires OpenGL 3.0, which XP's drivers generally do not provide.

## VirusTotal & code signing

Releases are unpacked, non-obfuscated, statically-linked PE files built with the official
`x86_64-pc-windows-msvc`, `i686-pc-windows-msvc` and `aarch64-pc-windows-msvc` Rust targets in
GitHub Actions. The release workflow records the VirusTotal report URL for every artifact next to
the SHA-256 checksums on the *Releases* page.

False positives still happen on brand-new unsigned Rust binaries. If you see one, please open an
issue with the exact engine + signature so we can submit a clean-file report to the vendor.

## License

MIT — see [LICENSE](LICENSE). SourceVault is unaffiliated with Valve, Microsoft, id Software,
Relic Entertainment or Sega.
