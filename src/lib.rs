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

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod arpabet;
pub mod dictionary;
pub mod error;

pub use dictionary::entry::{DictEntry, Pronunciation, Region};
pub use dictionary::PronunciationDict;
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
    }
}
