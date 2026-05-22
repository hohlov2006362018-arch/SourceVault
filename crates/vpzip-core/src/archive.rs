//! Core archive abstraction.
//!
//! Every supported format is exposed through the [`Archive`] trait. Users typically obtain an
//! [`OpenedArchive`] via [`crate::formats::open`], which dispatches to the correct format
//! implementation based on the file's signature.

use std::fs;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};

use crate::error::Result;

/// Type of entry inside an archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EntryKind {
    File,
    Directory,
}

/// Metadata for a single entry inside an archive.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArchiveEntry {
    /// Forward-slash normalised path inside the archive (e.g. `materials/dev/dev_wall_01.vtf`).
    pub path: String,
    /// Whether this is a file or a synthesized directory marker.
    pub kind: EntryKind,
    /// Uncompressed size in bytes (for files). For directories this is always 0.
    pub size: u64,
    /// Compressed/on-disk size in bytes — `None` if the format does not compress entries
    /// (e.g. VPK, PAK).
    pub compressed_size: Option<u64>,
    /// Optional CRC-32 reported by the archive (e.g. VPK stores one per entry).
    pub crc32: Option<u32>,
}

impl ArchiveEntry {
    pub fn file(path: impl Into<String>, size: u64) -> Self {
        Self {
            path: path.into(),
            kind: EntryKind::File,
            size,
            compressed_size: None,
            crc32: None,
        }
    }
}

/// Trait implemented by every format reader.
pub trait Archive {
    /// Human-friendly format name (e.g. `"VPK v2"`, `"WAD3"`, `"GCF"`).
    fn format_name(&self) -> &'static str;

    /// Returns a sorted listing of all entries in the archive.
    fn entries(&self) -> &[ArchiveEntry];

    /// Reads the full contents of a file entry into memory.
    fn read_entry(&mut self, path: &str) -> Result<Vec<u8>>;

    /// Extracts a single entry to disk.
    fn extract_entry(&mut self, path: &str, dest: &Path) -> Result<u64> {
        let data = self.read_entry(path)?;
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(dest, &data)?;
        Ok(data.len() as u64)
    }

    /// Extracts every file entry under the optional `prefix` to `out_dir`, preserving the
    /// internal directory layout. Returns the number of files written.
    fn extract_all(&mut self, out_dir: &Path, prefix: Option<&str>) -> Result<usize> {
        let entries: Vec<String> = self
            .entries()
            .iter()
            .filter(|e| e.kind == EntryKind::File)
            .filter(|e| match prefix {
                Some(p) => e.path.starts_with(p),
                None => true,
            })
            .map(|e| e.path.clone())
            .collect();

        let mut count = 0;
        for path in &entries {
            let mut dest = PathBuf::from(out_dir);
            for part in path.split('/') {
                dest.push(part);
            }
            self.extract_entry(path, &dest)?;
            count += 1;
        }
        Ok(count)
    }
}

/// Owned wrapper returned by [`crate::formats::open`].
pub enum OpenedArchive {
    Vpk(crate::formats::vpk::VpkArchive),
    Pak(crate::formats::pak::PakArchive),
    Wad(crate::formats::wad::WadArchive),
    Xzp(crate::formats::xzp::XzpArchive),
    Gcf(crate::formats::gcf::GcfArchive),
    Sga(crate::formats::sga::SgaArchive),
}

impl Archive for OpenedArchive {
    fn format_name(&self) -> &'static str {
        match self {
            Self::Vpk(a) => a.format_name(),
            Self::Pak(a) => a.format_name(),
            Self::Wad(a) => a.format_name(),
            Self::Xzp(a) => a.format_name(),
            Self::Gcf(a) => a.format_name(),
            Self::Sga(a) => a.format_name(),
        }
    }

    fn entries(&self) -> &[ArchiveEntry] {
        match self {
            Self::Vpk(a) => a.entries(),
            Self::Pak(a) => a.entries(),
            Self::Wad(a) => a.entries(),
            Self::Xzp(a) => a.entries(),
            Self::Gcf(a) => a.entries(),
            Self::Sga(a) => a.entries(),
        }
    }

    fn read_entry(&mut self, path: &str) -> Result<Vec<u8>> {
        match self {
            Self::Vpk(a) => a.read_entry(path),
            Self::Pak(a) => a.read_entry(path),
            Self::Wad(a) => a.read_entry(path),
            Self::Xzp(a) => a.read_entry(path),
            Self::Gcf(a) => a.read_entry(path),
            Self::Sga(a) => a.read_entry(path),
        }
    }
}

/// Helper trait alias for any input source we accept. Implemented automatically for `File`,
/// `Cursor<Vec<u8>>`, etc.
pub trait Source: Read + Seek + Send {}
impl<T: Read + Seek + Send> Source for T {}
