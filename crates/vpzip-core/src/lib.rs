//! VPZip core — format readers (and selected writers) for Valve Source engine archives.
//!
//! Supported formats:
//!
//! | Extension | Description                                  | Read | Write |
//! |-----------|----------------------------------------------|:----:|:-----:|
//! | `.vpk`    | Valve Pak (v1 and v2, single and multi-part) |  Y   |   Y   |
//! | `.pak`    | Quake / GoldSrc / early Source pack          |  Y   |   Y   |
//! | `.gcf`    | Steam Game Cache File (read-only, v1/v3/v6)  |  Y   |   -   |
//! | `.sga`    | Relic / Iron Lore archive (v2/v4/v5/v7)      |  Y   |   -   |
//! | `.wad`    | id Software / GoldSrc texture WAD (WAD2/3)   |  Y   |   -   |
//! | `.xzp`    | Xbox HL2 Zip archive (xZip v1/v6)            |  Y   |   -   |
//!
//! All readers operate on a `Read + Seek` source. None of them require the full archive to be
//! loaded into memory.

pub mod archive;
pub mod error;
pub mod formats;
pub mod util;

pub use archive::{Archive, ArchiveEntry, EntryKind, OpenedArchive};
pub use error::{Error, Result};
pub use formats::Format;
