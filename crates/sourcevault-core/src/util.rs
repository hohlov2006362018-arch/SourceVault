//! Internal helpers used by the format readers.

use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::error::{Error, Result};

/// Reads a NUL-terminated ASCII string from the current position.
pub(crate) fn read_cstring<R: Read>(r: &mut R) -> Result<String> {
    let mut buf = Vec::with_capacity(64);
    loop {
        let b = r.read_u8()?;
        if b == 0 {
            break;
        }
        buf.push(b);
    }
    Ok(String::from_utf8(buf)?)
}

/// Reads a fixed-length, NUL-padded ASCII string.
pub(crate) fn read_fixed_string<R: Read>(r: &mut R, len: usize) -> Result<String> {
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    let nul = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    buf.truncate(nul);
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

/// Reads a UTF-16LE NUL-terminated string of at most `max_chars` characters.
pub(crate) fn read_utf16_string<R: Read>(r: &mut R, max_chars: usize) -> Result<String> {
    let mut units = Vec::with_capacity(max_chars);
    for _ in 0..max_chars {
        let u = r.read_u16::<LittleEndian>()?;
        if u == 0 {
            break;
        }
        units.push(u);
    }
    Ok(String::from_utf16_lossy(&units))
}

/// Normalises a path inside an archive to use forward slashes and strip leading `./` / `/`.
pub(crate) fn normalise_path(p: &str) -> String {
    let trimmed = p.trim_start_matches('/').trim_start_matches("./");
    trimmed.replace('\\', "/")
}

/// Verifies that `actual` matches `expected` CRC, otherwise returns [`Error::CrcMismatch`].
pub(crate) fn check_crc(path: &str, expected: u32, actual: u32) -> Result<()> {
    if expected == 0 {
        return Ok(());
    }
    if expected != actual {
        return Err(Error::CrcMismatch {
            path: path.to_owned(),
            expected,
            actual,
        });
    }
    Ok(())
}
