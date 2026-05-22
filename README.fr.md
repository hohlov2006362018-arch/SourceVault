# VPZip

[🇬🇧 English](README.md) ·
[🇷🇺 Русский](README.ru.md) ·
[🇺🇦 Українська](README.uk.md) ·
[🇩🇪 Deutsch](README.de.md) ·
**🇫🇷 Français** ·
[🇪🇸 Español](README.es.md) ·
[🇨🇳 中文](README.zh.md) ·
[🇯🇵 日本語](README.ja.md)

> Archiveur natif gratuit en Rust pour les formats d'archive du moteur Valve Source — une fusion
> moderne de **WinRAR**, **7-Zip**, **l'Explorateur Windows** et **GCFScape**, limitée aux
> formats que Valve a publiés depuis GoldSrc.

## Formats pris en charge

| Extension | Description                                                  | Lecture | Écriture |
|-----------|--------------------------------------------------------------|:-------:|:--------:|
| `.vpk`    | Valve Pak v1 / v2 (mono-fichier et multi-fichier `_dir.vpk`) |    ✔    |    ✔     |
| `.pak`    | Quake / GoldSrc / premiers mods Source                       |    ✔    |    ✔     |
| `.gcf`    | Steam Game Cache File (héritage, avant SteamPipe)            |    ✔    |    –     |
| `.sga`    | Relic SGA v4 / v5 (Dawn of War, Company of Heroes)           |    ✔    |    –     |
| `.wad`    | id Software / GoldSrc WAD de textures (WAD2 / WAD3)          |    ✔    |    –     |
| `.xzp`    | Archive zip Xbox HL2 (xZip v1 / v6)                          |    ✔    |    –     |

## Points forts

- **Indépendant.** Pas de runtime, pas de Java, pas de .NET — un seul exécutable Windows lié
  statiquement (~5 MiB).
- **Toutes les architectures.** Versions précompilées pour `x86_64`, `i686` (32 bits) et `aarch64`.
- **Intégration shell complète.** L'installeur enregistre les associations de fichiers pour
  chaque extension prise en charge et ajoute *Ouvrir avec VPZip* au menu contextuel.
- **Interface multilingue** — English, Русский, Українська, Deutsch, Français, Español, 中文,
  日本語. Changement instantané via le menu *Langue*.
- **Dépôt multilingue** — README, CHANGELOG et CONTRIBUTING traduits.
- **GUI + CLI.** Fenêtre style WinRAR et compagnon de ligne de commande scriptable.

## Installation

Téléchargez `VPZip-x.y.z-setup.exe` depuis la [dernière version](https://github.com/hohlov2006362018-arch/SourceVault/releases/latest).
L'installeur configure les associations, ajoute l'entrée du menu contextuel et un désinstalleur
propre dans *Applications et fonctionnalités*.

Version portable : `VPZip-x.y.z-portable-<arch>.zip`, à extraire et lancer.

## Ligne de commande

```pwsh
vpzip info chemin\vers\pak01_dir.vpk
vpzip list -l chemin\vers\pak01_dir.vpk
vpzip extract -o C:\extracted chemin\vers\pak01_dir.vpk
vpzip pack -o mod.vpk .\mod
```

## Construction depuis les sources

Requiert **Rust 1.74+** (`rustup default stable`), sur Windows la chaîne MSVC.

```pwsh
cargo build --release
cargo test --workspace
```

## Licence

MIT — voir [LICENSE](LICENSE). Sans affiliation avec Valve, Microsoft, id Software,
Relic Entertainment ou Sega.
