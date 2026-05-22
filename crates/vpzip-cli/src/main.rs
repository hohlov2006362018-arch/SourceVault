//! VPZip command-line interface.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use vpzip_core::{formats, Archive, ArchiveEntry, EntryKind, Format};

#[derive(Parser)]
#[command(
    name = "vpzip",
    author,
    version,
    about = "Free, native Rust archiver for Valve Source engine formats.",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Print the table of contents of an archive.
    List {
        /// Path to the archive (.vpk, .pak, .gcf, .sga, .wad, .xzp).
        archive: PathBuf,

        /// Show entry sizes and CRC values.
        #[arg(short = 'l', long = "long")]
        long: bool,
    },

    /// Print general info about an archive (format, version, entry count, total size).
    Info { archive: PathBuf },

    /// Extract one or more entries from an archive.
    Extract {
        /// Path to the archive.
        archive: PathBuf,

        /// Destination directory (defaults to a folder beside the archive).
        #[arg(short = 'o', long = "out")]
        out: Option<PathBuf>,

        /// Only extract entries whose path starts with this prefix.
        #[arg(short = 'p', long = "prefix")]
        prefix: Option<String>,
    },

    /// List supported formats and their file extensions.
    Formats,

    /// Create a new VPK v1 archive from a directory.
    Pack {
        /// Source directory whose contents become the archive.
        source: PathBuf,

        /// Output archive path (.vpk).
        #[arg(short = 'o', long = "out")]
        output: PathBuf,
    },
}

fn main() -> ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.cmd {
        Cmd::List { archive, long } => cmd_list(&archive, long),
        Cmd::Info { archive } => cmd_info(&archive),
        Cmd::Extract {
            archive,
            out,
            prefix,
        } => cmd_extract(&archive, out.as_deref(), prefix.as_deref()),
        Cmd::Formats => {
            for f in Format::all() {
                let exts: Vec<String> = f.extensions().iter().map(|e| format!(".{e}")).collect();
                println!("  {:<4}  {}", f.name(), exts.join(", "));
            }
            Ok(())
        }
        Cmd::Pack { source, output } => cmd_pack(&source, &output),
    }
}

fn cmd_list(path: &Path, long: bool) -> Result<()> {
    let archive = formats::open(path).with_context(|| format!("opening {}", path.display()))?;
    let entries = archive.entries();
    for e in entries {
        if long {
            let crc = e
                .crc32
                .map(|c| format!("0x{c:08X}"))
                .unwrap_or_else(|| "-".to_string());
            println!("{:>12}  {:>10}  {}", e.size, crc, e.path);
        } else {
            println!("{}", e.path);
        }
    }
    Ok(())
}

fn cmd_info(path: &Path) -> Result<()> {
    let archive = formats::open(path).with_context(|| format!("opening {}", path.display()))?;
    let entries: &[ArchiveEntry] = archive.entries();
    let files: usize = entries.iter().filter(|e| e.kind == EntryKind::File).count();
    let total: u64 = entries.iter().map(|e| e.size).sum();
    let meta = std::fs::metadata(path)?;
    println!("path           : {}", path.display());
    println!("format         : {}", archive.format_name());
    println!("file size      : {} bytes", meta.len());
    println!("entries        : {}", entries.len());
    println!("file entries   : {}", files);
    println!("uncompressed   : {} bytes", total);
    Ok(())
}

fn cmd_extract(path: &Path, out: Option<&Path>, prefix: Option<&str>) -> Result<()> {
    let mut archive = formats::open(path).with_context(|| format!("opening {}", path.display()))?;
    let out_owned: PathBuf = match out {
        Some(p) => p.to_path_buf(),
        None => {
            let stem = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "extracted".to_string());
            path.parent().unwrap_or_else(|| Path::new(".")).join(stem)
        }
    };
    std::fs::create_dir_all(&out_owned)?;

    let target_paths: Vec<String> = archive
        .entries()
        .iter()
        .filter(|e| e.kind == EntryKind::File)
        .filter(|e| match prefix {
            Some(p) => e.path.starts_with(p),
            None => true,
        })
        .map(|e| e.path.clone())
        .collect();

    let pb = ProgressBar::new(target_paths.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} [{wide_bar}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=> "),
    );

    for p in &target_paths {
        pb.set_message(p.clone());
        let mut dest = out_owned.clone();
        for part in p.split('/') {
            dest.push(part);
        }
        archive.extract_entry(p, &dest)?;
        pb.inc(1);
    }
    pb.finish_with_message(format!("extracted to {}", out_owned.display()));
    Ok(())
}

fn cmd_pack(source: &Path, output: &Path) -> Result<()> {
    if !source.is_dir() {
        anyhow::bail!("source `{}` is not a directory", source.display());
    }
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    walk(source, source, &mut entries)?;

    let file = std::fs::File::create(output)?;
    let mut writer = std::io::BufWriter::new(file);
    vpzip_core::formats::vpk::write_v1(&mut writer, entries)?;
    println!("wrote {}", output.display());
    Ok(())
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk(root, &path, out)?;
        } else if path.is_file() {
            let rel = path.strip_prefix(root)?;
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            let data = std::fs::read(&path)?;
            out.push((rel_str, data));
        }
    }
    Ok(())
}
