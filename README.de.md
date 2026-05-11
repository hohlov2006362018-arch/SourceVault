# SourceVault

[🇬🇧 English](README.md) ·
[🇷🇺 Русский](README.ru.md) ·
[🇺🇦 Українська](README.uk.md) ·
**🇩🇪 Deutsch** ·
[🇫🇷 Français](README.fr.md) ·
[🇪🇸 Español](README.es.md) ·
[🇨🇳 中文](README.zh.md) ·
[🇯🇵 日本語](README.ja.md)

> Kostenloser, nativer Rust-Archivierer für die Archivformate der Valve-Source-Engine —
> eine moderne Mischung aus **WinRAR**, **7-Zip**, **Windows-Explorer** und **GCFScape**,
> ausschließlich für die Formate, die Valve seit GoldSrc ausgeliefert hat.

## Unterstützte Formate

| Endung  | Beschreibung                                          | Lesen | Schreiben |
|---------|-------------------------------------------------------|:-----:|:---------:|
| `.vpk`  | Valve Pak v1 / v2 (einzeln und mehrteilig `_dir.vpk`) |   ✔   |     ✔     |
| `.pak`  | Quake / GoldSrc / frühe Source-Mods                   |   ✔   |     ✔     |
| `.gcf`  | Steam Game Cache File (Legacy, vor SteamPipe)         |   ✔   |     –     |
| `.sga`  | Relic SGA v4 / v5                                     |   ✔   |     –     |
| `.wad`  | id Software / GoldSrc-Textur-WAD (WAD2 / WAD3)        |   ✔   |     –     |
| `.xzp`  | Xbox-HL2-Zip-Archiv (xZip v1 / v6)                    |   ✔   |     –     |

## Eigenschaften

- **Unabhängig.** Keine Runtime, kein Java, kein .NET — eine einzelne, statisch gelinkte
  Windows-EXE (~5 MiB).
- **Alle Architekturen.** Vorgefertigte Builds für `x86_64`, `i686` (32-Bit) und `aarch64`.
- **Volle Shell-Integration.** Der Installer registriert Dateizuordnungen für jede unterstützte
  Endung und fügt *Mit SourceVault öffnen* zum Rechtsklick-Menü hinzu.
- **Mehrsprachige Oberfläche** — English, Русский, Українська, Deutsch, Français, Español, 中文,
  日本語. Sofortiger Wechsel über das Menü *Sprache*.
- **Mehrsprachiges Repository** — README, CHANGELOG und CONTRIBUTING sind übersetzt.
- **GUI + CLI.** WinRAR-artiges Fenster und ein skriptbares Kommandozeilen-Werkzeug.

## Installation

Lade `SourceVault-x.y.z-setup.exe` aus dem [neuesten Release](https://github.com/hohlov2006362018-arch/SourceVault/releases/latest).
Der Installer registriert Dateizuordnungen, fügt den Kontextmenüeintrag hinzu und liefert einen
sauberen Deinstallateur in *Apps & Features*.

Portable Variante: `SourceVault-x.y.z-portable-<arch>.zip`, entpacken, `SourceVault.exe` starten.

## Kommandozeile

```pwsh
sourcevault info Pfad\zu\pak01_dir.vpk
sourcevault list -l Pfad\zu\pak01_dir.vpk
sourcevault extract -o C:\extracted Pfad\zu\pak01_dir.vpk
sourcevault pack -o mod.vpk .\mod
```

## Aus Quellcode bauen

Benötigt **Rust 1.74+** (`rustup default stable`), unter Windows die MSVC-Toolchain.

```pwsh
cargo build --release
cargo test --workspace
```

## Lizenz

MIT — siehe [LICENSE](LICENSE). Nicht mit Valve, Microsoft, id Software, Relic Entertainment
oder Sega verbunden.
