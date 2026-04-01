//! Prefix trie for dictionary word lookup and autocomplete.
//!
//! [`PrefixTrie`] provides O(k) prefix-based search over dictionary words,
//! where k is the length of the prefix. This enables autocomplete and
//! partial matching (e.g., "comput" matches "computer", "compute", "computing").
//!
//! # Examples
//!
//! ```rust
//! use shabdakosh::dictionary::trie::PrefixTrie;
//! use shabdakosh::PronunciationDict;
//!
//! let dict = PronunciationDict::english_minimal();
//! let trie = PrefixTrie::from_dict(&dict);
//!
//! assert!(trie.contains("hello"));
//! assert!(!trie.contains("xyzzy"));
//!
//! let matches = trie.search_prefix("he");
//! assert!(matches.contains(&"hello".to_string()));
//! ```

use alloc::{collections::BTreeMap, string::String, vec::Vec};
use serde::{Deserialize, Serialize};

use super::PronunciationDict;

/// A prefix trie node.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct TrieNode {
    children: BTreeMap<char, TrieNode>,
    is_word: bool,
}

/// A prefix trie built from dictionary words.
///
/// Supports O(k) exact match and prefix search, where k is the key length.
/// Uses [`BTreeMap`] for children to keep memory layout compact and
/// serialization deterministic.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrefixTrie {
    root: TrieNode,
    word_count: usize,
}

impl PrefixTrie {
    /// Creates an empty trie.
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: TrieNode::default(),
            word_count: 0,
        }
    }

    /// Builds a trie from all words in a dictionary (base + user overlay).
    #[must_use]
    pub fn from_dict(dict: &PronunciationDict) -> Self {
        let mut trie = Self::new();
        for word in dict.entries().keys() {
            trie.insert(word);
        }
        for word in dict.user_entries().keys() {
            trie.insert(word);
        }
        trie
    }

    /// Inserts a word into the trie.
    pub fn insert(&mut self, word: &str) {
        let mut node = &mut self.root;
        for ch in word.chars() {
            node = node.children.entry(ch).or_default();
        }
        if !node.is_word {
            node.is_word = true;
            self.word_count += 1;
        }
    }

    /// Returns `true` if the trie contains the exact word.
    #[must_use]
    pub fn contains(&self, word: &str) -> bool {
        let mut node = &self.root;
        for ch in word.chars() {
            match node.children.get(&ch) {
                Some(child) => node = child,
                None => return false,
            }
        }
        node.is_word
    }

    /// Returns all words in the trie that start with the given prefix.
    ///
    /// Results are sorted alphabetically.
    #[must_use]
    pub fn search_prefix(&self, prefix: &str) -> Vec<String> {
        let mut node = &self.root;
        for ch in prefix.chars() {
            match node.children.get(&ch) {
                Some(child) => node = child,
                None => return Vec::new(),
            }
        }
        let mut results = Vec::new();
        let mut current = String::from(prefix);
        collect_words(node, &mut current, &mut results);
        results
    }

    /// Returns the number of words in the trie.
    #[must_use]
    pub fn len(&self) -> usize {
        self.word_count
    }

    /// Returns `true` if the trie contains no words.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.word_count == 0
    }
}

/// Recursively collects all words from a node.
fn collect_words(node: &TrieNode, current: &mut String, results: &mut Vec<String>) {
    if node.is_word {
        results.push(current.clone());
    }
    for (&ch, child) in &node.children {
        current.push(ch);
        collect_words(child, current, results);
        current.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_trie() {
        let trie = PrefixTrie::new();
        assert!(trie.is_empty());
        assert_eq!(trie.len(), 0);
        assert!(!trie.contains("hello"));
        assert!(trie.search_prefix("h").is_empty());
    }

    #[test]
    fn test_insert_and_contains() {
        let mut trie = PrefixTrie::new();
        trie.insert("hello");
        trie.insert("help");
        trie.insert("world");

        assert!(trie.contains("hello"));
        assert!(trie.contains("help"));
        assert!(trie.contains("world"));
        assert!(!trie.contains("hell"));
        assert!(!trie.contains("helloo"));
        assert_eq!(trie.len(), 3);
    }

    #[test]
    fn test_duplicate_insert() {
        let mut trie = PrefixTrie::new();
        trie.insert("hello");
        trie.insert("hello");
        assert_eq!(trie.len(), 1);
    }

    #[test]
    fn test_search_prefix() {
        let mut trie = PrefixTrie::new();
        trie.insert("hello");
        trie.insert("help");
        trie.insert("helper");
        trie.insert("world");

        let results = trie.search_prefix("hel");
        assert_eq!(results.len(), 3);
        assert!(results.contains(&"hello".to_string()));
        assert!(results.contains(&"help".to_string()));
        assert!(results.contains(&"helper".to_string()));
    }

    #[test]
    fn test_search_prefix_exact_match() {
        let mut trie = PrefixTrie::new();
        trie.insert("hello");

        let results = trie.search_prefix("hello");
        assert_eq!(results, alloc::vec!["hello".to_string()]);
    }

    #[test]
    fn test_search_prefix_no_match() {
        let mut trie = PrefixTrie::new();
        trie.insert("hello");

        assert!(trie.search_prefix("xyz").is_empty());
    }

    #[test]
    fn test_search_prefix_empty() {
        let mut trie = PrefixTrie::new();
        trie.insert("a");
        trie.insert("b");

        let results = trie.search_prefix("");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_from_dict() {
        let dict = PronunciationDict::english_minimal();
        let trie = PrefixTrie::from_dict(&dict);

        assert_eq!(trie.len(), dict.len());
        assert!(trie.contains("hello"));
        assert!(trie.contains("the"));
        assert!(!trie.contains("xyzzy"));
    }

    #[test]
    fn test_from_dict_includes_user_overlay() {
        let mut dict = PronunciationDict::new();
        dict.insert("base", &[svara::phoneme::Phoneme::PlosiveB]);
        dict.insert_user("user", &[svara::phoneme::Phoneme::VowelU]);
        let trie = PrefixTrie::from_dict(&dict);

        assert!(trie.contains("base"));
        assert!(trie.contains("user"));
        assert_eq!(trie.len(), 2);
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut trie = PrefixTrie::new();
        trie.insert("hello");
        trie.insert("help");
        trie.insert("world");

        let json = serde_json::to_string(&trie).unwrap();
        let trie2: PrefixTrie = serde_json::from_str(&json).unwrap();

        assert_eq!(trie.len(), trie2.len());
        assert!(trie2.contains("hello"));
        assert!(trie2.contains("help"));
        assert!(trie2.contains("world"));
    }

    #[test]
    fn test_results_sorted() {
        let mut trie = PrefixTrie::new();
        trie.insert("cat");
        trie.insert("car");
        trie.insert("card");
        trie.insert("care");

        let results = trie.search_prefix("car");
        // BTreeMap iteration is sorted, so results should be alphabetical.
        assert_eq!(
            results,
            alloc::vec!["car".to_string(), "card".to_string(), "care".to_string()]
        );
    }
}
