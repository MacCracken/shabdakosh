//! Error types for the shabdakosh crate.

use alloc::string::String;
use serde::{Deserialize, Serialize};

/// Errors that can occur during dictionary operations.
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[non_exhaustive]
pub enum ShabdakoshError {
    /// Dictionary parsing or I/O failed.
    #[error("dictionary parse error: {0}")]
    DictParseError(String),

    /// An unrecognized ARPABET symbol was encountered.
    #[error("unknown ARPABET symbol: {0}")]
    UnknownSymbol(String),

    /// A phoneme is not in the target language's inventory.
    #[error("phoneme {phoneme} not in {language} inventory")]
    PhonemeNotInInventory {
        /// The IPA representation of the invalid phoneme.
        phoneme: String,
        /// The language code whose inventory was checked.
        language: String,
    },

    /// An unknown language code was provided.
    #[error("unknown language: {0}")]
    UnknownLanguage(String),
}

/// Convenience type alias for shabdakosh results.
pub type Result<T> = core::result::Result<T, ShabdakoshError>;
