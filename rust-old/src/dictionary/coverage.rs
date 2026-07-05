//! Text corpus coverage analysis.
//!
//! Given a text corpus, reports what percentage of tokens are covered by the
//! dictionary vs. falling through to rules or G2P. Identifies gaps.
//!
//! # Examples
//!
//! ```rust
//! use shabdakosh::PronunciationDict;
//! use shabdakosh::dictionary::coverage::CoverageReport;
//!
//! let dict = PronunciationDict::english_minimal();
//! let report = dict.coverage("the hello world is not in this");
//! assert!(report.coverage_pct() > 0.0);
//! assert!(report.total_tokens > 0);
//! ```

use alloc::{collections::BTreeSet, string::String, vec::Vec};
use serde::{Deserialize, Serialize};

use super::PronunciationDict;

/// Coverage analysis of a text corpus against a dictionary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CoverageReport {
    /// Total number of word tokens in the corpus.
    pub total_tokens: usize,
    /// Number of tokens found in the dictionary.
    pub covered_tokens: usize,
    /// Unique words not found in the dictionary (sorted).
    pub uncovered_words: Vec<String>,
}

impl CoverageReport {
    /// Returns the coverage percentage (0.0–100.0).
    ///
    /// Returns 0.0 if the corpus is empty.
    #[must_use]
    pub fn coverage_pct(&self) -> f32 {
        if self.total_tokens == 0 {
            return 0.0;
        }
        (self.covered_tokens as f32 / self.total_tokens as f32) * 100.0
    }

    /// Returns the number of unique uncovered words.
    #[must_use]
    pub fn uncovered_count(&self) -> usize {
        self.uncovered_words.len()
    }

    /// Returns `true` if every token was covered.
    #[must_use]
    pub fn is_fully_covered(&self) -> bool {
        self.uncovered_words.is_empty()
    }
}

/// Normalizes a token for dictionary lookup.
///
/// Strips leading/trailing punctuation and lowercases.
fn normalize_token(token: &str) -> String {
    token
        .trim_matches(|c: char| c.is_ascii_punctuation())
        .to_lowercase()
}

impl PronunciationDict {
    /// Analyzes how well this dictionary covers a text corpus.
    ///
    /// Tokenizes by whitespace, strips punctuation, and checks each token
    /// against the dictionary. Returns a [`CoverageReport`] with statistics.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use shabdakosh::PronunciationDict;
    ///
    /// let dict = PronunciationDict::english_minimal();
    /// let report = dict.coverage("Hello, world! The cat is here.");
    /// println!("coverage: {:.1}%", report.coverage_pct());
    /// println!("uncovered: {:?}", report.uncovered_words);
    /// ```
    #[must_use]
    pub fn coverage(&self, text: &str) -> CoverageReport {
        let mut total_tokens = 0_usize;
        let mut covered_tokens = 0_usize;
        let mut uncovered_set = BTreeSet::new();

        for token in text.split_whitespace() {
            let normalized = normalize_token(token);
            if normalized.is_empty() {
                continue;
            }
            total_tokens += 1;
            if self.lookup_entry(&normalized).is_some() {
                covered_tokens += 1;
            } else {
                uncovered_set.insert(normalized);
            }
        }

        CoverageReport {
            total_tokens,
            covered_tokens,
            uncovered_words: uncovered_set.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_full() {
        let dict = PronunciationDict::english_minimal();
        let report = dict.coverage("the hello world");
        assert_eq!(report.total_tokens, 3);
        assert_eq!(report.covered_tokens, 3);
        assert!(report.is_fully_covered());
        assert_eq!(report.coverage_pct(), 100.0);
    }

    #[test]
    fn test_coverage_partial() {
        let dict = PronunciationDict::english_minimal();
        let report = dict.coverage("the hello xyzzy");
        assert_eq!(report.total_tokens, 3);
        assert_eq!(report.covered_tokens, 2);
        assert!(!report.is_fully_covered());
        assert_eq!(report.uncovered_words, alloc::vec!["xyzzy".to_string()]);
    }

    #[test]
    fn test_coverage_empty() {
        let dict = PronunciationDict::english_minimal();
        let report = dict.coverage("");
        assert_eq!(report.total_tokens, 0);
        assert_eq!(report.coverage_pct(), 0.0);
        assert!(report.is_fully_covered());
    }

    #[test]
    fn test_coverage_strips_punctuation() {
        let dict = PronunciationDict::english_minimal();
        let report = dict.coverage("hello, world! the.");
        assert_eq!(report.total_tokens, 3);
        assert_eq!(report.covered_tokens, 3);
    }

    #[test]
    fn test_coverage_case_insensitive() {
        let dict = PronunciationDict::english_minimal();
        let report = dict.coverage("Hello THE World");
        assert_eq!(report.covered_tokens, 3);
    }

    #[test]
    fn test_coverage_unique_uncovered() {
        let dict = PronunciationDict::english_minimal();
        let report = dict.coverage("xyzzy xyzzy xyzzy");
        assert_eq!(report.total_tokens, 3);
        assert_eq!(report.covered_tokens, 0);
        // Only one unique uncovered word.
        assert_eq!(report.uncovered_count(), 1);
    }

    #[test]
    fn test_coverage_serde_roundtrip() {
        let report = CoverageReport {
            total_tokens: 10,
            covered_tokens: 8,
            uncovered_words: alloc::vec!["foo".to_string(), "bar".to_string()],
        };
        let json = serde_json::to_string(&report).unwrap();
        let rt: CoverageReport = serde_json::from_str(&json).unwrap();
        assert_eq!(report, rt);
    }

    #[test]
    fn test_normalize_token() {
        assert_eq!(normalize_token("Hello,"), "hello");
        assert_eq!(normalize_token("\"world\""), "world");
        assert_eq!(normalize_token("(test)"), "test");
        assert_eq!(normalize_token("..."), "");
    }
}
