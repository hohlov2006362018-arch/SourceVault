# SourceVault

[🇬🇧 English](README.md) ·
**🇷🇺 Русский** ·
[🇺🇦 Українська](README.uk.md) ·
[🇩🇪 Deutsch](README.de.md) ·
[🇫🇷 Français](README.fr.md) ·
[🇪🇸 Español](README.es.md) ·
[🇨🇳 中文](README.zh.md) ·
[🇯🇵 日本語](README.ja.md)

> Бесплатный нативный архиватор на Rust для архивных форматов движка Valve Source —
> современный гибрид **WinRAR**, **7-Zip**, **Проводника Windows** и **GCFScape**, ограниченный
> только теми форматами, которые Valve выпускала со времён GoldSrc.

![ci](https://github.com/hohlov2006362018-arch/SourceVault/actions/workflows/ci.yml/badge.svg)
![release](https://img.shields.io/github/v/release/hohlov2006362018-arch/SourceVault?include_prereleases&sort=semver)
![license](https://img.shields.io/badge/license-MIT-blue)

---

## Поддерживаемые форматы

| Расширение | Описание                                                | Чтение | Запись |
|------------|---------------------------------------------------------|:------:|:------:|
| `.vpk`     | Valve Pak v1 / v2 (одиночный и многотомный `_dir.vpk`)  |   ✔    |   ✔    |
| `.pak`     | Quake / GoldSrc / ранние моды Source                    |   ✔    |   ✔    |
| `.gcf`     | Steam Game Cache File (до перехода на SteamPipe)        |   ✔    |   –    |
| `.sga`     | Relic SGA v4 / v5 (Dawn of War, Company of Heroes)      |   ✔    |   –    |
| `.wad`     | id Software / GoldSrc WAD текстур (WAD2 / WAD3)         |   ✔    |   –    |
| `.xzp`     | Xbox HL2 zip-архив (xZip v1 / v6)                       |   ✔    |   –    |

Все читатели работают стримом — ни один не загружает архив целиком в память.

## Особенности

- **Независимая.** Без рантайма, без Java и .NET. Один статически слинкованный .exe ~5 МиБ.
  Открыл — работаешь.
- **Все архитектуры.** Готовые сборки под `x86_64`, `i686` (32-битная) и `aarch64` Windows.
- **Интеграция с Проводником.** Инсталлятор регистрирует ассоциации для `.vpk / .pak / .gcf /
  .sga / .wad / .xzp` и добавляет пункт *Открыть с помощью SourceVault* в контекстное меню.
- **Многоязычный интерфейс** — English, Русский, Українська, Deutsch, Français, Español, 中文,
  日本語. Переключение из меню «Язык» без перезапуска; новые переводы — это просто `.json`-файлы.
- **Многоязычный репозиторий** — README, CHANGELOG и CONTRIBUTING переведены.
- **GUI + CLI.** `sourcevault-gui.exe` — окно в стиле WinRAR; `sourcevault.exe` — скриптуемый
  спутник для командной строки.
- **Современный Rust.** `#[forbid(unsafe_code)]` в крейте-парсере, без UPX, без обфускации,
  воспроизводимые сборки `cargo` + GitHub Actions.

## Установка

### Инсталлятор (рекомендуется)

Скачайте `SourceVault-x.y.z-setup.exe` из [последнего релиза](https://github.com/hohlov2006362018-arch/SourceVault/releases/latest).

Инсталлятор:

- Ставит `SourceVault.exe` и `sourcevault.exe` в `%ProgramFiles%\SourceVault`.
- Регистрирует ассоциации для всех поддерживаемых расширений.
- Добавляет пункт *Открыть с помощью SourceVault* в правый клик.
- Создаёт ярлыки в меню «Пуск».
- Имеет чистый деинсталлятор в *Приложения и возможности*.

### Портативная сборка

Скачайте `SourceVault-x.y.z-portable-<arch>.zip`, распакуйте куда угодно, запустите
`SourceVault.exe`. Реестр не трогается — идеально для USB-флешек и песочниц.

## Примеры командной строки

```pwsh
# Информация об архиве
sourcevault info путь\к\pak01_dir.vpk

# Список файлов с размером и CRC32
sourcevault list -l путь\к\pak01_dir.vpk

# Извлечь всё в папку
sourcevault extract -o C:\extracted путь\к\pak01_dir.vpk

# Извлечь только `materials/dev/`
sourcevault extract -p materials/dev/ -o C:\extracted путь\к\pak01_dir.vpk

# Собрать каталог в новый VPK v1
sourcevault pack -o my_mod.vpk .\my_mod
```

## Сборка из исходников

Нужен **Rust 1.74+** (`rustup default stable`), на Windows — MSVC toolchain.

```pwsh
cargo build --release
.\target\release\sourcevault.exe formats
.\target\release\sourcevault-gui.exe
```

Запуск всех тестов:

```pwsh
cargo test --workspace
```

## Поддержка ОС

| Целевая ОС                       | GUI | CLI | Проверено |
|----------------------------------|:---:|:---:|:---------:|
| Windows 11 / 10 / 8.1 / 8 (x64)  | ✔   | ✔   |     ✔     |
| Windows 7 SP1 (x64)              | ✔   | ✔   |     ✔     |
| Windows Vista SP2 / 7 (x86)      | ✔   | ✔   |     △¹    |
| Windows XP SP3 (x86)             | –²  | ✔   |     △¹    |
| Windows 11 ARM64                 | ✔   | ✔   |     ✔     |

¹ Best-effort. Официальный stable-toolchain Rust поддерживает Windows 7+. CLI-бинарник работает
на XP/Vista через устаревший MSVC-линкер, а GPU-ускоренная GUI-среда (`eframe`/`glow`) требует
Windows 7+ с OpenGL 3.0.

² На XP пользуйтесь CLI. GUI требует OpenGL 3.0, которого штатные драйверы XP обычно не дают.

## VirusTotal и подпись кода

Релизные бинарники — это статически слинкованные PE-файлы без упаковщиков и обфускации, собранные
официальными целями `x86_64-pc-windows-msvc`, `i686-pc-windows-msvc` и `aarch64-pc-windows-msvc`
в GitHub Actions. Workflow релиза публикует ссылки на отчёты VirusTotal рядом с SHA-256-суммами
на странице *Releases*.

Ложные срабатывания на свежих неподписанных Rust-бинарниках всё равно случаются. Если поймали
такое — откройте issue с точным движком и сигнатурой, отправим вендору запрос на whitelisting.

## Лицензия

MIT — см. [LICENSE](LICENSE). Проект SourceVault не связан с Valve, Microsoft, id Software,
Relic Entertainment и Sega.
