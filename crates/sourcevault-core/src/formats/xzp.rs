//! XZP (xZip) reader — used by the Xbox port of Half-Life 2 and a handful of mods.
//!
//! The header starts with the ASCII string `piZx` and contains a directory of fixed-length
//! filename entries plus a parallel array of `(name_hash, offset, length)` triples. We use the
//! filename strings exclusively, so name collisions in the hash table are not an issue.
//!
//! There are at least two variants in the wild:
//!   * `xZip v1` — magic `piZx`, 36-byte header, two parallel arrays.
//!   * `xZip v6` — same magic but a slightly different header layout.
//!
//! We support the most common `v1` layout from the Source SDK leak.

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::archive::{Archive, ArchiveEntry, EntryKind};
use crate::error::{Error, Result};
use crate::util::normalise_path;

#[derive(Debug, Clone)]
struct XzpEntry {
    path: String,
    offset: u32,
    length: u32,
}

pub struct XzpArchive {
    file: BufReader<File>,
    entries: Vec<ArchiveEntry>,
    raw: Vec<XzpEntry>,
}

impl XzpArchive {
    pub fn open(path: &Path) -> Result<Self> {
        let mut file = BufReader::new(File::open(path)?);
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        if &magic != b"piZx" {
            return Err(Error::BadMagic { expected: "XZP" });
        }
        let version = file.read_u32::<LittleEndian>()?;
        let preload_directory_entries = file.read_u32::<LittleEndian>()?;
        let directory_entries = file.read_u32::<LittleEndian>()?;
        let _preload_bytes = file.read_u32::<LittleEndian>()?;
        let _header_length = file.read_u32::<LittleEndian>()?;
        let directory_item_offset = file.read_u32::<LittleEndian>()?;
        let directory_item_count = file.read_u32::<LittleEndian>()?;
        let _directory_item_length = file.read_u32::<LittleEndian>()?;

        if version != 1 && version != 6 {
            return Err(Error::UnsupportedVersion {
                format: "XZP",
                version,
            });
        }

        // Directory entries: 12 bytes each — { u32 name_crc, u32 entry_length, u32 entry_offset }
        let mut entries_raw: Vec<(u32, u32, u32)> = Vec::with_capacity(directory_entries as usize);
        for _ in 0..directory_entries {
            let crc = file.read_u32::<LittleEndian>()?;
            let length = file.read_u32::<LittleEndian>()?;
            let offset = file.read_u32::<LittleEndian>()?;
            entries_raw.push((crc, length, offset));
        }
        // Skip preload directory section (each 4 bytes, u32 index into entries_raw).
        let _ = file.seek(SeekFrom::Current((preload_directory_entries as i64) * 4))?;

        // Directory items (filename listing): each item is `u32 name_offset, u32 time_created,
        // u32 entry_index` (12 bytes), followed by a NUL-terminated string table.
        file.seek(SeekFrom::Start(directory_item_offset as u64))?;
        let mut items = Vec::with_capacity(directory_item_count as usize);
        for _ in 0..directory_item_count {
            let name_offset = file.read_u32::<LittleEndian>()?;
            let _time_created = file.read_u32::<LittleEndian>()?;
            let entry_index = file.read_u32::<LittleEndian>()?;
            items.push((name_offset, entry_index));
        }
        // Read the string table that immediately follows.
        let string_table_start = file.stream_position()?;
        // We don't know the table length; read until EOF for safety.
        let file_len = file.get_ref().metadata()?.len();
        let mut string_table = vec![0u8; (file_len - string_table_start).min(1 << 24) as usize];
        let read = file.read(&mut string_table)?;
        string_table.truncate(read);

        let mut raw = Vec::new();
        for (name_offset, entry_index) in items {
            let off = name_offset as usize;
            if off >= string_table.len() {
                continue;
            }
            let end = string_table[off..]
                .iter()
                .position(|&b| b == 0)
                .map(|p| off + p)
                .unwrap_or(string_table.len());
            let name = String::from_utf8_lossy(&string_table[off..end]).into_owned();
            if let Some((_, length, offset)) = entries_raw.get(entry_index as usize) {
                raw.push(XzpEntry {
                    path: normalise_path(&name),
                    offset: *offset,
                    length: *length,
                });
            }
        }
        raw.sort_by(|a, b| a.path.cmp(&b.path));

        let entries = raw
            .iter()
            .map(|e| ArchiveEntry {
                path: e.path.clone(),
                kind: EntryKind::File,
                size: e.length as u64,
                compressed_size: None,
                crc32: None,
            })
            .collect();

        Ok(Self { file, entries, raw })
    }
}

impl Archive for XzpArchive {
    fn format_name(&self) -> &'static str {
        "XZP"
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
        let mut buf = vec![0u8; entry.length as usize];
        self.file.read_exact(&mut buf)?;
        Ok(buf)
    }
}
