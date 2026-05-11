# SourceVault

[🇬🇧 English](README.md) ·
[🇷🇺 Русский](README.ru.md) ·
**🇺🇦 Українська** ·
[🇩🇪 Deutsch](README.de.md) ·
[🇫🇷 Français](README.fr.md) ·
[🇪🇸 Español](README.es.md) ·
[🇨🇳 中文](README.zh.md) ·
[🇯🇵 日本語](README.ja.md)

> Безкоштовний нативний архіватор на Rust для архівних форматів рушія Valve Source —
> сучасний гібрид **WinRAR**, **7-Zip**, **Провідника Windows** і **GCFScape**, обмежений лише
> тими форматами, які Valve випустила від GoldSrc.

![ci](https://github.com/hohlov2006362018-arch/SourceVault/actions/workflows/ci.yml/badge.svg)
![release](https://img.shields.io/github/v/release/hohlov2006362018-arch/SourceVault?include_prereleases&sort=semver)
![license](https://img.shields.io/badge/license-MIT-blue)

## Підтримувані формати

| Розширення | Опис                                                      | Читання | Запис |
|------------|-----------------------------------------------------------|:-------:|:-----:|
| `.vpk`     | Valve Pak v1 / v2 (одинарний і багатотомний `_dir.vpk`)   |    ✔    |   ✔   |
| `.pak`     | Quake / GoldSrc / ранні моди Source                       |    ✔    |   ✔   |
| `.gcf`     | Steam Game Cache File (до SteamPipe)                      |    ✔    |   –   |
| `.sga`     | Relic SGA v4 / v5 (Dawn of War, Company of Heroes)        |    ✔    |   –   |
| `.wad`     | id Software / GoldSrc WAD текстур (WAD2 / WAD3)           |    ✔    |   –   |
| `.xzp`     | Xbox HL2 zip-архів (xZip v1 / v6)                         |    ✔    |   –   |

## Особливості

- **Незалежний.** Без рантайму, без Java, без .NET. Один статично злінкований `.exe` ~5 МіБ.
- **Усі архітектури.** Готові збірки під `x86_64`, `i686` (32-бітна) та `aarch64` Windows.
- **Інтеграція з Провідником.** Інсталятор реєструє асоціації для `.vpk / .pak / .gcf / .sga /
  .wad / .xzp` і додає пункт *Відкрити за допомогою SourceVault* у контекстне меню.
- **Багатомовний інтерфейс** — English, Русский, Українська, Deutsch, Français, Español, 中文,
  日本語. Перемикання без перезапуску; нові переклади — це лише `.json`-файли.
- **Багатомовне сховище** — README, CHANGELOG та CONTRIBUTING перекладено.
- **GUI + CLI.** `sourcevault-gui.exe` — вікно у стилі WinRAR; `sourcevault.exe` — скриптовий
  командний рядок.

## Встановлення

Завантажте `SourceVault-x.y.z-setup.exe` з [останнього релізу](https://github.com/hohlov2006362018-arch/SourceVault/releases/latest).
Інсталятор реєструє асоціації файлів, додає пункт контекстного меню та створює деінсталятор.

Портативна версія — `SourceVault-x.y.z-portable-<arch>.zip` — не змінює реєстр.

## Приклади командного рядка

```pwsh
sourcevault info шлях\до\pak01_dir.vpk
sourcevault list -l шлях\до\pak01_dir.vpk
sourcevault extract -o C:\extracted шлях\до\pak01_dir.vpk
sourcevault pack -o my_mod.vpk .\my_mod
```

## Збірка з джерел

Потрібен **Rust 1.74+** (`rustup default stable`), на Windows — MSVC toolchain.

```pwsh
cargo build --release
cargo test --workspace
```

## Підтримка ОС

Від Windows 11 / 10 / 8.1 / 8 / 7 (x64 і ARM64) — повна підтримка GUI та CLI. Для Vista/XP — лише
CLI (стабільний Rust офіційно підтримує Windows 7+).

## Ліцензія

MIT — див. [LICENSE](LICENSE). Проєкт не пов’язаний із Valve, Microsoft, id Software,
Relic Entertainment чи Sega.
