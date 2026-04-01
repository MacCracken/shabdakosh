//! # shabdakosh — Pronunciation Dictionary
//!
//! **shabdakosh** (Sanskrit: dictionary) provides pronunciation dictionaries
//! mapping words to svara [`Phoneme`](svara::phoneme::Phoneme) sequences.
//!
//! ## Features
//!
//! - **10,000+ entry English dictionary** generated at compile time from CMUdict
//! - **ARPABET mapping** — bidirectional conversion between ARPABET and svara phonemes
//! - **User overlay** — application-specific entries that override the base dictionary
//! - **Import/export** — CMUdict text format and JSON (with `json` feature)
//!
//! ## Quick Start
//!
//! ```rust
//! use shabdakosh::PronunciationDict;
//!
//! let dict = PronunciationDict::english();
//! assert!(dict.lookup("hello").is_some());
//! assert!(dict.len() >= 10000);
//! ```
//!
//! ## Feature Flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `std` | Yes | Standard library. Disable for `no_std` + `alloc` |
//! | `json` | No | JSON import/export via serde_json |
//! | `varna` | No | Multi-language support: inventory validation, lexicon ingestion, script detection |

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod arpabet;
pub mod dictionary;
pub mod error;
#[cfg(feature = "ffi")]
pub mod ffi;
pub mod ipa;
#[cfg(feature = "wasm")]
pub mod wasm;

pub use dictionary::coverage::CoverageReport;
#[cfg(feature = "varna")]
pub use dictionary::detect::{detect_language_hint, detect_script, detect_script_name};
pub use dictionary::entry::{DictEntry, Pronunciation, Region};
pub use dictionary::g2p::{FallbackDict, G2PModel, G2PResult, LookupSource};
pub use dictionary::stream::LookupStream;
pub use dictionary::trie::PrefixTrie;
#[cfg(feature = "varna")]
pub use dictionary::validate::{
    InvalidEntry, PhonotacticReport, PhonotacticViolation, ValidationReport,
};
pub use dictionary::{DictDiff, PronunciationDict};
pub use error::{Result, ShabdakoshError};

// Compile-time trait assertions.
#[cfg(test)]
mod assert_traits {
    fn _assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn public_types_are_send_sync() {
        _assert_send_sync::<crate::error::ShabdakoshError>();
        _assert_send_sync::<crate::dictionary::PronunciationDict>();
        _assert_send_sync::<crate::dictionary::entry::DictEntry>();
        _assert_send_sync::<crate::dictionary::entry::Pronunciation>();
        _assert_send_sync::<crate::dictionary::entry::Region>();
        _assert_send_sync::<crate::dictionary::DictDiff>();
        _assert_send_sync::<crate::dictionary::g2p::G2PResult>();
        _assert_send_sync::<crate::dictionary::g2p::LookupSource>();
        _assert_send_sync::<crate::dictionary::g2p::FstModel>();
        _assert_send_sync::<crate::dictionary::g2p::FstNotation>();
        _assert_send_sync::<crate::dictionary::trie::PrefixTrie>();
        _assert_send_sync::<crate::dictionary::coverage::CoverageReport>();
        _assert_send_sync::<crate::dictionary::heteronym::HeteronymContext>();
    }

    #[cfg(feature = "varna")]
    #[test]
    fn varna_types_are_send_sync() {
        _assert_send_sync::<crate::dictionary::validate::ValidationReport>();
        _assert_send_sync::<crate::dictionary::validate::InvalidEntry>();
        _assert_send_sync::<crate::dictionary::validate::PhonotacticViolation>();
        _assert_send_sync::<crate::dictionary::validate::PhonotacticReport>();
    }
}
