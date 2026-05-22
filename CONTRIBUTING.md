# Contributing to VPZip

Thanks for your interest in VPZip! This document is intentionally short — pull requests of
all sizes are welcome.

## Quick start

```pwsh
git clone https://github.com/hohlov2006362018-arch/SourceVault.git
cd VPZip
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

The CI runs the same four commands; if all of them pass locally, your PR will pass CI.

## Adding a new translation

UI translations live in `crates/vpzip-gui/i18n/<lang>.json`. To add `it.json` (Italian):

1. Copy `crates/vpzip-gui/i18n/en.json` to `crates/vpzip-gui/i18n/it.json` and
   translate the values.
2. Register the language in `crates/vpzip-gui/src/i18n.rs` (`Lang::IT` constant, the `ALL`
   array, and the `label` function).
3. Translate `README.md` to `README.it.md` and update the language switcher row at the top of
   every README.
4. Run `cargo build` and `cargo test`. Open a PR.

The English file (`en.json`) is the canonical key set; missing keys in other languages fall back
to English at runtime.

## Adding format support

Any new format must be implemented as a separate module under
`crates/vpzip-core/src/formats/<fmt>.rs`, expose a struct that implements the `Archive`
trait, and be registered in `formats/mod.rs` (`Format::detect`, `Format::open_as`).

Please include at least one round-trip or sample-file test — see `vpk::tests` and
`pak::tests` for the conventions.

## Reporting security issues

Open a private security advisory via GitHub rather than a public issue.

## Code of conduct

Be kind to each other. We follow the [Contributor Covenant 2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/).
