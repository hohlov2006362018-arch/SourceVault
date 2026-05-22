# VPZip

[🇬🇧 English](README.md) ·
[🇷🇺 Русский](README.ru.md) ·
[🇺🇦 Українська](README.uk.md) ·
[🇩🇪 Deutsch](README.de.md) ·
[🇫🇷 Français](README.fr.md) ·
**🇪🇸 Español** ·
[🇨🇳 中文](README.zh.md) ·
[🇯🇵 日本語](README.ja.md)

> Archivador nativo gratuito en Rust para los formatos de archivo del motor Valve Source —
> una fusión moderna de **WinRAR**, **7-Zip**, **Explorador de Windows** y **GCFScape**, limitada
> a los formatos que Valve ha publicado desde GoldSrc.

## Formatos compatibles

| Extensión | Descripción                                            | Lectura | Escritura |
|-----------|--------------------------------------------------------|:-------:|:---------:|
| `.vpk`    | Valve Pak v1 / v2 (un solo fichero y `_dir.vpk`)       |    ✔    |     ✔     |
| `.pak`    | Quake / GoldSrc / primeros mods de Source              |    ✔    |     ✔     |
| `.gcf`    | Steam Game Cache File (heredado, antes de SteamPipe)   |    ✔    |     –     |
| `.sga`    | Relic SGA v4 / v5 (Dawn of War, Company of Heroes)     |    ✔    |     –     |
| `.wad`    | id Software / GoldSrc WAD de texturas (WAD2 / WAD3)    |    ✔    |     –     |
| `.xzp`    | Archivo zip de Xbox HL2 (xZip v1 / v6)                 |    ✔    |     –     |

## Características

- **Independiente.** Sin runtime, sin Java, sin .NET — un único ejecutable de Windows enlazado
  estáticamente (~5 MiB).
- **Todas las arquitecturas.** Binarios precompilados para `x86_64`, `i686` (32 bits) y `aarch64`.
- **Integración completa con el Explorador.** El instalador registra asociaciones para todas las
  extensiones y añade *Abrir con VPZip* al menú contextual.
- **Interfaz multilingüe** — English, Русский, Українська, Deutsch, Français, Español, 中文,
  日本語. Cambio inmediato desde el menú *Idioma*.
- **Repositorio multilingüe** — README, CHANGELOG y CONTRIBUTING traducidos.
- **GUI + CLI.** Ventana estilo WinRAR y compañero de línea de comandos scriptable.

## Instalación

Descarga `VPZip-x.y.z-setup.exe` desde la [última versión](https://github.com/hohlov2006362018-arch/SourceVault/releases/latest).

Versión portable: `VPZip-x.y.z-portable-<arch>.zip`, extrae y ejecuta.

## Línea de comandos

```pwsh
vpzip info ruta\a\pak01_dir.vpk
vpzip list -l ruta\a\pak01_dir.vpk
vpzip extract -o C:\extracted ruta\a\pak01_dir.vpk
vpzip pack -o mod.vpk .\mod
```

## Compilación desde código fuente

Requiere **Rust 1.74+** (`rustup default stable`); en Windows, la cadena MSVC.

```pwsh
cargo build --release
cargo test --workspace
```

## Licencia

MIT — consulta [LICENSE](LICENSE). Sin afiliación con Valve, Microsoft, id Software,
Relic Entertainment o Sega.
