//! Binary encoding of the profile report.
//!
//! A report is a magic + version header followed by two tables of
//! fixed-width, little-endian rows, in the following format:
//!
//! ```text
//! magic    : b"JPRF"   (4 bytes)
//! version  : u8        (== VERSION)
//! entries  : u32       (number of entry rows that follow)
//! sites    : u32       (number of site rows after the entries)
//! entry[]  :           (12 bytes each, ordered by addr)
//!   addr   : u32
//!   count  : u64
//! site[]   :           (16 bytes each, ordered by (addr, opcode))
//!   addr   : u32
//!   opcode : u32
//!   count  : u64
//! ```
//!
//! Addresses are absolute, which means the report does not identify functions.
//! That is left to symbolization, which matches the sites against the module's
//! bytecode, using the entries as hints of where functions start.

use anyhow::{Result, ensure};

const MAGIC: &[u8; 4] = b"JPRF";
const VERSION: u8 = 1;
const HEADER_LEN: usize = MAGIC.len() + size_of::<u8>() + 2 * size_of::<u32>();
const ENTRY_LEN: usize = size_of::<u32>() + size_of::<u64>();
const SITE_LEN: usize = 2 * size_of::<u32>() + size_of::<u64>();

/// The address of the first dispatch of an interpreter activation, and
/// how many activations started there.
///
/// It is a hint, only. A normal call starts at byte 0 of its
/// function's bytecode, but a resumed generator or async function starts
/// at its suspension point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Address of the activation's first dispatch.
    pub addr: u32,
    /// Number of activations whose first dispatch was at `addr`.
    pub count: u64,
}

/// A bytecode site: an opcode read at an absolute address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Site {
    /// Address the dispatch load read the opcode from.
    pub addr: u32,
    /// The QuickJS opcode, i.e. the byte read by the dispatch load.
    pub opcode: u32,
    /// Total countable Wasm instructions attributed to the site.
    pub count: u64,
}

/// A decoded report.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// Activation entry hints, ordered by address.
    pub entries: Vec<Entry>,
    /// Bytecode sites, ordered by address then opcode.
    pub sites: Vec<Site>,
}

/// Serialize `entries` and `sites` into the binary report format.
pub fn write<E, S>(entries: E, sites: S) -> Vec<u8>
where
    E: ExactSizeIterator<Item = Entry>,
    S: ExactSizeIterator<Item = Site>,
{
    let mut out =
        Vec::with_capacity(HEADER_LEN + entries.len() * ENTRY_LEN + sites.len() * SITE_LEN);
    out.extend_from_slice(MAGIC);
    out.push(VERSION);
    out.extend_from_slice(&(entries.len() as u32).to_le_bytes());
    out.extend_from_slice(&(sites.len() as u32).to_le_bytes());
    for e in entries {
        out.extend_from_slice(&e.addr.to_le_bytes());
        out.extend_from_slice(&e.count.to_le_bytes());
    }
    for s in sites {
        out.extend_from_slice(&s.addr.to_le_bytes());
        out.extend_from_slice(&s.opcode.to_le_bytes());
        out.extend_from_slice(&s.count.to_le_bytes());
    }
    out
}

/// Parse a binary report, validating the magic and version.
pub fn read(bytes: &[u8]) -> Result<Report> {
    ensure!(bytes.len() >= HEADER_LEN, "report shorter than header");
    ensure!(&bytes[..MAGIC.len()] == MAGIC, "bad report magic");

    let version = bytes[MAGIC.len()];
    ensure!(version == VERSION, "unsupported report version {version}");

    let u32_at = |at: usize| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
    let num_entries = u32_at(MAGIC.len() + 1) as usize;
    let num_sites = u32_at(MAGIC.len() + 5) as usize;

    let body = &bytes[HEADER_LEN..];
    let entries_len = num_entries * ENTRY_LEN;
    ensure!(
        body.len() == entries_len + num_sites * SITE_LEN,
        "report body is {} bytes, expected {} for {num_entries} entries and {num_sites} sites",
        body.len(),
        entries_len + num_sites * SITE_LEN
    );

    let (entries, _) = body[..entries_len].as_chunks::<ENTRY_LEN>();
    let (sites, _) = body[entries_len..].as_chunks::<SITE_LEN>();

    Ok(Report {
        entries: entries
            .iter()
            .map(|c| Entry {
                addr: u32::from_le_bytes(c[0..4].try_into().unwrap()),
                count: u64::from_le_bytes(c[4..12].try_into().unwrap()),
            })
            .collect(),
        sites: sites
            .iter()
            .map(|c| Site {
                addr: u32::from_le_bytes(c[0..4].try_into().unwrap()),
                opcode: u32::from_le_bytes(c[4..8].try_into().unwrap()),
                count: u64::from_le_bytes(c[8..16].try_into().unwrap()),
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Report {
        Report {
            entries: vec![
                Entry {
                    addr: 0x1000,
                    count: 2,
                },
                Entry {
                    addr: 0x1005,
                    count: 1,
                },
            ],
            sites: vec![
                Site {
                    addr: 0x1000,
                    opcode: 5,
                    count: 3,
                },
                Site {
                    addr: 0x1005,
                    opcode: 7,
                    count: 5,
                },
            ],
        }
    }

    fn encode(report: &Report) -> Vec<u8> {
        write(
            report.entries.clone().into_iter(),
            report.sites.clone().into_iter(),
        )
    }

    #[test]
    fn roundtrips() {
        let report = sample();
        assert_eq!(read(&encode(&report)).unwrap(), report);
    }

    #[test]
    fn empty_roundtrips() {
        let bytes = write(std::iter::empty(), std::iter::empty());
        assert_eq!(read(&bytes).unwrap(), Report::default());
    }

    #[test]
    fn rejects_bad_magic() {
        let mut bytes = encode(&sample());
        bytes[0] = b'X';
        assert!(read(&bytes).is_err());
    }

    #[test]
    fn rejects_truncated_body() {
        let bytes = encode(&sample());
        assert!(read(&bytes[..bytes.len() - 1]).is_err());
    }
}
