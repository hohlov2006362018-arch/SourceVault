//! End-to-end tests for the `sourcevault` CLI binary.

use std::io::Cursor;
use std::path::PathBuf;
use std::process::Command;

fn binary() -> PathBuf {
    let exe = env!("CARGO_BIN_EXE_sourcevault");
    PathBuf::from(exe)
}

fn make_pak(path: &std::path::Path) {
    let entries = vec![
        ("readme.txt".to_string(), b"hello quake\n".to_vec()),
        (
            "maps/start.bsp".to_string(),
            (0..256u32).map(|i| i as u8).collect(),
        ),
    ];
    let file = std::fs::File::create(path).unwrap();
    let mut writer = std::io::BufWriter::new(file);
    sourcevault_core::formats::pak::write(&mut writer, entries).unwrap();
}

fn make_vpk(path: &std::path::Path) {
    let mut buf = Vec::new();
    sourcevault_core::formats::vpk::write_v1(
        &mut Cursor::new(&mut buf),
        vec![("hello.txt".to_string(), b"world".to_vec())],
    )
    .unwrap();
    std::fs::write(path, buf).unwrap();
}

#[test]
fn list_and_extract_pak() {
    let tmp = tempdir();
    let pak = tmp.join("test.pak");
    make_pak(&pak);

    let out = Command::new(binary())
        .arg("list")
        .arg(&pak)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let listing = String::from_utf8_lossy(&out.stdout);
    assert!(listing.contains("readme.txt"));
    assert!(listing.contains("maps/start.bsp"));

    let extract_dir = tmp.join("extracted");
    let out = Command::new(binary())
        .args(["extract", "-o"])
        .arg(&extract_dir)
        .arg(&pak)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let extracted = std::fs::read(extract_dir.join("readme.txt")).unwrap();
    assert_eq!(extracted, b"hello quake\n");
}

#[test]
fn info_vpk() {
    let tmp = tempdir();
    let vpk = tmp.join("test.vpk");
    make_vpk(&vpk);

    let out = Command::new(binary())
        .arg("info")
        .arg(&vpk)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("VPK"));
    assert!(stdout.contains("entries        : 1"));
}

#[test]
fn formats_listing() {
    let out = Command::new(binary()).arg("formats").output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    for f in ["VPK", "PAK", "WAD", "XZP", "GCF", "SGA"] {
        assert!(stdout.contains(f), "missing format {f} in: {stdout}");
    }
}

fn tempdir() -> PathBuf {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("sourcevault-it-{id}"));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
