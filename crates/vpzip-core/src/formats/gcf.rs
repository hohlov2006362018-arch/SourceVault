//! GCF (Game Cache File) reader — Steam's pre-SteamPipe content format.
//!
//! GCF is a complex container with multiple internal indices. The high-level layout, as
//! reverse-engineered by the HLLib / NCFLib authors, is:
//!
//! 1.  File header              (28 bytes)
//! 2.  Block allocation header  (32 bytes) + block entries
//! 3.  Fragmentation map header + fragmentation map
//! 4.  Manifest header          (56 bytes)
//! 5.  Manifest entries, name table, hash table, minimum footprint, user-config
//! 6.  Manifest map (parent-block-index per file)
//! 7.  Checksums   (skipped here)
//! 8.  Data block header + raw data blocks
//!
//! We only need enough of this to enumerate the manifest and read uncompressed data blocks.
//! The reader is intentionally read-only.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::archive::{Archive, ArchiveEntry, EntryKind};
use crate::error::{Error, Result};
use crate::util::normalise_path;

/// Manifest entry flag bit indicating a directory.
const FLAG_IS_DIRECTORY: u32 = 0x4000_0000;

#[derive(Debug, Clone)]
struct GcfFile {
    path: String,
    size: u32,
    /// Index into the manifest map table (one entry per manifest entry).
    manifest_index: u32,
}

pub struct GcfArchive {
    file: BufReader<File>,
    entries: Vec<ArchiveEntry>,
    files: Vec<GcfFile>,
    /// Cluster size in bytes (block size).
    cluster_size: u32,
    /// Offset of the data section in the file.
    data_offset: u64,
    /// For each manifest entry, the index of its first cluster (0xFFFFFFFF if none).
    first_cluster_of_manifest: Vec<u32>,
    /// next_cluster[i] gives the cluster that follows cluster i in a chain (0xFFFFFFFF = end).
    next_cluster: Vec<u32>,
}

impl GcfArchive {
    pub fn open(path: &Path) -> Result<Self> {
        let mut file = BufReader::new(File::open(path)?);

        // ----------------------- File header (28 bytes) -----------------------
        let header_version = file.read_u32::<LittleEndian>()?;
        let cache_type = file.read_u32::<LittleEndian>()?;
        let format_version = file.read_u32::<LittleEndian>()?;
        let _app_id = file.read_u32::<LittleEndian>()?;
        let _app_version = file.read_u32::<LittleEndian>()?;
        let _is_mounted = file.read_u32::<LittleEndian>()?;
        let _dummy0 = file.read_u32::<LittleEndian>()?;

        if header_version != 1 || cache_type != 1 {
            return Err(Error::malformed(
                "GCF",
                format!("unexpected header_version={header_version}, cache_type={cache_type}"),
            ));
        }
        if !matches!(format_version, 1 | 3 | 5 | 6) {
            return Err(Error::UnsupportedVersion {
                format: "GCF",
                version: format_version,
            });
        }

        let _file_size = file.read_u32::<LittleEndian>()?;
        let cluster_size = file.read_u32::<LittleEndian>()?;
        let _cluster_count = file.read_u32::<LittleEndian>()?;
        let _checksum = file.read_u32::<LittleEndian>()?;

        // ------------------ Block allocation header (32 bytes) ------------------
        let block_count = file.read_u32::<LittleEndian>()?;
        let _blocks_used = file.read_u32::<LittleEndian>()?;
        let _last_block_used = file.read_u32::<LittleEndian>()?;
        let _dummy1 = file.read_u32::<LittleEndian>()?;
        let _dummy2 = file.read_u32::<LittleEndian>()?;
        let _dummy3 = file.read_u32::<LittleEndian>()?;
        let _dummy4 = file.read_u32::<LittleEndian>()?;
        let _block_header_checksum = file.read_u32::<LittleEndian>()?;

        // Block entries: each entry is 28 bytes. We need (manifest_index, first_cluster).
        // Layout: flags(u16), dummy0(u16), file_offset(u32), file_size(u32), first_data_index(u32),
        //         next_block(u32), prev_block(u32), manifest_index(u32)
        let mut block_first_cluster: HashMap<u32, u32> = HashMap::new();
        for _ in 0..block_count {
            let _flags = file.read_u16::<LittleEndian>()?;
            let _dummy = file.read_u16::<LittleEndian>()?;
            let _file_offset = file.read_u32::<LittleEndian>()?;
            let _file_size = file.read_u32::<LittleEndian>()?;
            let first_cluster = file.read_u32::<LittleEndian>()?;
            let _next_block = file.read_u32::<LittleEndian>()?;
            let _prev_block = file.read_u32::<LittleEndian>()?;
            let manifest_index = file.read_u32::<LittleEndian>()?;
            // First wins in case multiple blocks share a manifest index (unusual).
            block_first_cluster
                .entry(manifest_index)
                .or_insert(first_cluster);
        }

        // ---------------- Fragmentation map header (16 bytes) + table ----------------
        let cluster_count = file.read_u32::<LittleEndian>()?;
        let _first_unused_entry = file.read_u32::<LittleEndian>()?;
        let _terminator = file.read_u32::<LittleEndian>()?;
        let _frag_checksum = file.read_u32::<LittleEndian>()?;

        let mut next_cluster = Vec::with_capacity(cluster_count as usize);
        for _ in 0..cluster_count {
            next_cluster.push(file.read_u32::<LittleEndian>()?);
        }

        // ---------------- Manifest header (56 bytes) ----------------
        let _manifest_header_version = file.read_u32::<LittleEndian>()?;
        let _manifest_app_id = file.read_u32::<LittleEndian>()?;
        let _manifest_app_version = file.read_u32::<LittleEndian>()?;
        let node_count = file.read_u32::<LittleEndian>()?;
        let _file_count = file.read_u32::<LittleEndian>()?;
        let _compression_block_size = file.read_u32::<LittleEndian>()?;
        let _binary_size = file.read_u32::<LittleEndian>()?;
        let name_size = file.read_u32::<LittleEndian>()?;
        let hash_table_key_count = file.read_u32::<LittleEndian>()?;
        let minimum_footprint_count = file.read_u32::<LittleEndian>()?;
        let user_config_count = file.read_u32::<LittleEndian>()?;
        let _bitmask = file.read_u32::<LittleEndian>()?;
        let _fingerprint = file.read_u32::<LittleEndian>()?;
        let _manifest_checksum = file.read_u32::<LittleEndian>()?;

        // Manifest entries: 28 bytes each — name_offset(u32), item_size(u32), file_id(u32),
        //                                       directory_type(u32), parent_index(u32),
        //                                       next_index(u32), first_index(u32).
        let mut name_offsets = Vec::with_capacity(node_count as usize);
        let mut item_sizes = Vec::with_capacity(node_count as usize);
        let mut directory_types = Vec::with_capacity(node_count as usize);
        let mut parent_indices = Vec::with_capacity(node_count as usize);
        let mut next_indices = Vec::with_capacity(node_count as usize);
        let mut first_child_indices = Vec::with_capacity(node_count as usize);

        for _ in 0..node_count {
            name_offsets.push(file.read_u32::<LittleEndian>()?);
            item_sizes.push(file.read_u32::<LittleEndian>()?);
            let _file_id = file.read_u32::<LittleEndian>()?;
            directory_types.push(file.read_u32::<LittleEndian>()?);
            parent_indices.push(file.read_u32::<LittleEndian>()?);
            next_indices.push(file.read_u32::<LittleEndian>()?);
            first_child_indices.push(file.read_u32::<LittleEndian>()?);
        }

        // Name table.
        let mut name_table = vec![0u8; name_size as usize];
        file.read_exact(&mut name_table)?;

        // Skip hash table, minimum footprint and user config sections.
        let hash_bytes = (hash_table_key_count + node_count) as u64 * 4;
        let min_bytes = minimum_footprint_count as u64 * 4;
        let user_bytes = user_config_count as u64 * 4;
        file.seek(SeekFrom::Current(
            (hash_bytes + min_bytes + user_bytes) as i64,
        ))?;

        // ---------------- Manifest map header (8 bytes) + table ----------------
        let _map_header_version = file.read_u32::<LittleEndian>()?;
        let _map_dummy = file.read_u32::<LittleEndian>()?;
        let mut manifest_map = Vec::with_capacity(node_count as usize);
        for _ in 0..node_count {
            manifest_map.push(file.read_u32::<LittleEndian>()?);
        }
        let _ = manifest_map; // currently unused; reserved for future use

        // ---------------- Checksum section (variable) — skip ----------------
        let _checksum_header_version = file.read_u32::<LittleEndian>()?;
        let checksum_size = file.read_u32::<LittleEndian>()?;
        file.seek(SeekFrom::Current(checksum_size as i64))?;

        // ---------------- Data block header (24 bytes) ----------------
        let _data_header_version = file.read_u32::<LittleEndian>()?;
        let _data_cluster_count = file.read_u32::<LittleEndian>()?;
        let _data_cluster_size = file.read_u32::<LittleEndian>()?;
        let _data_first_cluster_offset = file.read_u32::<LittleEndian>()?;
        let _data_clusters_used = file.read_u32::<LittleEndian>()?;
        let _data_checksum = file.read_u32::<LittleEndian>()?;
        let data_offset = file.stream_position()?;

        // Build the absolute path for each manifest entry that represents a file.
        let mut files = Vec::new();
        let mut first_cluster_of_manifest = vec![0xFFFF_FFFFu32; node_count as usize];
        for i in 0..node_count as usize {
            // Build path by walking parent chain.
            let mut parts: Vec<String> = Vec::new();
            let mut cur = i as u32;
            while cur != 0xFFFF_FFFF {
                let name = read_name(&name_table, name_offsets[cur as usize]);
                if !name.is_empty() {
                    parts.push(name);
                }
                let parent = parent_indices[cur as usize];
                if parent == cur {
                    break;
                }
                cur = parent;
            }
            parts.reverse();
            let path = parts.join("/");
            let is_dir = directory_types[i] & FLAG_IS_DIRECTORY != 0;
            if is_dir || path.is_empty() {
                continue;
            }
            if let Some(&fc) = block_first_cluster.get(&(i as u32)) {
                first_cluster_of_manifest[i] = fc;
            }
            files.push(GcfFile {
                path: normalise_path(&path),
                size: item_sizes[i],
                manifest_index: i as u32,
            });
        }
        files.sort_by(|a, b| a.path.cmp(&b.path));

        let entries = files
            .iter()
            .map(|f| ArchiveEntry {
                path: f.path.clone(),
                kind: EntryKind::File,
                size: f.size as u64,
                compressed_size: None,
                crc32: None,
            })
            .collect();

        Ok(Self {
            file,
            entries,
            files,
            cluster_size,
            data_offset,
            first_cluster_of_manifest,
            next_cluster,
        })
    }
}

impl Archive for GcfArchive {
    fn format_name(&self) -> &'static str {
        "GCF"
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
        let mut cluster = self.first_cluster_of_manifest[file.manifest_index as usize];
        let mut out = Vec::with_capacity(file.size as usize);
        let mut remaining = file.size as usize;
        while cluster != 0xFFFF_FFFF && remaining > 0 {
            let take = remaining.min(self.cluster_size as usize);
            let off = self.data_offset + cluster as u64 * self.cluster_size as u64;
            self.file.seek(SeekFrom::Start(off))?;
            let mut buf = vec![0u8; take];
            self.file.read_exact(&mut buf)?;
            out.extend_from_slice(&buf);
            remaining -= take;
            let next = *self
                .next_cluster
                .get(cluster as usize)
                .unwrap_or(&0xFFFF_FFFF);
            cluster = next;
        }
        Ok(out)
    }
}

fn read_name(table: &[u8], offset: u32) -> String {
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
