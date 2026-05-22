# VPZip

[🇬🇧 English](README.md) ·
[🇷🇺 Русский](README.ru.md) ·
[🇺🇦 Українська](README.uk.md) ·
[🇩🇪 Deutsch](README.de.md) ·
[🇫🇷 Français](README.fr.md) ·
[🇪🇸 Español](README.es.md) ·
**🇨🇳 中文** ·
[🇯🇵 日本語](README.ja.md)

> 用 Rust 原生编写、免费开源、面向 Valve Source 引擎归档格式的归档工具 ——
> **WinRAR**、**7-Zip**、**Windows 资源管理器**和 **GCFScape** 的现代融合,
> 仅针对 Valve 自 GoldSrc 以来发布的格式。

## 支持的格式

| 扩展名 | 说明                                                          | 读取 | 写入 |
|--------|---------------------------------------------------------------|:----:|:----:|
| `.vpk` | Valve Pak v1 / v2(单文件与多分卷 `_dir.vpk`)                  |  ✔   |  ✔   |
| `.pak` | Quake / GoldSrc / 早期 Source mod                             |  ✔   |  ✔   |
| `.gcf` | Steam Game Cache File(SteamPipe 之前的旧格式)                 |  ✔   |  –   |
| `.sga` | Relic SGA v4 / v5(Dawn of War、Company of Heroes)             |  ✔   |  –   |
| `.wad` | id Software / GoldSrc 纹理 WAD(WAD2 / WAD3)                  |  ✔   |  –   |
| `.xzp` | Xbox HL2 zip 归档(xZip v1 / v6)                              |  ✔   |  –   |

## 亮点

- **独立运行。** 无 Java、无 .NET、无运行时——单个静态链接的 Windows 可执行文件(~5 MiB)。
- **多架构。** 提供 `x86_64`、`i686`(32 位)、`aarch64` Windows 预编译版本。
- **完整 Shell 集成。** 安装器为每个受支持的扩展名注册文件关联,并在右键菜单中加入
  *用 VPZip 打开*。
- **多语言界面** —— English、Русский、Українська、Deutsch、Français、Español、中文、日本語。
  通过*语言*菜单即时切换。
- **多语言仓库** —— README、CHANGELOG 与 CONTRIBUTING 全部翻译。
- **GUI + CLI。** WinRAR 风格的窗口,以及可编写脚本的命令行工具。

## 安装

从[最新版本](https://github.com/hohlov2006362018-arch/SourceVault/releases/latest)下载
`VPZip-x.y.z-setup.exe`。

便携版:`VPZip-x.y.z-portable-<arch>.zip`,解压即可使用。

## 命令行示例

```pwsh
vpzip info 路径\到\pak01_dir.vpk
vpzip list -l 路径\到\pak01_dir.vpk
vpzip extract -o C:\extracted 路径\到\pak01_dir.vpk
vpzip pack -o mod.vpk .\mod
```

## 从源代码构建

需要 **Rust 1.74+**(`rustup default stable`);Windows 上需要 MSVC 工具链。

```pwsh
cargo build --release
cargo test --workspace
```

## 许可证

MIT —— 见 [LICENSE](LICENSE)。与 Valve、Microsoft、id Software、Relic Entertainment 或 Sega 无任何隶属关系。
