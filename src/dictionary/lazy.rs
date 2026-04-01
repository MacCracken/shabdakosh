//! Lazy-loading dictionary backed by memory-mapped binary files.
//!
//! [`LazyDict`] maps a binary dictionary file into memory and deserializes
//! the full dictionary on open. This avoids reading the file into a heap
//! buffer — the OS pages in data as needed.
//!
//! # Prerequisites
//!
//! Requires the `mmap` feature (which implies `std` and `binary`).
//! Create binary files with [`format::binary::save_binary_file`](super::format::binary::save_binary_file).
//!
//! # Examples
//!
//! ```rust,no_run
//! # #[cfg(feature = "mmap")]
//! # {
//! use shabdakosh::dictionary::lazy::LazyDict;
//!
//! let dict = LazyDict::open("dictionary.bin").unwrap();
//! if let Some(phonemes) = dict.lookup("hello") {
//!     println!("hello => {:?}", phonemes);
//! }
//! # }
//! ```

use std::path::Path;

use svara::phoneme::Phoneme;

use super::PronunciationDict;
use super::entry::DictEntry;
use super::format::binary;
use crate::error::{Result, ShabdakoshError};

/// A dictionary backed by a memory-mapped binary file.
///
/// On open, the mmap'd data is deserialized into a full [`PronunciationDict`].
/// The mmap handle is kept alive for the lifetime of the `LazyDict` so the
/// OS can manage physical memory pressure (pages can be evicted and re-paged).
pub struct LazyDict {
    dict: PronunciationDict,
    // Keep the mmap alive so the OS can manage paging.
    _mmap: memmap2::Mmap,
}

impl LazyDict {
    /// Opens a binary dictionary file via memory mapping.
    ///
    /// # Errors
    ///
    /// Returns [`ShabdakoshError::DictParseError`] if the file cannot be opened,
    /// mapped, or deserialized.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = std::fs::File::open(path.as_ref())
            .map_err(|e| ShabdakoshError::DictParseError(format!("failed to open file: {e}")))?;

        // SAFETY: We rely on the file not being modified while mapped.
        // This is the standard contract for memory-mapped files.
        let mmap = unsafe {
            memmap2::MmapOptions::new()
                .map(&file)
                .map_err(|e| ShabdakoshError::DictParseError(format!("failed to mmap file: {e}")))?
        };

        let dict = binary::from_binary(&mmap)?;

        Ok(Self { dict, _mmap: mmap })
    }

    /// Looks up the primary pronunciation of a word.
    #[must_use]
    pub fn lookup(&self, word: &str) -> Option<&[Phoneme]> {
        self.dict.lookup(word)
    }

    /// Looks up the full dictionary entry for a word.
    #[must_use]
    pub fn lookup_entry(&self, word: &str) -> Option<&DictEntry> {
        self.dict.lookup_entry(word)
    }

    /// Returns the number of entries in the dictionary.
    #[must_use]
    pub fn len(&self) -> usize {
        self.dict.len()
    }

    /// Returns whether the dictionary is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.dict.is_empty()
    }

    /// Returns a reference to the underlying dictionary.
    #[must_use]
    pub fn dict(&self) -> &PronunciationDict {
        &self.dict
    }

    /// Consumes the lazy dict and returns the owned dictionary.
    ///
    /// The memory map is released.
    #[must_use]
    pub fn into_dict(self) -> PronunciationDict {
        self.dict
    }
}

impl core::fmt::Debug for LazyDict {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LazyDict")
            .field("entries", &self.dict.len())
            .field("user_entries", &self.dict.user_len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_dict_roundtrip() {
        let dict = PronunciationDict::english_minimal();
        let tmp = std::env::temp_dir().join("shabdakosh_test_lazy.bin");
        binary::save_binary_file(&dict, &tmp).unwrap();

        let lazy = LazyDict::open(&tmp).unwrap();
        assert_eq!(lazy.len(), dict.len());
        assert_eq!(lazy.lookup("hello"), dict.lookup("hello"));
        assert_eq!(lazy.lookup("the"), dict.lookup("the"));
        assert!(lazy.lookup("xyzzy").is_none());

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_lazy_dict_into_dict() {
        let dict = PronunciationDict::english_minimal();
        let tmp = std::env::temp_dir().join("shabdakosh_test_lazy_into.bin");
        binary::save_binary_file(&dict, &tmp).unwrap();

        let lazy = LazyDict::open(&tmp).unwrap();
        let recovered = lazy.into_dict();
        assert_eq!(recovered.len(), dict.len());

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_lazy_dict_open_nonexistent() {
        let result = LazyDict::open("/tmp/nonexistent_shabdakosh.bin");
        assert!(result.is_err());
    }

    #[test]
    fn test_lazy_dict_debug() {
        let dict = PronunciationDict::english_minimal();
        let tmp = std::env::temp_dir().join("shabdakosh_test_lazy_debug.bin");
        binary::save_binary_file(&dict, &tmp).unwrap();

        let lazy = LazyDict::open(&tmp).unwrap();
        let debug = format!("{lazy:?}");
        assert!(debug.contains("LazyDict"));

        let _ = std::fs::remove_file(&tmp);
    }
}
