//! Heteronym disambiguation via POS tagger or contextual callbacks.
//!
//! Heteronyms like "read" (past vs. present), "live" (verb vs. adjective),
//! and "wind" (air vs. coil) have multiple pronunciations. This module
//! provides a [`HeteronymResolver`] trait that lets POS taggers or other
//! context-aware systems select the correct pronunciation.
//!
//! # Examples
//!
//! ```rust
//! use shabdakosh::dictionary::heteronym::{HeteronymContext, HeteronymResolver};
//! use shabdakosh::PronunciationDict;
//!
//! /// A resolver that always picks the first (primary) pronunciation.
//! struct PrimaryResolver;
//!
//! impl HeteronymResolver for PrimaryResolver {
//!     fn select_variant(&self, _word: &str, _context: &HeteronymContext) -> Option<usize> {
//!         Some(0)
//!     }
//! }
//!
//! let dict = PronunciationDict::english();
//! let resolver = PrimaryResolver;
//! let context = HeteronymContext::new(&["I", "read", "books"], 1);
//!
//! let phonemes = dict.lookup_with_context("read", &resolver, &context);
//! assert!(phonemes.is_some());
//! ```

use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};
use svara::phoneme::Phoneme;

use super::PronunciationDict;

/// Context provided to a heteronym resolver for disambiguation.
///
/// Contains the surrounding words and the position of the target word.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeteronymContext {
    /// The surrounding words (the full sentence or phrase).
    pub words: Vec<String>,
    /// Index of the target word within `words`.
    pub position: usize,
}

impl HeteronymContext {
    /// Creates a new context from a word sequence and target position.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if `position >= words.len()`.
    #[must_use]
    pub fn new(words: &[&str], position: usize) -> Self {
        debug_assert!(
            position < words.len(),
            "position {position} out of bounds for {} words",
            words.len()
        );
        Self {
            words: words
                .iter()
                .map(alloc::string::ToString::to_string)
                .collect(),
            position,
        }
    }

    /// Returns the words before the target word.
    #[must_use]
    pub fn preceding_words(&self) -> &[String] {
        &self.words[..self.position]
    }

    /// Returns the words after the target word.
    #[must_use]
    pub fn following_words(&self) -> &[String] {
        if self.position + 1 < self.words.len() {
            &self.words[self.position + 1..]
        } else {
            &[]
        }
    }

    /// Returns the target word.
    #[must_use]
    pub fn target_word(&self) -> &str {
        &self.words[self.position]
    }
}

/// Trait for selecting among variant pronunciations of heteronyms.
///
/// Implementors typically use POS tagging, syntactic context, or
/// application-specific rules to choose the correct pronunciation variant.
pub trait HeteronymResolver: Send + Sync {
    /// Selects a pronunciation variant index for the given word in context.
    ///
    /// Returns `Some(index)` to select a specific variant from the entry's
    /// pronunciation list, or `None` to fall back to the primary pronunciation.
    fn select_variant(&self, word: &str, context: &HeteronymContext) -> Option<usize>;
}

impl PronunciationDict {
    /// Looks up a word's pronunciation using a heteronym resolver for disambiguation.
    ///
    /// If the word has multiple pronunciations and the resolver selects a variant,
    /// that variant's phonemes are returned. Otherwise, the primary pronunciation
    /// is used.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use shabdakosh::dictionary::heteronym::{HeteronymContext, HeteronymResolver};
    /// use shabdakosh::PronunciationDict;
    ///
    /// struct AlwaysSecond;
    /// impl HeteronymResolver for AlwaysSecond {
    ///     fn select_variant(&self, _word: &str, _ctx: &HeteronymContext) -> Option<usize> {
    ///         Some(1)  // Always pick the second variant
    ///     }
    /// }
    ///
    /// let dict = PronunciationDict::english();
    /// let ctx = HeteronymContext::new(&["I", "read", "books"], 1);
    /// let phonemes = dict.lookup_with_context("read", &AlwaysSecond, &ctx);
    /// assert!(phonemes.is_some());
    /// ```
    #[must_use]
    pub fn lookup_with_context(
        &self,
        word: &str,
        resolver: &dyn HeteronymResolver,
        context: &HeteronymContext,
    ) -> Option<&[Phoneme]> {
        let entry = self.lookup_entry(word)?;

        // Only consult the resolver if there are multiple variants.
        if entry.len() > 1
            && let Some(idx) = resolver.select_variant(word, context)
            && idx < entry.len()
        {
            return Some(entry.all()[idx].phonemes());
        }

        Some(entry.primary_phonemes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PrimaryResolver;
    impl HeteronymResolver for PrimaryResolver {
        fn select_variant(&self, _word: &str, _ctx: &HeteronymContext) -> Option<usize> {
            Some(0)
        }
    }

    struct SecondVariantResolver;
    impl HeteronymResolver for SecondVariantResolver {
        fn select_variant(&self, _word: &str, _ctx: &HeteronymContext) -> Option<usize> {
            Some(1)
        }
    }

    struct NoneResolver;
    impl HeteronymResolver for NoneResolver {
        fn select_variant(&self, _word: &str, _ctx: &HeteronymContext) -> Option<usize> {
            None
        }
    }

    #[test]
    fn test_context_new() {
        let ctx = HeteronymContext::new(&["I", "read", "books"], 1);
        assert_eq!(ctx.target_word(), "read");
        assert_eq!(ctx.preceding_words(), &["I".to_string()]);
        assert_eq!(ctx.following_words(), &["books".to_string()]);
    }

    #[test]
    fn test_context_first_word() {
        let ctx = HeteronymContext::new(&["read", "this"], 0);
        assert!(ctx.preceding_words().is_empty());
        assert_eq!(ctx.following_words(), &["this".to_string()]);
    }

    #[test]
    fn test_context_last_word() {
        let ctx = HeteronymContext::new(&["I", "read"], 1);
        assert_eq!(ctx.preceding_words(), &["I".to_string()]);
        assert!(ctx.following_words().is_empty());
    }

    #[test]
    fn test_lookup_with_context_single_pronunciation() {
        let dict = PronunciationDict::english_minimal();
        let ctx = HeteronymContext::new(&["the"], 0);
        // "the" has one pronunciation — resolver is irrelevant.
        let result = dict.lookup_with_context("the", &SecondVariantResolver, &ctx);
        assert!(result.is_some());
        assert_eq!(result, dict.lookup("the"));
    }

    #[test]
    fn test_lookup_with_context_heteronym() {
        let dict = PronunciationDict::english();
        let ctx = HeteronymContext::new(&["I", "read", "books"], 1);

        // "read" has multiple pronunciations.
        let entry = dict.lookup_entry("read").unwrap();
        assert!(entry.len() >= 2);

        // Primary resolver picks first variant.
        let primary = dict.lookup_with_context("read", &PrimaryResolver, &ctx);
        assert_eq!(primary, Some(entry.all()[0].phonemes()));

        // Second variant resolver picks second.
        let second = dict.lookup_with_context("read", &SecondVariantResolver, &ctx);
        assert_eq!(second, Some(entry.all()[1].phonemes()));
    }

    #[test]
    fn test_lookup_with_context_none_resolver_returns_primary() {
        let dict = PronunciationDict::english();
        let ctx = HeteronymContext::new(&["I", "read", "books"], 1);
        let result = dict.lookup_with_context("read", &NoneResolver, &ctx);
        assert_eq!(result, dict.lookup("read"));
    }

    #[test]
    fn test_lookup_with_context_missing_word() {
        let dict = PronunciationDict::english_minimal();
        let ctx = HeteronymContext::new(&["xyzzy"], 0);
        let result = dict.lookup_with_context("xyzzy", &PrimaryResolver, &ctx);
        assert!(result.is_none());
    }

    #[test]
    fn test_lookup_with_context_out_of_bounds_index() {
        let dict = PronunciationDict::english();
        let ctx = HeteronymContext::new(&["read"], 0);

        // Resolver returns an out-of-bounds index — should fall back to primary.
        struct BadResolver;
        impl HeteronymResolver for BadResolver {
            fn select_variant(&self, _word: &str, _ctx: &HeteronymContext) -> Option<usize> {
                Some(999)
            }
        }

        let result = dict.lookup_with_context("read", &BadResolver, &ctx);
        assert_eq!(result, dict.lookup("read"));
    }

    #[test]
    fn test_context_serde_roundtrip() {
        let ctx = HeteronymContext::new(&["I", "read", "books"], 1);
        let json = serde_json::to_string(&ctx).unwrap();
        let ctx2: HeteronymContext = serde_json::from_str(&json).unwrap();
        assert_eq!(ctx, ctx2);
    }
}
