//! Reader and writer for Warcraft III import lists, `war3map.imp`: the
//! files World Editor's Import Manager added to a map, by path.
//!
//! ```no_run
//! let bytes = std::fs::read("war3map.imp").unwrap();
//! let list = imp_rs::parse(&bytes).unwrap();
//! for file in &list.files {
//!     println!("{}", file.path_in_map());
//! }
//! assert_eq!(list.to_bytes().unwrap(), bytes);
//! ```
//!
//! Reading and writing are exact. For people, a list has a text form
//! ([`text`]).

pub mod text;

use parser_rs::bin::{Reader, Writer};
pub use parser_rs::{BinError, BinResult};

/// Where World Editor puts an imported file that has no path of its own.
pub const IMPORTED: &str = "war3mapImported\\";

/// [`Import::flags`]: the path is the file's own (a custom path); without
/// it, the file is at [`IMPORTED`] and its name.
pub const CUSTOM_PATH: u8 = 0x01;

/// The flags' names, by bit; the file's other bits are kept as they
/// are. `0x02` and up aren't known well: one description has `0x02` drop
/// the entry when read.
pub const FLAGS: [(u8, &str); 5] = [
    (0x01, "custom path"),
    (0x02, "unknown 0x02"),
    (0x04, "unknown 0x04"),
    (0x08, "unknown 0x08"),
    (0x10, "unknown 0x10"),
];

/// One imported file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    /// Version 1 only; World Editor writes `5`, `10` and `13` too.
    pub flags: u8,
    /// As stored: a full path with [`CUSTOM_PATH`], else whatever World
    /// Editor kept (often the name alone).
    pub path: String,
}

impl Import {
    /// Where the file is in the map: the path itself with
    /// [`CUSTOM_PATH`] (and in version 0, which has no flags, as given),
    /// else [`IMPORTED`] and the path's file name.
    pub fn path_in_map(&self) -> String {
        if self.flags & CUSTOM_PATH != 0 || self.path.starts_with(IMPORTED) {
            return self.path.clone();
        }
        let name = self.path.rsplit(['\\', '/']).next().unwrap_or(&self.path);
        format!("{IMPORTED}{name}")
    }
}

/// A whole `war3map.imp`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Imports {
    /// 0 for Reign of Chaos (no flags), 1 from The Frozen Throne.
    pub version: u32,
    pub files: Vec<Import>,
}

impl Imports {
    /// The file's bytes.
    pub fn to_bytes(&self) -> BinResult<Vec<u8>> {
        if self.version > 1 {
            return Err(BinError::new(0, format!("unknown imp version {}", self.version)));
        }
        let mut w = Writer::new();
        w.u32(self.version);
        w.count(self.files.len())?;
        for f in &self.files {
            if self.version == 1 {
                w.u8(f.flags);
            }
            w.cstr(&f.path)?;
        }
        Ok(w.into_bytes())
    }
}

/// Reads a `war3map.imp`. Every byte must be read.
pub fn parse(bytes: &[u8]) -> BinResult<Imports> {
    let mut r = Reader::new(bytes);
    let version = r.u32()?;
    if version > 1 {
        return Err(BinError::new(0, format!("unknown imp version {version} (0 and 1 are known)")));
    }
    let min = if version == 1 { 2 } else { 1 };
    let files = r.list(min, |r| {
        let flags = if version == 1 { r.u8()? } else { 0 };
        Ok(Import { flags, path: r.cstr()? })
    })?;
    r.finish()?;
    Ok(Imports { version, files })
}

#[cfg(test)]
#[path = "lib.test.rs"]
mod tests;
