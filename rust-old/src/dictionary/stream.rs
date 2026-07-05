//! Streaming word-to-phoneme lookup.
//!
//! Process words as an iterator without materializing all results at once.
//! Useful for real-time TTS pipelines where words arrive one at a time.
//!
//! # Examples
//!
//! ```rust
//! use shabdakosh::PronunciationDict;
//!
//! let dict = PronunciationDict::english_minimal();
//! let words = ["hello", "world", "xyzzy"];
//!
//! for (word, phonemes) in dict.lookup_stream(words.iter().copied()) {
//!     match phonemes {
//!         Some(p) => println!("{word} => {p:?}"),
//!         None => println!("{word} => (unknown)"),
//!     }
//! }
//! ```

use svara::phoneme::Phoneme;

use super::PronunciationDict;

/// An iterator that pairs each word with its pronunciation lookup result.
///
/// Created by [`PronunciationDict::lookup_stream`].
pub struct LookupStream<'d, I> {
    dict: &'d PronunciationDict,
    words: I,
}

impl<'d, I, S> Iterator for LookupStream<'d, I>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    type Item = (S, Option<&'d [Phoneme]>);

    fn next(&mut self) -> Option<Self::Item> {
        let word = self.words.next()?;
        let phonemes = self.dict.lookup(word.as_ref());
        Some((word, phonemes))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.words.size_hint()
    }
}

impl PronunciationDict {
    /// Creates a streaming lookup iterator over words.
    ///
    /// Each word is looked up lazily as the iterator is consumed.
    /// No results are buffered — this is a zero-allocation streaming operation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use shabdakosh::PronunciationDict;
    ///
    /// let dict = PronunciationDict::english_minimal();
    /// let results: Vec<_> = dict
    ///     .lookup_stream(["hello", "xyzzy"].iter().copied())
    ///     .collect();
    /// assert!(results[0].1.is_some());  // hello found
    /// assert!(results[1].1.is_none());  // xyzzy not found
    /// ```
    pub fn lookup_stream<I, S>(&self, words: I) -> LookupStream<'_, I::IntoIter>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        LookupStream {
            dict: self,
            words: words.into_iter(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_basic() {
        let dict = PronunciationDict::english_minimal();
        let results: alloc::vec::Vec<_> = dict
            .lookup_stream(["hello", "world"].iter().copied())
            .collect();
        assert_eq!(results.len(), 2);
        assert!(results[0].1.is_some());
        assert!(results[1].1.is_some());
    }

    #[test]
    fn test_stream_with_miss() {
        let dict = PronunciationDict::english_minimal();
        let results: alloc::vec::Vec<_> = dict
            .lookup_stream(["hello", "xyzzy"].iter().copied())
            .collect();
        assert!(results[0].1.is_some());
        assert!(results[1].1.is_none());
    }

    #[test]
    fn test_stream_empty() {
        let dict = PronunciationDict::english_minimal();
        let empty: &[&str] = &[];
        let results: alloc::vec::Vec<_> = dict.lookup_stream(empty.iter().copied()).collect();
        assert!(results.is_empty());
    }

    #[test]
    fn test_stream_with_strings() {
        let dict = PronunciationDict::english_minimal();
        let words = alloc::vec![
            alloc::string::String::from("hello"),
            alloc::string::String::from("the"),
        ];
        let results: alloc::vec::Vec<_> = dict.lookup_stream(words.iter()).collect();
        assert_eq!(results.len(), 2);
        assert!(results[0].1.is_some());
        assert!(results[1].1.is_some());
    }

    #[test]
    fn test_stream_preserves_words() {
        let dict = PronunciationDict::english_minimal();
        let results: alloc::vec::Vec<_> = dict
            .lookup_stream(["hello", "world"].iter().copied())
            .collect();
        assert_eq!(results[0].0, "hello");
        assert_eq!(results[1].0, "world");
    }

    #[test]
    fn test_stream_size_hint() {
        let dict = PronunciationDict::english_minimal();
        let stream = dict.lookup_stream(["a", "b", "c"].iter().copied());
        assert_eq!(stream.size_hint(), (3, Some(3)));
    }
}
