//! Valve Pak (VPK) reader & writer.
//!
//! Implements both version 1 and version 2 of the VPK format, including the multi-part `_dir.vpk`
//! variant which references payload bytes in sibling `_001.vpk`, `_002.vpk`, … chunk files.
//!
//! Reference: <https://developer.valvesoftware.com/wiki/VPK_(file_format)>
//!
//! The directory tree is laid out as a series of NUL-terminated strings:
//!
//! ```text
//! for each unique extension:
//!     <extension>\0
//!     for each unique directory path under that extension:
//!         <path>\0           (use ' ' for root)
//!         for each file in that directory:
//!             <stem>\0
//!             { entry header }
//!         \0
//!     \0
//! \0
//! ```
//!
//! Each entry header is:
//!
//! ```text
//!   u32 crc32
//!   u16 preload_bytes
//!   u16 archive_index    (0x7FFF means "this file" — data follows the tree)
//!   u32 entry_offset     (offset within archive_index file, or after-tree offset)
//!   u32 entry_length
//!   u16 terminator       (always 0xFFFF)
//!   u8  preload[preload_bytes]
//! ```

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::archive::{Archive, ArchiveEntry, EntryKind};
use crate::error::{Error, Result};
use crate::util::{check_crc, normalise_path, read_cstring};

const VPK_SIGNATURE: u32 = 0x55AA_1234;
const ENTRY_TERMINATOR: u16 = 0xFFFF;
const SELF_ARCHIVE_INDEX: u16 = 0x7FFF;

#[derive(Debug, Clone)]
struct VpkEntry {
    path: String,
    crc32: u32,
    preload: Vec<u8>,
    archive_index: u16,
    entry_offset: u32,
    entry_length: u32,
}

/// Reader for a `.vpk` archive.
pub struct VpkArchive {
    version: u32,
    /// Path to the root file (either single-file vpk or the `_dir.vpk`).
    root_path: PathBuf,
    /// For multi-part archives this is the byte offset to the data payload section that lives
    /// inside the `_dir.vpk` file itself (i.e. the file we're holding open).
    data_offset_in_dir: u64,
    entries: Vec<ArchiveEntry>,
    raw_entries: Vec<VpkEntry>,
    dir_file: BufReader<File>,
    is_multipart: bool,
    base_stem: Option<String>,
}

impl VpkArchive {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let mut rdr = BufReader::new(file);

        let signature = rdr.read_u32::<LittleEndian>()?;
        if signature != VPK_SIGNATURE {
            return Err(Error::BadMagic { expected: "VPK" });
        }
        let version = rdr.read_u32::<LittleEndian>()?;
        let tree_size = rdr.read_u32::<LittleEndian>()? as u64;

        match version {
            1 => { /* header is now 12 bytes */ }
            2 => {
                // Skip the remaining v2 header fields — we don't use the MD5 / signature sections
                // for read, but we do need to know they exist so the tree offset is correct.
                let _file_data_section_size = rdr.read_u32::<LittleEndian>()?;
                let _archive_md5_section_size = rdr.read_u32::<LittleEndian>()?;
                let _other_md5_section_size = rdr.read_u32::<LittleEndian>()?;
                let _signature_section_size = rdr.read_u32::<LittleEndian>()?;
            }
            v => {
                return Err(Error::UnsupportedVersion {
                    format: "VPK",
                    version: v,
                })
            }
        }

        let header_size: u64 = if version == 1 { 12 } else { 12 + 16 };
        let tree_end = header_size + tree_size;

        let raw_entries = parse_tree(&mut rdr, tree_end)?;

        let entries = raw_entries
            .iter()
            .map(|e| ArchiveEntry {
                path: e.path.clone(),
                kind: EntryKind::File,
                size: (e.entry_length as u64) + (e.preload.len() as u64),
                compressed_size: None,
                crc32: Some(e.crc32),
            })
            .collect();

        let is_multipart = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.ends_with("_dir"))
            .unwrap_or(false);

        let base_stem = if is_multipart {
            path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.trim_end_matches("_dir").to_string())
        } else {
            None
        };

        Ok(Self {
            version,
            root_path: path.to_path_buf(),
            data_offset_in_dir: tree_end,
            entries,
            raw_entries,
            dir_file: rdr,
            is_multipart,
            base_stem,
        })
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn is_multipart(&self) -> bool {
        self.is_multipart
    }

    fn locate(&self, path: &str) -> Result<&VpkEntry> {
        let normalised = normalise_path(path);
        self.raw_entries
            .iter()
            .find(|e| e.path == normalised)
            .ok_or_else(|| Error::EntryNotFound(path.to_owned()))
    }

    fn chunk_path(&self, archive_index: u16) -> Option<PathBuf> {
        let stem = self.base_stem.as_ref()?;
        let parent = self.root_path.parent()?;
        let name = format!("{stem}_{:03}.vpk", archive_index);
        Some(parent.join(name))
    }
}

impl Archive for VpkArchive {
    fn format_name(&self) -> &'static str {
        if self.version == 1 {
            "VPK v1"
        } else {
            "VPK v2"
        }
    }

    fn entries(&self) -> &[ArchiveEntry] {
        &self.entries
    }

    fn read_entry(&mut self, path: &str) -> Result<Vec<u8>> {
        let entry = self.locate(path)?.clone();
        let total_len = (entry.preload.len() as u64) + entry.entry_length as u64;
        let mut out = Vec::with_capacity(total_len as usize);
        out.extend_from_slice(&entry.preload);

        if entry.entry_length > 0 {
            if entry.archive_index == SELF_ARCHIVE_INDEX {
                self.dir_file.seek(SeekFrom::Start(
                    self.data_offset_in_dir + entry.entry_offset as u64,
                ))?;
                let mut buf = vec![0u8; entry.entry_length as usize];
                self.dir_file.read_exact(&mut buf)?;
                out.extend_from_slice(&buf);
            } else {
                let chunk_path = self.chunk_path(entry.archive_index).ok_or_else(|| {
                    Error::MissingDataPart(format!(
                        "archive index {} but archive is not multi-part",
                        entry.archive_index
                    ))
                })?;
                let mut chunk = File::open(&chunk_path).map_err(|e| {
                    Error::MissingDataPart(format!("{}: {}", chunk_path.display(), e))
                })?;
                chunk.seek(SeekFrom::Start(entry.entry_offset as u64))?;
                let mut buf = vec![0u8; entry.entry_length as usize];
                chunk.read_exact(&mut buf)?;
                out.extend_from_slice(&buf);
            }
        }

        // Verify CRC if non-zero.
        if entry.crc32 != 0 {
            let actual = crc32fast::hash(&out);
            check_crc(&entry.path, entry.crc32, actual)?;
        }

        Ok(out)
    }
}

fn parse_tree(rdr: &mut BufReader<File>, tree_end: u64) -> Result<Vec<VpkEntry>> {
    let mut entries = Vec::new();

    loop {
        if rdr.stream_position()? >= tree_end {
            break;
        }
        let extension = read_cstring(rdr)?;
        if extension.is_empty() {
            break;
        }

        loop {
            if rdr.stream_position()? >= tree_end {
                break;
            }
            let directory = read_cstring(rdr)?;
            if directory.is_empty() {
                break;
            }

            loop {
                if rdr.stream_position()? >= tree_end {
                    break;
                }
                let filename = read_cstring(rdr)?;
                if filename.is_empty() {
                    break;
                }

                let crc32 = rdr.read_u32::<LittleEndian>()?;
                let preload_bytes = rdr.read_u16::<LittleEndian>()? as usize;
                let archive_index = rdr.read_u16::<LittleEndian>()?;
                let entry_offset = rdr.read_u32::<LittleEndian>()?;
                let entry_length = rdr.read_u32::<LittleEndian>()?;
                let terminator = rdr.read_u16::<LittleEndian>()?;
                if terminator != ENTRY_TERMINATOR {
                    return Err(Error::malformed(
                        "VPK",
                        format!("entry terminator was 0x{terminator:04X}"),
                    ));
                }
                let mut preload = vec![0u8; preload_bytes];
                rdr.read_exact(&mut preload)?;

                let ext_eff = if extension == " " {
                    ""
                } else {
                    extension.as_str()
                };
                let dir_eff = if directory == " " {
                    ""
                } else {
                    directory.as_str()
                };
                let full = match (dir_eff, ext_eff) {
                    ("", "") => filename.clone(),
                    ("", ext) => format!("{filename}.{ext}"),
                    (dir, "") => format!("{dir}/{filename}"),
                    (dir, ext) => format!("{dir}/{filename}.{ext}"),
                };

                entries.push(VpkEntry {
                    path: normalise_path(&full),
                    crc32,
                    preload,
                    archive_index,
                    entry_offset,
                    entry_length,
                });
            }
        }
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}

/// Writes a brand-new VPK v1 single-file archive containing the provided `(path, bytes)` entries.
///
/// Mostly useful for round-trip tests and the "Create archive" workflow in the GUI.
pub fn write_v1<W: Write + Seek>(
    out: &mut W,
    entries: impl IntoIterator<Item = (String, Vec<u8>)>,
) -> Result<()> {
    let entries: Vec<(String, Vec<u8>)> = entries.into_iter().collect();

    // Group entries by (extension, directory).
    use std::collections::BTreeMap;
    type FilesByDir = BTreeMap<String, Vec<(String, Vec<u8>)>>;
    let mut grouped: BTreeMap<String, FilesByDir> = BTreeMap::new();
    for (path, data) in entries {
        let normalised = normalise_path(&path);
        let (dir, name) = split_dir_and_name(&normalised);
        let (stem, ext) = split_stem_and_ext(&name);
        grouped
            .entry(ext)
            .or_default()
            .entry(dir)
            .or_default()
            .push((stem, data));
    }

    // Reserve header space.
    let header_pos = out.stream_position()?;
    out.write_u32::<LittleEndian>(VPK_SIGNATURE)?;
    out.write_u32::<LittleEndian>(1)?; // version
    out.write_u32::<LittleEndian>(0)?; // tree_size placeholder

    let tree_start = out.stream_position()?;

    // First pass: emit the tree with placeholder offsets; collect the patch table.
    struct Patch {
        offset_field_pos: u64,
        data: Vec<u8>,
    }
    let mut patches: Vec<Patch> = Vec::new();

    for (ext, dirs) in &grouped {
        let ext_serialised = if ext.is_empty() { " " } else { ext.as_str() };
        write_cstring(out, ext_serialised)?;
        for (dir, files) in dirs {
            let dir_serialised = if dir.is_empty() { " " } else { dir.as_str() };
            write_cstring(out, dir_serialised)?;
            for (stem, data) in files {
                write_cstring(out, stem)?;
                let crc = crc32fast::hash(data);
                out.write_u32::<LittleEndian>(crc)?;
                out.write_u16::<LittleEndian>(0)?; // no preload bytes
                out.write_u16::<LittleEndian>(SELF_ARCHIVE_INDEX)?;
                let offset_field_pos = out.stream_position()?;
                out.write_u32::<LittleEndian>(0)?; // entry_offset placeholder
                out.write_u32::<LittleEndian>(data.len() as u32)?;
                out.write_u16::<LittleEndian>(ENTRY_TERMINATOR)?;
                patches.push(Patch {
                    offset_field_pos,
                    data: data.clone(),
                });
            }
            out.write_u8(0)?; // end of files in this directory
        }
        out.write_u8(0)?; // end of directories under this extension
    }
    out.write_u8(0)?; // end of tree

    let tree_end = out.stream_position()?;
    let tree_size = tree_end - tree_start;

    // Append payloads, recording their offsets relative to tree_end.
    for patch in &patches {
        let payload_offset = out.stream_position()? - tree_end;
        out.write_all(&patch.data)?;
        let here = out.stream_position()?;
        out.seek(SeekFrom::Start(patch.offset_field_pos))?;
        out.write_u32::<LittleEndian>(payload_offset as u32)?;
        out.seek(SeekFrom::Start(here))?;
    }

    // Patch tree_size in the header.
    let end_pos = out.stream_position()?;
    out.seek(SeekFrom::Start(header_pos + 8))?;
    out.write_u32::<LittleEndian>(tree_size as u32)?;
    out.seek(SeekFrom::Start(end_pos))?;
    Ok(())
}

fn write_cstring<W: Write>(out: &mut W, s: &str) -> Result<()> {
    out.write_all(s.as_bytes())?;
    out.write_u8(0)?;
    Ok(())
}

fn split_dir_and_name(path: &str) -> (String, String) {
    match path.rsplit_once('/') {
        Some((d, n)) => (d.to_string(), n.to_string()),
        None => (String::new(), path.to_string()),
    }
}

fn split_stem_and_ext(name: &str) -> (String, String) {
    match name.rsplit_once('.') {
        Some((stem, ext)) => (stem.to_string(), ext.to_string()),
        None => (name.to_string(), String::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn roundtrip_v1() {
        let payload = vec![
            ("hello.txt".to_string(), b"world".to_vec()),
            (
                "materials/dev/sample.vtf".to_string(),
                (0..=255u8).collect::<Vec<_>>(),
            ),
            ("empty".to_string(), Vec::new()),
        ];

        let mut buf = Vec::new();
        {
            let mut cur = Cursor::new(&mut buf);
            write_v1(&mut cur, payload.clone()).unwrap();
        }

        let tmp = std::env::temp_dir().join("sv-test.vpk");
        std::fs::write(&tmp, &buf).unwrap();
        let mut a = VpkArchive::open(&tmp).unwrap();
        assert_eq!(a.entries().len(), 3);
        let got = a.read_entry("hello.txt").unwrap();
        assert_eq!(got, b"world");
        let got = a.read_entry("materials/dev/sample.vtf").unwrap();
        assert_eq!(got, (0..=255u8).collect::<Vec<_>>());
        let got = a.read_entry("empty").unwrap();
        assert!(got.is_empty());
        std::fs::remove_file(&tmp).ok();
    }
}
