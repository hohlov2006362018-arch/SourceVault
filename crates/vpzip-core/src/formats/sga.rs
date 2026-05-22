//! Relic SGA archive reader (used by Dawn of War / Company of Heroes; bundled into Source-based
//! tooling like GCFScape for completeness).
//!
//! Multiple versions exist (v2, v4, v5, v7, v9). The on-disk layouts diverge significantly between
//! v4-/v5- and v7-/v9-era files. This reader supports v4 and v5 — which cover Dawn of War 1,
//! Dawn of War 40K and Company of Heroes 1.
//!
//! Layout (v4/v5):
//!
//! ```text
//! Header:
//!   char[8]   magic = "_ARCHIVE"
//!   u32       version            (4 or 5)
//!   u8[16]    file_md5
//!   wchar[64] archive_name       (UTF-16 LE, NUL-padded)
//!   u8[16]    header_md5
//!   u32       data_offset
//!   u32       _data_header_size  (v5 only)
//!
//! Table-of-contents header (immediately follows file header):
//!   u32 toc_offset
//!   u16 toc_count
//!   u32 folder_offset
//!   u16 folder_count
//!   u32 file_offset
//!   u16 file_count
//!   u32 string_offset
//!   u16 string_count
//!
//! TOC entry (size = 0x88):
//!   char[64] alias
//!   char[64] name
//!   u16      folder_start
//!   u16      folder_end
//!   u16      file_start
//!   u16      file_end
//!   u16      folder_root
//!
//! Folder entry (16 bytes):
//!   u32 name_offset
//!   u16 folder_start
//!   u16 folder_end
//!   u16 file_start
//!   u16 file_end
//!
//! File entry (20 bytes, v4) / (24 bytes, v5):
//!   u32 name_offset
//!   u32 data_offset
//!   u32 compressed_length
//!   u32 uncompressed_length
//!   u32 modified              (v5 only)
//!   u8  verification_type
//!   u8  storage_type          (0 = none, 1 = stream, 2 = buffer)
//! ```

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;

use crate::archive::{Archive, ArchiveEntry, EntryKind};
use crate::error::{Error, Result};
use crate::util::{normalise_path, read_utf16_string};

#[derive(Debug, Clone)]
struct SgaFile {
    path: String,
    data_offset: u32,
    compressed_length: u32,
    uncompressed_length: u32,
    storage_type: u8,
}

pub struct SgaArchive {
    file: BufReader<File>,
    version: u32,
    data_offset: u64,
    files: Vec<SgaFile>,
    entries: Vec<ArchiveEntry>,
}

impl SgaArchive {
    pub fn open(path: &Path) -> Result<Self> {
        let mut file = BufReader::new(File::open(path)?);
        let mut magic = [0u8; 8];
        file.read_exact(&mut magic)?;
        if &magic != b"_ARCHIVE" {
            return Err(Error::BadMagic { expected: "SGA" });
        }
        let version = file.read_u32::<LittleEndian>()?;
        if !matches!(version, 4 | 5) {
            return Err(Error::UnsupportedVersion {
                format: "SGA",
                version,
            });
        }

        let mut _md5 = [0u8; 16];
        file.read_exact(&mut _md5)?;
        let _archive_name = read_utf16_string(&mut file, 64)?;
        // wide string is fixed 64 chars (128 bytes); skip remainder if read terminated early.
        let cur = file.stream_position()?;
        let target = cur - (_archive_name.encode_utf16().count() as u64 + 1) * 2 + 128;
        if target > cur {
            file.seek(SeekFrom::Start(target))?;
        }
        let mut _header_md5 = [0u8; 16];
        file.read_exact(&mut _header_md5)?;
        let data_offset = file.read_u32::<LittleEndian>()? as u64;
        if version == 5 {
            let _data_header_size = file.read_u32::<LittleEndian>()?;
        }

        // TOC pointer block.
        let toc_offset = file.read_u32::<LittleEndian>()?;
        let _toc_count = file.read_u16::<LittleEndian>()?;
        let folder_offset = file.read_u32::<LittleEndian>()?;
        let folder_count = file.read_u16::<LittleEndian>()?;
        let file_offset = file.read_u32::<LittleEndian>()?;
        let file_count = file.read_u16::<LittleEndian>()?;
        let string_offset = file.read_u32::<LittleEndian>()?;
        let _string_count = file.read_u16::<LittleEndian>()?;

        let toc_block_start = file.stream_position()? - 28; // start of pointer block (per spec, v4/v5)

        // Read all folders.
        file.seek(SeekFrom::Start(toc_block_start + folder_offset as u64))?;
        let mut folders = Vec::with_capacity(folder_count as usize);
        for _ in 0..folder_count {
            let name_offset = file.read_u32::<LittleEndian>()?;
            let folder_start = file.read_u16::<LittleEndian>()?;
            let folder_end = file.read_u16::<LittleEndian>()?;
            let file_start = file.read_u16::<LittleEndian>()?;
            let file_end = file.read_u16::<LittleEndian>()?;
            folders.push((name_offset, folder_start, folder_end, file_start, file_end));
        }

        // Read all files.
        file.seek(SeekFrom::Start(toc_block_start + file_offset as u64))?;
        let mut raw_files = Vec::with_capacity(file_count as usize);
        for _ in 0..file_count {
            let name_offset = file.read_u32::<LittleEndian>()?;
            let f_data_offset = file.read_u32::<LittleEndian>()?;
            let compressed_length = file.read_u32::<LittleEndian>()?;
            let uncompressed_length = file.read_u32::<LittleEndian>()?;
            if version == 5 {
                let _modified = file.read_u32::<LittleEndian>()?;
            }
            let _verification = file.read_u8()?;
            let storage_type = file.read_u8()?;
            raw_files.push((
                name_offset,
                f_data_offset,
                compressed_length,
                uncompressed_length,
                storage_type,
            ));
        }

        // Read string table.
        file.seek(SeekFrom::Start(toc_block_start + string_offset as u64))?;
        let mut string_table = Vec::new();
        file.read_to_end(&mut string_table)?;

        // Walk folder hierarchy and emit files.
        let mut files_out: Vec<SgaFile> = Vec::new();
        fn walk(
            folders: &[(u32, u16, u16, u16, u16)],
            raw_files: &[(u32, u32, u32, u32, u8)],
            string_table: &[u8],
            folder_index: u16,
            base: &str,
            out: &mut Vec<SgaFile>,
        ) {
            if folder_index as usize >= folders.len() {
                return;
            }
            let (name_offset, fs, fe, fis, fie) = folders[folder_index as usize];
            let folder_name = read_string(string_table, name_offset);
            let current = if folder_name.is_empty() {
                base.to_string()
            } else if base.is_empty() {
                folder_name
            } else {
                format!("{base}/{folder_name}")
            };

            for fi in fis..fie {
                if (fi as usize) >= raw_files.len() {
                    continue;
                }
                let (n_off, d_off, c_len, u_len, storage) = raw_files[fi as usize];
                let name = read_string(string_table, n_off);
                let path = if current.is_empty() {
                    name
                } else {
                    format!("{current}/{name}")
                };
                out.push(SgaFile {
                    path: normalise_path(&path),
                    data_offset: d_off,
                    compressed_length: c_len,
                    uncompressed_length: u_len,
                    storage_type: storage,
                });
            }

            for sub in fs..fe {
                walk(folders, raw_files, string_table, sub, &current, out);
            }
        }
        if folder_count > 0 {
            walk(&folders, &raw_files, &string_table, 0, "", &mut files_out);
        }
        files_out.sort_by(|a, b| a.path.cmp(&b.path));

        let entries = files_out
            .iter()
            .map(|f| ArchiveEntry {
                path: f.path.clone(),
                kind: EntryKind::File,
                size: f.uncompressed_length as u64,
                compressed_size: Some(f.compressed_length as u64),
                crc32: None,
            })
            .collect();

        // Skip the (unused) TOC entries themselves; we already have folders.
        let _ = toc_offset;

        Ok(Self {
            file,
            version,
            data_offset,
            files: files_out,
            entries,
        })
    }
}

fn read_string(table: &[u8], offset: u32) -> String {
    let off = offset as usize;
    if off >= table.len() {
        return String::new();
    }
    let end = table[off..]
        .iter()
        .position(|&b| b == 0)
        .map(|p| off + p)
        .unwrap_or(table.len());
    String::from_utf8_lossy(&table[off..end]).into_owned()
}

impl Archive for SgaArchive {
    fn format_name(&self) -> &'static str {
        if self.version == 4 {
            "SGA v4"
        } else {
            "SGA v5"
        }
    }

    fn entries(&self) -> &[ArchiveEntry] {
        &self.entries
    }

    fn read_entry(&mut self, path: &str) -> Result<Vec<u8>> {
        let normalised = normalise_path(path);
        let file = self
            .files
            .iter()
            .find(|f| f.path == normalised)
            .ok_or_else(|| Error::EntryNotFound(path.to_owned()))?
            .clone();
        let absolute = self.data_offset + file.data_offset as u64;
        self.file.seek(SeekFrom::Start(absolute))?;
        let mut raw = vec![0u8; file.compressed_length as usize];
        self.file.read_exact(&mut raw)?;

        if file.storage_type == 0 || file.compressed_length == file.uncompressed_length {
            Ok(raw)
        } else {
            // Zlib-compressed.
            let mut decoder = ZlibDecoder::new(&raw[..]);
            let mut out = Vec::with_capacity(file.uncompressed_length as usize);
            decoder.read_to_end(&mut out)?;
            Ok(out)
        }
    }
}
