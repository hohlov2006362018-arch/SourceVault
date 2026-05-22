# VPZip

[🇬🇧 English](README.md) ·
[🇷🇺 Русский](README.ru.md) ·
[🇺🇦 Українська](README.uk.md) ·
[🇩🇪 Deutsch](README.de.md) ·
[🇫🇷 Français](README.fr.md) ·
[🇪🇸 Español](README.es.md) ·
[🇨🇳 中文](README.zh.md) ·
**🇯🇵 日本語**

> Valve Source エンジンのアーカイブ形式専用、Rust 製の無料ネイティブ アーカイバ ——
> **WinRAR**、**7-Zip**、**Windows エクスプローラー**、**GCFScape** の現代的なハイブリッドで、
> GoldSrc 以降に Valve が出した形式に絞っています。

## 対応形式

| 拡張子 | 説明                                                            | 読込 | 書込 |
|--------|-----------------------------------------------------------------|:----:|:----:|
| `.vpk` | Valve Pak v1 / v2(単一ファイル・複数分割 `_dir.vpk`)            |  ✔   |  ✔   |
| `.pak` | Quake / GoldSrc / 初期 Source mod                               |  ✔   |  ✔   |
| `.gcf` | Steam Game Cache File(SteamPipe 以前のレガシー形式)            |  ✔   |  –   |
| `.sga` | Relic SGA v4 / v5(Dawn of War、Company of Heroes)              |  ✔   |  –   |
| `.wad` | id Software / GoldSrc テクスチャ WAD(WAD2 / WAD3)              |  ✔   |  –   |
| `.xzp` | Xbox HL2 zip アーカイブ(xZip v1 / v6)                          |  ✔   |  –   |

## 特長

- **独立動作。** Java/.NET 不要、ランタイム不要 —— 単一の静的リンク済み Windows EXE(~5 MiB)。
- **全アーキテクチャ対応。** `x86_64`、`i686`(32-bit)、`aarch64` Windows 用ビルドを提供。
- **シェル統合。** インストーラーが各拡張子に対するファイル関連付けを登録し、
  右クリックメニューに *VPZip で開く* を追加します。
- **多言語 UI** —— English、Русский、Українська、Deutsch、Français、Español、中文、日本語。
  *言語* メニューから即時切替。
- **多言語リポジトリ** —— README、CHANGELOG、CONTRIBUTING を全て翻訳済み。
- **GUI + CLI。** WinRAR 風の GUI と、スクリプト可能なコマンドライン版を同梱。

## インストール

[最新リリース](https://github.com/hohlov2006362018-arch/SourceVault/releases/latest)から
`VPZip-x.y.z-setup.exe` をダウンロードしてください。

ポータブル版:`VPZip-x.y.z-portable-<arch>.zip` を展開して `VPZip.exe` を起動。

## コマンドライン例

```pwsh
vpzip info パス\to\pak01_dir.vpk
vpzip list -l パス\to\pak01_dir.vpk
vpzip extract -o C:\extracted パス\to\pak01_dir.vpk
vpzip pack -o mod.vpk .\mod
```

## ソースからビルド

**Rust 1.74+**(`rustup default stable`)が必要。Windows では MSVC ツールチェインも必要。

```pwsh
cargo build --release
cargo test --workspace
```

## ライセンス

MIT —— [LICENSE](LICENSE) を参照。Valve、Microsoft、id Software、Relic Entertainment、
Sega とは関係ありません。
