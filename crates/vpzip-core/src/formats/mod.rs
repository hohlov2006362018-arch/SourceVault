//! Format detection and dispatch.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use crate::archive::OpenedArchive;
use crate::error::{Error, Result};

pub mod gcf;
pub mod pak;
pub mod sga;
pub mod vpk;
pub mod wad;
pub mod xzp;

/// Identifier for a supported archive format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Format {
    Vpk,
    Pak,
    Wad,
    Xzp,
    Gcf,
    Sga,
}

impl Format {
    pub fn name(self) -> &'static str {
        match self {
            Self::Vpk => "VPK",
            Self::Pak => "PAK",
            Self::Wad => "WAD",
            Self::Xzp => "XZP",
            Self::Gcf => "GCF",
            Self::Sga => "SGA",
        }
    }

    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Self::Vpk => &["vpk"],
            Self::Pak => &["pak"],
            Self::Wad => &["wad"],
            Self::Xzp => &["xzp"],
            Self::Gcf => &["gcf", "ncf"],
            Self::Sga => &["sga"],
        }
    }

    pub fn all() -> &'static [Format] {
        &[
            Self::Vpk,
            Self::Pak,
            Self::Wad,
            Self::Xzp,
            Self::Gcf,
            Self::Sga,
        ]
    }
}

/// Sniffs the first bytes of a file to figure out which format it is.
pub fn detect<R: Read + Seek>(reader: &mut R) -> Result<Format> {
    let mut sig = [0u8; 16];
    reader.seek(SeekFrom::Start(0))?;
    let read = reader.read(&mut sig)?;
    reader.seek(SeekFrom::Start(0))?;

    if read < 4 {
        return Err(Error::UnknownFormat);
    }

    // VPK: 0x55AA1234 little-endian
    if sig[..4] == [0x34, 0x12, 0xAA, 0x55] {
        return Ok(Format::Vpk);
    }
    // PAK: "PACK"
    if &sig[..4] == b"PACK" {
        return Ok(Format::Pak);
    }
    // WAD: "WAD2" or "WAD3"
    if &sig[..4] == b"WAD2" || &sig[..4] == b"WAD3" {
        return Ok(Format::Wad);
    }
    // XZP: "piZx" (0x70695A78 / "xZip" in little-endian char order)
    if &sig[..4] == b"piZx" {
        return Ok(Format::Xzp);
    }
    // GCF: starts with HeaderVersion (1 or 2), CacheType (1 = GCF, 2 = NCF),
    // FormatVersion (1, 3 or 6). The reliable marker is the 4-byte HeaderVersion=1
    // followed by CacheType=1 or 2.
    if read >= 8
        && sig[0] == 0x01
        && sig[1] == 0x00
        && sig[2] == 0x00
        && sig[3] == 0x00
        && (sig[4] == 0x01 || sig[4] == 0x02)
    {
        return Ok(Format::Gcf);
    }
    // SGA: "_ARCHIVE"
    if read >= 8 && &sig[..8] == b"_ARCHIVE" {
        return Ok(Format::Sga);
    }

    Err(Error::UnknownFormat)
}

/// Opens an archive from a filesystem path, auto-detecting its format.
pub fn open(path: &Path) -> Result<OpenedArchive> {
    let mut file = File::open(path)?;
    let fmt = detect(&mut file)?;
    open_as(path, fmt)
}

/// Opens an archive from a filesystem path with an explicit format.
pub fn open_as(path: &Path, fmt: Format) -> Result<OpenedArchive> {
    Ok(match fmt {
        Format::Vpk => OpenedArchive::Vpk(vpk::VpkArchive::open(path)?),
        Format::Pak => OpenedArchive::Pak(pak::PakArchive::open(path)?),
        Format::Wad => OpenedArchive::Wad(wad::WadArchive::open(path)?),
        Format::Xzp => OpenedArchive::Xzp(xzp::XzpArchive::open(path)?),
        Format::Gcf => OpenedArchive::Gcf(gcf::GcfArchive::open(path)?),
        Format::Sga => OpenedArchive::Sga(sga::SgaArchive::open(path)?),
    })
}
