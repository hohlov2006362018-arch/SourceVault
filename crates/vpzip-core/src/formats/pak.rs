//! Quake-style PAK reader & writer (also used by GoldSrc and early Source mods).
//!
//! Layout:
//!
//! ```text
//! Header:
//!   char[4] magic = "PACK"
//!   i32     dir_offset
//!   i32     dir_size      (multiple of 64)
//!
//! Directory entries (64 bytes each):
//!   char[56] name (NUL-padded)
//!   i32      offset
//!   i32      size
//! ```

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::archive::{Archive, ArchiveEntry, EntryKind};
use crate::error::{Error, Result};
use crate::util::{normalise_path, read_fixed_string};

const PAK_ENTRY_SIZE: usize = 64;

#[derive(Debug, Clone)]
struct PakEntry {
    path: String,
    offset: u32,
    size: u32,
}

pub struct PakArchive {
    file: BufReader<File>,
    entries: Vec<ArchiveEntry>,
    raw: Vec<PakEntry>,
}

impl PakArchive {
    pub fn open(path: &Path) -> Result<Self> {
        let mut file = BufReader::new(File::open(path)?);
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic)?;
        if &magic != b"PACK" {
            return Err(Error::BadMagic { expected: "PAK" });
        }
        let dir_offset = file.read_u32::<LittleEndian>()?;
        let dir_size = file.read_u32::<LittleEndian>()?;
        if !(dir_size as usize).is_multiple_of(PAK_ENTRY_SIZE) {
            return Err(Error::malformed(
                "PAK",
                format!("directory size {dir_size} is not a multiple of 64"),
            ));
        }
        let count = dir_size as usize / PAK_ENTRY_SIZE;

        file.seek(SeekFrom::Start(dir_offset as u64))?;
        let mut raw = Vec::with_capacity(count);
        for _ in 0..count {
            let name = read_fixed_string(&mut file, 56)?;
            let offset = file.read_u32::<LittleEndian>()?;
            let size = file.read_u32::<LittleEndian>()?;
            raw.push(PakEntry {
                path: normalise_path(&name),
                offset,
                size,
            });
        }
        raw.sort_by(|a, b| a.path.cmp(&b.path));

        let entries = raw
            .iter()
            .map(|e| ArchiveEntry {
                path: e.path.clone(),
                kind: EntryKind::File,
                size: e.size as u64,
                compressed_size: None,
                crc32: None,
            })
            .collect();

        Ok(Self { file, entries, raw })
    }
}

impl Archive for PakArchive {
    fn format_name(&self) -> &'static str {
        "PAK"
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
        let mut buf = vec![0u8; entry.size as usize];
        self.file.read_exact(&mut buf)?;
        Ok(buf)
    }
}

/// Writes a Quake-style PAK archive containing the provided entries.
pub fn write<W: Write + Seek>(
    out: &mut W,
    entries: impl IntoIterator<Item = (String, Vec<u8>)>,
) -> Result<()> {
    let entries: Vec<(String, Vec<u8>)> = entries.into_iter().collect();
    for (name, _) in &entries {
        if name.len() > 55 {
            return Err(Error::malformed(
                "PAK",
                format!("file name `{name}` exceeds 55 character limit"),
            ));
        }
    }

    out.write_all(b"PACK")?;
    let header_pos = out.stream_position()?;
    out.write_u32::<LittleEndian>(0)?; // dir_offset placeholder
    out.write_u32::<LittleEndian>(0)?; // dir_size placeholder

    let mut written = Vec::new();
    for (name, data) in &entries {
        let offset = out.stream_position()? as u32;
        out.write_all(data)?;
        written.push((name.clone(), offset, data.len() as u32));
    }

    let dir_offset = out.stream_position()? as u32;
    for (name, offset, size) in &written {
        let mut name_buf = [0u8; 56];
        let bytes = name.as_bytes();
        name_buf[..bytes.len()].copy_from_slice(bytes);
        out.write_all(&name_buf)?;
        out.write_u32::<LittleEndian>(*offset)?;
        out.write_u32::<LittleEndian>(*size)?;
    }
    let dir_size = (written.len() * PAK_ENTRY_SIZE) as u32;
    let end = out.stream_position()?;
    out.seek(SeekFrom::Start(header_pos))?;
    out.write_u32::<LittleEndian>(dir_offset)?;
    out.write_u32::<LittleEndian>(dir_size)?;
    out.seek(SeekFrom::Start(end))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn roundtrip_pak() {
        let payload = vec![
            ("readme.txt".to_string(), b"hello quake".to_vec()),
            (
                "maps/start.bsp".to_string(),
                (0..1024u32).map(|i| i as u8).collect(),
            ),
        ];
        let mut buf = Vec::new();
        write(&mut Cursor::new(&mut buf), payload.clone()).unwrap();
        let tmp = std::env::temp_dir().join("sv-test.pak");
        std::fs::write(&tmp, &buf).unwrap();
        let mut a = PakArchive::open(&tmp).unwrap();
        assert_eq!(a.entries().len(), 2);
        assert_eq!(a.read_entry("readme.txt").unwrap(), b"hello quake");
        std::fs::remove_file(&tmp).ok();
    }
}
