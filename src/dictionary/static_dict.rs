//! Zero-allocation static dictionary backed by a compile-time perfect hash.
//!
//! When the `phf` feature is enabled, the English dictionary is available as
//! a static [`phf::Map`] with no runtime heap allocation. This is ideal for
//! embedded systems, WASM, and latency-sensitive applications.
//!
//! # Examples
//!
//! ```rust
//! # #[cfg(feature = "phf")]
//! # {
//! use shabdakosh::dictionary::static_dict;
//!
//! let phonemes = static_dict::lookup("hello");
//! assert!(phonemes.is_some());
//!
//! // No heap allocation — data is baked into the binary.
//! let count = static_dict::len();
//! assert!(count >= 10000);
//! # }
//! ```

use svara::phoneme::Phoneme;

use crate::dictionary::entry::Region;

/// A single static pronunciation (no heap allocation).
#[derive(Debug, Clone, Copy)]
pub struct StaticPronunciation {
    /// Phoneme sequence (static slice).
    pub phonemes: &'static [Phoneme],
    /// Relative frequency (0.0–1.0), if known.
    pub frequency: Option<f32>,
    /// Regional variant, if specified.
    pub region: Option<Region>,
}

/// A static dictionary entry containing one or more pronunciations.
#[derive(Debug, Clone, Copy)]
pub struct StaticEntry {
    /// All pronunciations, sorted by frequency descending.
    pub pronunciations: &'static [StaticPronunciation],
}

impl StaticEntry {
    /// Returns the primary (highest-frequency) pronunciation's phonemes.
    #[must_use]
    pub fn primary_phonemes(&self) -> &'static [Phoneme] {
        self.pronunciations[0].phonemes
    }

    /// Returns all pronunciations.
    #[must_use]
    pub fn all(&self) -> &'static [StaticPronunciation] {
        self.pronunciations
    }

    /// Returns the number of pronunciation variants.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pronunciations.len()
    }

    /// Always returns false (entries are guaranteed non-empty).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        false
    }
}

// Pull in the generated phf map from build.rs output.
#[cfg(feature = "phf")]
include!(concat!(env!("OUT_DIR"), "/generated_phf_dict.rs"));

/// Looks up the primary pronunciation of a word in the static dictionary.
///
/// Returns `None` if the word is not in the dictionary.
#[cfg(feature = "phf")]
#[must_use]
pub fn lookup(word: &str) -> Option<&'static [Phoneme]> {
    lookup_entry(&word.to_lowercase()).map(|e| e.primary_phonemes())
}

/// Looks up the full static entry for a word.
#[cfg(feature = "phf")]
#[must_use]
pub fn lookup_entry(word: &str) -> Option<&'static StaticEntry> {
    let key = word.to_lowercase();
    PHF_ENGLISH_DICT.get(key.as_str())
}

/// Returns the number of entries in the static dictionary.
#[cfg(feature = "phf")]
#[must_use]
pub fn len() -> usize {
    PHF_ENGLISH_DICT.len()
}

/// Returns whether the static dictionary is empty.
#[cfg(feature = "phf")]
#[must_use]
pub fn is_empty() -> bool {
    PHF_ENGLISH_DICT.is_empty()
}

#[cfg(all(test, feature = "phf"))]
mod tests {
    use super::*;

    #[test]
    fn test_static_lookup_hello() {
        let phonemes = lookup("hello");
        assert!(phonemes.is_some(), "hello should be in static dict");
        assert!(!phonemes.unwrap().is_empty());
    }

    #[test]
    fn test_static_lookup_the() {
        assert!(lookup("the").is_some());
    }

    #[test]
    fn test_static_lookup_case_insensitive() {
        assert_eq!(lookup("Hello"), lookup("hello"));
    }

    #[test]
    fn test_static_lookup_miss() {
        assert!(lookup("zxqvbnm").is_none());
    }

    #[test]
    fn test_static_len() {
        assert!(len() >= 10000);
    }

    #[test]
    fn test_static_entry_hello() {
        let entry = lookup_entry("hello").unwrap();
        assert!(!entry.is_empty());
        assert_eq!(entry.len(), 1); // hello has one pronunciation
    }

    #[test]
    fn test_static_entry_read_variants() {
        let entry = lookup_entry("read").unwrap();
        assert!(
            entry.len() >= 2,
            "read should have 2+ variants, got {}",
            entry.len()
        );
    }

    #[test]
    fn test_static_matches_dynamic() {
        // Verify that static and dynamic dictionaries produce the same results.
        let dynamic = crate::PronunciationDict::english();
        let words = ["hello", "the", "computer", "beautiful", "psychology"];
        for word in &words {
            let static_phonemes = lookup(word);
            let dynamic_phonemes = dynamic.lookup(word);
            assert_eq!(static_phonemes, dynamic_phonemes, "mismatch for '{word}'");
        }
    }
}
