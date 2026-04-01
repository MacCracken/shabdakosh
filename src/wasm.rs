//! WebAssembly bindings via `wasm-bindgen`.
//!
//! Provides a JavaScript-friendly API for dictionary lookup in the browser.
//! Complex types (phoneme arrays) are returned as JSON strings.
//!
//! # Usage (JavaScript)
//!
//! ```javascript
//! import init, { WasmDict } from './shabdakosh_bg.wasm';
//!
//! await init();
//! const dict = WasmDict.english();
//! const phonemes = dict.lookup("hello"); // JSON string or null
//! console.log(JSON.parse(phonemes));
//! dict.free();
//! ```

use wasm_bindgen::prelude::*;

use crate::PronunciationDict;

/// A pronunciation dictionary for use from JavaScript.
///
/// Wraps [`PronunciationDict`] with a wasm-bindgen compatible API.
#[wasm_bindgen]
pub struct WasmDict {
    inner: PronunciationDict,
}

#[wasm_bindgen]
impl WasmDict {
    /// Creates a new empty dictionary.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: PronunciationDict::new(),
        }
    }

    /// Creates the built-in English dictionary (10,000+ entries).
    pub fn english() -> Self {
        Self {
            inner: PronunciationDict::english(),
        }
    }

    /// Creates the minimal English dictionary (~29 entries).
    pub fn english_minimal() -> Self {
        Self {
            inner: PronunciationDict::english_minimal(),
        }
    }

    /// Returns the number of base entries.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns whether the dictionary is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Looks up a word and returns its phonemes as a JSON array of IPA strings.
    ///
    /// Returns `null` if the word is not found.
    pub fn lookup(&self, word: &str) -> Option<String> {
        let phonemes = self.inner.lookup(word)?;
        let ipa_strings: Vec<&str> = phonemes
            .iter()
            .filter_map(crate::ipa::phoneme_to_ipa)
            .collect();
        serde_json::to_string(&ipa_strings).ok()
    }

    /// Looks up a word and returns the number of pronunciation variants.
    ///
    /// Returns 0 if the word is not found.
    pub fn variant_count(&self, word: &str) -> usize {
        self.inner.lookup_entry(word).map_or(0, |entry| entry.len())
    }

    /// Inserts a word into the user overlay with IPA pronunciation.
    ///
    /// The IPA string is parsed into phonemes.
    pub fn insert_user_ipa(&mut self, word: &str, ipa: &str) {
        let phonemes = crate::ipa::parse_ipa_word(ipa);
        if !phonemes.is_empty() {
            self.inner.insert_user(word, &phonemes);
        }
    }

    /// Removes a word from the user overlay.
    ///
    /// Returns `true` if the word was removed.
    pub fn remove_user(&mut self, word: &str) -> bool {
        self.inner.remove_user(word)
    }

    /// Returns the number of user overlay entries.
    pub fn user_len(&self) -> usize {
        self.inner.user_len()
    }

    /// Returns words matching a prefix as a JSON array.
    pub fn prefix_search(&self, prefix: &str) -> String {
        let results = self.inner.prefix_search(prefix);
        serde_json::to_string(&results).unwrap_or_else(|_| "[]".to_string())
    }

    /// Returns coverage analysis of a text as JSON.
    pub fn coverage(&self, text: &str) -> String {
        let report = self.inner.coverage(text);
        serde_json::to_string(&report).unwrap_or_else(|_| "{}".to_string())
    }
}

impl Default for WasmDict {
    fn default() -> Self {
        Self::new()
    }
}

// Note: wasm-bindgen tests require wasm-pack and a browser/Node.js runtime.
// Unit tests for the underlying PronunciationDict cover the core logic.
// WASM-specific tests should be run via `wasm-pack test --node`.
