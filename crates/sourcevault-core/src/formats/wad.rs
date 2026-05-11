//! GoldSrc / Quake WAD reader (WAD2 and WAD3).
//!
//! Layout:
//!
//! ```text
//! Header:
//!   char[4] magic        ("WAD2" or "WAD3")
//!   u32     num_entries
//!   u32     dir_offset
//!
//! Directory entry (32 bytes):
//!   u32     entry_offset
//!   u32     disk_size
//!   u32     entry_size
//!   u8      file_type
//!   u8      compressed
//!   u16     padding
//!   char[16] name (NUL-padded)
//! ```

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::archive::{Archive, ArchiveEntry, EntryKind};
use crate::error::{Error, Result};
use crate::util::{normalise_path, read_fixed_string};

#[derive(Debug, Clone)]
struct WadEntry {
    path: String,
    offset: u32,
    disk_size: u32,
    entry_size: u32,
    file_type: u8,
    compressed: bool,
}

pub struct WadArchive {
    file: BufReader<File>,
    version: &'static str,
    entries: Vec<ArchiveEntry>,
    raw: Vec<WadEntry>,
}

impl WadArchive {
    pub fn open(path: &Path) -> Result<Self> {
        let mut file = BufReader::new(File::open(path)?);
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        let version = match &magic {
            b"WAD2" => "WAD2",
            b"WAD3" => "WAD3",
            _ => {
                return Err(Error::BadMagic {
                    expected: "WAD2/WAD3",
                })
            }
        };
        let num_entries = file.read_u32::<LittleEndian>()?;
        let dir_offset = file.read_u32::<LittleEndian>()?;

        file.seek(SeekFrom::Start(dir_offset as u64))?;
        let mut raw = Vec::with_capacity(num_entries as usize);
        for _ in 0..num_entries {
            let offset = file.read_u32::<LittleEndian>()?;
            let disk_size = file.read_u32::<LittleEndian>()?;
            let entry_size = file.read_u32::<LittleEndian>()?;
            let file_type = file.read_u8()?;
            let compressed = file.read_u8()? != 0;
            let _padding = file.read_u16::<LittleEndian>()?;
            let name = read_fixed_string(&mut file, 16)?;
            let path = if name.is_empty() {
                format!("entry_{offset:08X}")
            } else {
                normalise_path(&name)
            };
            raw.push(WadEntry {
                path,
                offset,
                disk_size,
                entry_size,
                file_type,
                compressed,
            });
        }
        raw.sort_by(|a, b| a.path.cmp(&b.path));

        let entries = raw
            .iter()
            .map(|e| ArchiveEntry {
                path: e.path.clone(),
                kind: EntryKind::File,
                size: e.entry_size as u64,
                compressed_size: Some(e.disk_size as u64),
                crc32: None,
            })
            .collect();

        Ok(Self {
            file,
            version,
            entries,
            raw,
        })
    }

    /// Returns the per-entry file_type byte (0x42=qpic, 0x43=miptex, 0x46=font, 0x40=spraydecal).
    pub fn file_type_of(&self, path: &str) -> Option<u8> {
        let normalised = normalise_path(path);
        self.raw
            .iter()
            .find(|e| e.path == normalised)
            .map(|e| e.file_type)
    }
}

impl Archive for WadArchive {
    fn format_name(&self) -> &'static str {
        // SAFETY: `self.version` is a 'static str from the match above.
        self.version
    }

    fn entries(&self) -> &[ArchiveEntry] {
        &self.entries
    }

    fn read_entry(&mut self, path: &str) -> Result<Vec<u8>> {
        let normalised = normalise_path(path);
        let entry = self
            .raw
            .iter()
            .find(|e| e.path == normalised)
            .ok_or_else(|| Error::EntryNotFound(path.to_owned()))?
            .clone();
        self.file.seek(SeekFrom::Start(entry.offset as u64))?;
        let mut buf = vec![0u8; entry.disk_size as usize];
        self.file.read_exact(&mut buf)?;
        if entry.compressed {
            // Half-Life never actually shipped compressed WAD entries; treat as unsupported.
            return Err(Error::Unsupported);
        }
        Ok(buf)
    }
}
