//! Binary encoding of the profile report.
//!
//! A report is a magic + version header followed by fixed-width,
//! little-endian records, in the following format:
//!
//! ```text
//! magic   : b"JPRF"   (4 bytes)
//! version : u8        (== VERSION)
//! records : u32       (number of records that follow)
//! record[] :          (16 bytes each, ordered by (func_addr, target))
//!   func_addr : u32
//!   target    : u32
//!   count     : u64
//! ```

use anyhow::{Result, ensure};

const MAGIC: &[u8; 4] = b"JPRF";
const VERSION: u8 = 1;
const HEADER_LEN: usize = MAGIC.len() + size_of::<u8>() + size_of::<u32>();
const RECORD_LEN: usize = size_of::<u32>() + size_of::<u32>() + size_of::<u64>();

/// A single `(func_addr, target) -> count` report entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    /// QuickJS bytecode buffer start address, identifying the JS function.
    pub func_addr: u32,
    /// The `br_table` target, i.e. the QuickJS opcode.
    pub target: u32,
    /// Total countable Wasm instructions attributed to that opcode.
    pub count: u64,
}

/// Serialize `records` into the binary report format.
pub fn write<I>(records: I) -> Vec<u8>
where
    I: ExactSizeIterator<Item = Record>,
{
    let mut out = Vec::with_capacity(HEADER_LEN + records.len() * RECORD_LEN);
    out.extend_from_slice(MAGIC);
    out.push(VERSION);
    out.extend_from_slice(&(records.len() as u32).to_le_bytes());
    for r in records {
        out.extend_from_slice(&r.func_addr.to_le_bytes());
        out.extend_from_slice(&r.target.to_le_bytes());
        out.extend_from_slice(&r.count.to_le_bytes());
    }
    out
}

/// Parse a binary report, validating the magic and version.
pub fn read(bytes: &[u8]) -> Result<Vec<Record>> {
    ensure!(bytes.len() >= HEADER_LEN, "report shorter than header");
    ensure!(&bytes[..MAGIC.len()] == MAGIC, "bad report magic");

    let version = bytes[MAGIC.len()];
    ensure!(version == VERSION, "unsupported report version {version}");

    let count_off = MAGIC.len() + size_of::<u8>();
    let count =
        u32::from_le_bytes(bytes[count_off..count_off + size_of::<u32>()].try_into()?) as usize;

    let body = &bytes[HEADER_LEN..];
    ensure!(
        body.len() == count * RECORD_LEN,
        "report body is {} bytes, expected {} for {count} records",
        body.len(),
        count * RECORD_LEN
    );

    let records = body
        .chunks_exact(RECORD_LEN)
        .map(|c| Record {
            func_addr: u32::from_le_bytes(c[0..4].try_into().unwrap()),
            target: u32::from_le_bytes(c[4..8].try_into().unwrap()),
            count: u64::from_le_bytes(c[8..16].try_into().unwrap()),
        })
        .collect();

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips() {
        let records = vec![
            Record {
                func_addr: 0x1000,
                target: 5,
                count: 3,
            },
            Record {
                func_addr: 0x1000,
                target: 7,
                count: 5,
            },
        ];
        let bytes = write(records.clone().into_iter());
        assert_eq!(read(&bytes).unwrap(), records);
    }

    #[test]
    fn empty_roundtrips() {
        let bytes = write(std::iter::empty());
        assert!(read(&bytes).unwrap().is_empty());
    }

    #[test]
    fn rejects_bad_magic() {
        let mut bytes = write(std::iter::empty());
        bytes[0] = b'X';
        assert!(read(&bytes).is_err());
    }

    #[test]
    fn rejects_truncated_body() {
        let bytes = write(std::iter::once(Record {
            func_addr: 1,
            target: 2,
            count: 3,
        }));
        assert!(read(&bytes[..bytes.len() - 1]).is_err());
    }
}
