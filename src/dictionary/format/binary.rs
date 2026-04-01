//! Compact binary serialization for pronunciation dictionaries.
//!
//! Uses [`postcard`] for a no_std-friendly, compact binary format that is
//! significantly faster to load and smaller than JSON or text formats.
//!
//! # Format
//!
//! The binary format uses a clean intermediate representation (no backward-compat
//! shims) with a 4-byte magic number and 1-byte version header.
//!
//! # Examples
//!
//! ```rust
//! # #[cfg(feature = "binary")]
//! # {
//! use shabdakosh::PronunciationDict;
//! use shabdakosh::dictionary::format::binary;
//!
//! let dict = PronunciationDict::english_minimal();
//! let bytes = binary::to_binary(&dict).unwrap();
//! let dict2 = binary::from_binary(&bytes).unwrap();
//! assert_eq!(dict.len(), dict2.len());
//! assert_eq!(dict.lookup("hello"), dict2.lookup("hello"));
//! # }
//! ```

use alloc::{collections::BTreeMap, string::String, vec::Vec};
use serde::{Deserialize, Serialize};

use crate::PronunciationDict;
use crate::dictionary::entry::{DictEntry, Pronunciation, Region};
use crate::error::{Result, ShabdakoshError};

/// Magic bytes identifying a shabdakosh binary dictionary.
const MAGIC: [u8; 4] = *b"SHBD";

/// Current binary format version.
const VERSION: u8 = 1;

/// Header size: 4 bytes magic + 1 byte version.
const HEADER_SIZE: usize = 5;

/// Intermediate representation for binary serialization.
///
/// Avoids the `#[serde(untagged)]` backward-compat deserializers in
/// `PronunciationDict` which postcard doesn't support.
#[derive(Serialize, Deserialize)]
struct BinaryDict {
    entries: BTreeMap<String, Vec<BinaryPronunciation>>,
    user_entries: BTreeMap<String, Vec<BinaryPronunciation>>,
    language: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct BinaryPronunciation {
    phonemes: Vec<svara::phoneme::Phoneme>,
    frequency: Option<f32>,
    region: Option<Region>,
}

impl BinaryDict {
    fn from_dict(dict: &PronunciationDict) -> Self {
        Self {
            entries: convert_entries(dict.entries()),
            user_entries: dict
                .user_entries()
                .iter()
                .map(|(k, v)| (k.clone(), convert_entry(v)))
                .collect(),
            language: dict.language().map(alloc::string::ToString::to_string),
        }
    }

    fn into_dict(self) -> PronunciationDict {
        let mut dict = PronunciationDict::new();
        if let Some(lang) = &self.language {
            dict.set_language(lang);
        }

        for (word, prons) in self.entries {
            if let Some(entry) = to_dict_entry(prons) {
                dict.insert_entry(&word, entry);
            }
        }

        for (word, prons) in self.user_entries {
            if let Some(entry) = to_dict_entry(prons) {
                dict.insert_user_entry(&word, entry);
            }
        }

        dict
    }
}

fn convert_entries(
    entries: &hashbrown::HashMap<String, DictEntry>,
) -> BTreeMap<String, Vec<BinaryPronunciation>> {
    entries
        .iter()
        .map(|(k, v)| (k.clone(), convert_entry(v)))
        .collect()
}

fn convert_entry(entry: &DictEntry) -> Vec<BinaryPronunciation> {
    entry
        .all()
        .iter()
        .map(|p| BinaryPronunciation {
            phonemes: p.phonemes().to_vec(),
            frequency: p.frequency(),
            region: p.region(),
        })
        .collect()
}

fn to_dict_entry(prons: Vec<BinaryPronunciation>) -> Option<DictEntry> {
    let pronunciations: Vec<Pronunciation> = prons
        .into_iter()
        .map(|bp| {
            let mut p = Pronunciation::new(bp.phonemes);
            if let Some(f) = bp.frequency {
                p = p.with_frequency(f);
            }
            if let Some(r) = bp.region {
                p = p.with_region(r);
            }
            p
        })
        .collect();
    DictEntry::from_pronunciations(pronunciations)
}

/// Serializes a [`PronunciationDict`] to compact binary format.
///
/// # Errors
///
/// Returns [`ShabdakoshError::DictParseError`] if serialization fails.
#[must_use = "serialization result should be used"]
pub fn to_binary(dict: &PronunciationDict) -> Result<Vec<u8>> {
    let intermediate = BinaryDict::from_dict(dict);
    let payload = postcard::to_allocvec(&intermediate).map_err(|e| {
        ShabdakoshError::DictParseError(alloc::format!("binary serialize error: {e}"))
    })?;

    let mut out = Vec::with_capacity(HEADER_SIZE + payload.len());
    out.extend_from_slice(&MAGIC);
    out.push(VERSION);
    out.extend_from_slice(&payload);
    Ok(out)
}

/// Deserializes a [`PronunciationDict`] from compact binary format.
///
/// # Errors
///
/// Returns [`ShabdakoshError::DictParseError`] if the magic number, version,
/// or payload is invalid.
#[must_use = "deserialization result should be used"]
pub fn from_binary(data: &[u8]) -> Result<PronunciationDict> {
    if data.len() < HEADER_SIZE {
        return Err(ShabdakoshError::DictParseError(
            "binary data too short for header".into(),
        ));
    }

    if data[..4] != MAGIC {
        return Err(ShabdakoshError::DictParseError(
            "invalid magic number: not a shabdakosh binary dictionary".into(),
        ));
    }

    if data[4] != VERSION {
        return Err(ShabdakoshError::DictParseError(alloc::format!(
            "unsupported binary format version: {} (expected {VERSION})",
            data[4]
        )));
    }

    let intermediate: BinaryDict = postcard::from_bytes(&data[HEADER_SIZE..]).map_err(|e| {
        ShabdakoshError::DictParseError(alloc::format!("binary deserialize error: {e}"))
    })?;
    Ok(intermediate.into_dict())
}

/// Saves a [`PronunciationDict`] to a binary file.
///
/// # Errors
///
/// Returns [`ShabdakoshError::DictParseError`] on serialization or I/O failure.
#[cfg(feature = "std")]
pub fn save_binary_file(dict: &PronunciationDict, path: &std::path::Path) -> Result<()> {
    let data = to_binary(dict)?;
    std::fs::write(path, data).map_err(|e| {
        ShabdakoshError::DictParseError(alloc::format!("failed to write binary file: {e}"))
    })
}

/// Loads a [`PronunciationDict`] from a binary file.
///
/// # Errors
///
/// Returns [`ShabdakoshError::DictParseError`] on I/O or deserialization failure.
#[cfg(feature = "std")]
pub fn load_binary_file(path: &std::path::Path) -> Result<PronunciationDict> {
    let data = std::fs::read(path).map_err(|e| {
        ShabdakoshError::DictParseError(alloc::format!("failed to read binary file: {e}"))
    })?;
    from_binary(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_roundtrip_minimal() {
        let dict = PronunciationDict::english_minimal();
        let bytes = to_binary(&dict).unwrap();
        let dict2 = from_binary(&bytes).unwrap();

        assert_eq!(dict.len(), dict2.len());
        assert_eq!(dict.language(), dict2.language());
        assert_eq!(dict.lookup("hello"), dict2.lookup("hello"));
        assert_eq!(dict.lookup("the"), dict2.lookup("the"));
    }

    #[test]
    fn test_binary_roundtrip_with_user_overlay() {
        let mut dict = PronunciationDict::english_minimal();
        dict.insert_user("custom", &[svara::phoneme::Phoneme::VowelA]);

        let bytes = to_binary(&dict).unwrap();
        let dict2 = from_binary(&bytes).unwrap();

        assert_eq!(dict2.user_len(), 1);
        assert_eq!(dict2.lookup("custom"), dict.lookup("custom"));
    }

    #[test]
    fn test_binary_roundtrip_empty() {
        let dict = PronunciationDict::new();
        let bytes = to_binary(&dict).unwrap();
        let dict2 = from_binary(&bytes).unwrap();
        assert!(dict2.is_empty());
    }

    #[test]
    fn test_binary_has_header() {
        let dict = PronunciationDict::new();
        let bytes = to_binary(&dict).unwrap();
        assert!(bytes.len() >= HEADER_SIZE);
        assert_eq!(&bytes[..4], b"SHBD");
        assert_eq!(bytes[4], VERSION);
    }

    #[test]
    fn test_binary_reject_short_data() {
        let result = from_binary(&[0, 1, 2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_reject_bad_magic() {
        let result = from_binary(&[0, 0, 0, 0, 1]);
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_reject_bad_version() {
        let mut bytes = to_binary(&PronunciationDict::new()).unwrap();
        bytes[4] = 99; // bad version
        let result = from_binary(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_smaller_than_json() {
        let dict = PronunciationDict::english_minimal();
        let binary = to_binary(&dict).unwrap();
        let json = serde_json::to_string(&dict).unwrap();
        assert!(
            binary.len() < json.len(),
            "binary ({}) should be smaller than JSON ({})",
            binary.len(),
            json.len()
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_binary_file_roundtrip() {
        let dict = PronunciationDict::english_minimal();
        let tmp = std::env::temp_dir().join("shabdakosh_test_binary.bin");
        save_binary_file(&dict, &tmp).unwrap();
        let dict2 = load_binary_file(&tmp).unwrap();
        assert_eq!(dict.len(), dict2.len());
        assert_eq!(dict.lookup("hello"), dict2.lookup("hello"));
        let _ = std::fs::remove_file(&tmp);
    }
}
