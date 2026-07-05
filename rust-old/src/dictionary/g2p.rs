//! Grapheme-to-phoneme (G2P) model integration.
//!
//! Defines the [`G2PModel`] trait for neural or rule-based G2P engines,
//! [`G2PResult`] for predictions with confidence scores, and [`FallbackDict`]
//! for chaining dictionary lookup with G2P fallback.
//!
//! # Architecture
//!
//! shabdakosh provides the trait; model crates provide implementations.
//! This follows the pattern used by gruut and eSpeak-ng:
//!
//! ```text
//! lookup("word") → user overlay → base dictionary → G2P model
//! ```
//!
//! # Example
//!
//! ```rust
//! use shabdakosh::dictionary::g2p::{G2PModel, G2PResult, FallbackDict, LookupSource};
//! use shabdakosh::PronunciationDict;
//! use svara::phoneme::Phoneme;
//!
//! struct DummyModel;
//!
//! impl G2PModel for DummyModel {
//!     fn predict(&self, word: &str) -> Option<G2PResult> {
//!         Some(G2PResult::new(vec![Phoneme::VowelSchwa], 0.5))
//!     }
//! }
//!
//! let dict = PronunciationDict::english_minimal();
//! let fallback = dict.with_fallback(DummyModel);
//!
//! // Known word — comes from dictionary
//! let (phonemes, source) = fallback.lookup_with_source("hello").unwrap();
//! assert!(matches!(source, LookupSource::BaseDictionary));
//!
//! // Unknown word — falls through to G2P model
//! let (phonemes, source) = fallback.lookup_with_source("xyzzy").unwrap();
//! assert!(matches!(source, LookupSource::G2PModel { .. }));
//! ```

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use svara::phoneme::Phoneme;

use super::PronunciationDict;
use super::entry::DictEntry;

/// Result of a G2P model prediction.
///
/// Contains the predicted phoneme sequence and a confidence score
/// indicating how reliable the prediction is.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct G2PResult {
    phonemes: Vec<Phoneme>,
    confidence: f32,
}

impl G2PResult {
    /// Creates a new G2P result with the given phonemes and confidence.
    ///
    /// Confidence should be in the range 0.0 (no confidence) to 1.0 (certain).
    #[must_use]
    pub fn new(phonemes: Vec<Phoneme>, confidence: f32) -> Self {
        Self {
            phonemes,
            confidence,
        }
    }

    /// Returns the predicted phoneme sequence.
    #[must_use]
    pub fn phonemes(&self) -> &[Phoneme] {
        &self.phonemes
    }

    /// Returns the confidence score (0.0–1.0).
    #[must_use]
    pub fn confidence(&self) -> f32 {
        self.confidence
    }

    /// Consumes the result and returns the phoneme vector.
    #[must_use]
    pub fn into_phonemes(self) -> Vec<Phoneme> {
        self.phonemes
    }
}

/// Trait for grapheme-to-phoneme models.
///
/// Implementations convert a written word into a predicted phoneme sequence.
/// shabdakosh provides this trait; model crates (neural, FST, rule-based)
/// provide implementations.
///
/// # Implementors
///
/// - Neural models (e.g., DeepPhonemizer, OpenPhonemizer)
/// - WFST models (e.g., Phonetisaurus)
/// - Rule-based engines (e.g., eSpeak letter-to-sound rules)
pub trait G2PModel: Send + Sync {
    /// Predicts the pronunciation of a word.
    ///
    /// Returns `None` if the model cannot produce a prediction for this word.
    fn predict(&self, word: &str) -> Option<G2PResult>;
}

/// Indicates where a lookup result came from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LookupSource {
    /// Result came from the user overlay.
    UserOverlay,
    /// Result came from the base dictionary.
    BaseDictionary,
    /// Result came from a G2P model prediction.
    G2PModel {
        /// Confidence score of the prediction (0.0–1.0).
        confidence: f32,
    },
}

/// A pronunciation dictionary with G2P model fallback.
///
/// Wraps a [`PronunciationDict`] and a [`G2PModel`] implementation.
/// Lookups try: user overlay → base dictionary → G2P model.
///
/// The model is runtime-only and not serialized. Use [`dict`](Self::dict)
/// and [`dict_mut`](Self::dict_mut) to access the underlying dictionary
/// for serialization.
pub struct FallbackDict<M: G2PModel> {
    dict: PronunciationDict,
    model: M,
}

impl<M: G2PModel> FallbackDict<M> {
    /// Creates a new fallback dictionary from a dictionary and model.
    #[must_use]
    pub fn new(dict: PronunciationDict, model: M) -> Self {
        Self { dict, model }
    }

    /// Returns a reference to the underlying dictionary.
    #[must_use]
    pub fn dict(&self) -> &PronunciationDict {
        &self.dict
    }

    /// Returns a mutable reference to the underlying dictionary.
    pub fn dict_mut(&mut self) -> &mut PronunciationDict {
        &mut self.dict
    }

    /// Returns a reference to the G2P model.
    #[must_use]
    pub fn model(&self) -> &M {
        &self.model
    }

    /// Looks up the primary pronunciation of a word.
    ///
    /// Tries: user overlay → base dictionary → G2P model.
    /// For dictionary results, returns the phoneme slice directly.
    /// For G2P results, the phonemes are owned — use [`lookup_with_source`](Self::lookup_with_source)
    /// to get full details.
    #[must_use]
    pub fn lookup(&self, word: &str) -> Option<Vec<Phoneme>> {
        if let Some(phonemes) = self.dict.lookup(word) {
            return Some(phonemes.to_vec());
        }
        self.model
            .predict(&word.to_lowercase())
            .map(|r| r.into_phonemes())
    }

    /// Looks up a word and returns both the phonemes and the source.
    ///
    /// The lookup chain is: user overlay → base dictionary → G2P model.
    #[must_use]
    pub fn lookup_with_source(&self, word: &str) -> Option<(Vec<Phoneme>, LookupSource)> {
        let key = alloc::string::ToString::to_string(&word.to_lowercase());

        // Check user overlay first.
        if let Some(entry) = self.dict.user_entries().get(&key) {
            return Some((entry.primary_phonemes().to_vec(), LookupSource::UserOverlay));
        }

        // Check base dictionary.
        if let Some(entry) = self.dict.entries().get(&key) {
            return Some((
                entry.primary_phonemes().to_vec(),
                LookupSource::BaseDictionary,
            ));
        }

        // Fall through to G2P model.
        let result = self.model.predict(&key)?;
        let confidence = result.confidence();
        Some((
            result.into_phonemes(),
            LookupSource::G2PModel { confidence },
        ))
    }

    /// Looks up the full dictionary entry for a word.
    ///
    /// Only checks the dictionary layers (user overlay + base), not the G2P model.
    /// Use [`lookup_with_source`](Self::lookup_with_source) for G2P fallback.
    #[must_use]
    pub fn lookup_entry(&self, word: &str) -> Option<&DictEntry> {
        self.dict.lookup_entry(word)
    }

    /// Promotes a G2P prediction to the user overlay.
    ///
    /// If the word is already in the dictionary (user or base), this is a no-op.
    /// Returns `true` if a prediction was promoted.
    pub fn promote_prediction(&mut self, word: &str) -> bool {
        // Already in dictionary — nothing to promote.
        if self.dict.lookup_entry(word).is_some() {
            return false;
        }

        let key = alloc::string::ToString::to_string(&word.to_lowercase());
        if let Some(result) = self.model.predict(&key) {
            self.dict.insert_user(word, result.phonemes());
            true
        } else {
            false
        }
    }

    /// Promotes a G2P prediction only if its confidence meets the threshold.
    ///
    /// Returns `true` if a prediction was promoted.
    pub fn promote_if_confident(&mut self, word: &str, threshold: f32) -> bool {
        if self.dict.lookup_entry(word).is_some() {
            return false;
        }

        let key = alloc::string::ToString::to_string(&word.to_lowercase());
        if let Some(result) = self.model.predict(&key)
            && result.confidence() >= threshold
        {
            self.dict.insert_user(word, result.phonemes());
            return true;
        }
        false
    }

    /// Consumes the fallback dictionary and returns the inner dictionary and model.
    #[must_use]
    pub fn into_parts(self) -> (PronunciationDict, M) {
        (self.dict, self.model)
    }
}

impl<M: G2PModel + core::fmt::Debug> core::fmt::Debug for FallbackDict<M> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FallbackDict")
            .field("dict", &self.dict)
            .field("model", &self.model)
            .finish()
    }
}

impl<M: G2PModel + Clone> Clone for FallbackDict<M> {
    fn clone(&self) -> Self {
        Self {
            dict: self.dict.clone(),
            model: self.model.clone(),
        }
    }
}

// --- Phonetisaurus FST integration ---

/// A Weighted Finite-State Transducer (WFST) based G2P model wrapper.
///
/// This struct provides the integration point for Phonetisaurus-style
/// WFST models. It is lighter than neural models and offers good accuracy
/// for many languages.
///
/// # Usage
///
/// Phonetisaurus models produce ARPABET or IPA output which must be
/// converted to svara phonemes. Configure the notation via [`FstNotation`].
///
/// ```rust,no_run
/// use shabdakosh::dictionary::g2p::{FstModel, FstNotation, G2PModel};
///
/// // Load a pre-trained FST model file.
/// let model = FstModel::new("/path/to/model.fst", FstNotation::Arpabet);
///
/// // Use with a fallback dictionary:
/// // let fallback = dict.with_fallback(model);
/// ```
///
/// # Implementation
///
/// `FstModel` implements [`G2PModel`] by:
/// 1. Passing the input word through the WFST
/// 2. Converting the output symbols to svara phonemes (ARPABET or IPA)
/// 3. Deriving a confidence score from the path weight
///
/// The actual FST engine is **not bundled** — this wrapper calls out to
/// an external library or binary. See the `phonetisaurus-g2p-rs` crate
/// (when available) for a pure-Rust implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FstModel {
    model_path: alloc::string::String,
    notation: FstNotation,
}

/// Output notation of an FST model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FstNotation {
    /// Model outputs ARPABET symbols (e.g., "K AE1 T").
    Arpabet,
    /// Model outputs IPA symbols (e.g., "kæt").
    Ipa,
}

impl FstModel {
    /// Creates a new FST model wrapper pointing to a model file.
    ///
    /// The model file is not loaded until [`predict`](G2PModel::predict) is called.
    #[must_use]
    pub fn new(model_path: &str, notation: FstNotation) -> Self {
        Self {
            model_path: alloc::string::ToString::to_string(model_path),
            notation,
        }
    }

    /// Returns the path to the model file.
    #[must_use]
    pub fn model_path(&self) -> &str {
        &self.model_path
    }

    /// Returns the output notation of this model.
    #[must_use]
    pub fn notation(&self) -> FstNotation {
        self.notation
    }
}

impl G2PModel for FstModel {
    /// Predicts pronunciation using the FST model.
    ///
    /// **Current status:** This is a stub that always returns `None`.
    /// Wire in an actual FST engine (e.g., `phonetisaurus-g2p-rs`) to
    /// provide real predictions.
    fn predict(&self, _word: &str) -> Option<G2PResult> {
        // Stub: when a real FST engine is available, this would:
        // 1. Run the word through the WFST at self.model_path
        // 2. Parse output symbols according to self.notation
        // 3. Convert to svara phonemes via arpabet:: or ipa:: module
        // 4. Derive confidence from path weight (e.g., exp(-cost))
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A mock G2P model for testing.
    #[derive(Debug, Clone)]
    struct MockG2PModel {
        /// Fixed confidence for all predictions.
        confidence: f32,
    }

    impl MockG2PModel {
        fn new(confidence: f32) -> Self {
            Self { confidence }
        }
    }

    impl G2PModel for MockG2PModel {
        fn predict(&self, _word: &str) -> Option<G2PResult> {
            Some(G2PResult::new(
                alloc::vec![Phoneme::VowelSchwa],
                self.confidence,
            ))
        }
    }

    /// A model that always returns None.
    #[derive(Debug, Clone)]
    struct NoopModel;

    impl G2PModel for NoopModel {
        fn predict(&self, _word: &str) -> Option<G2PResult> {
            None
        }
    }

    #[test]
    fn test_g2p_result_new() {
        let result = G2PResult::new(alloc::vec![Phoneme::PlosiveK], 0.85);
        assert_eq!(result.phonemes(), &[Phoneme::PlosiveK]);
        assert_eq!(result.confidence(), 0.85);
    }

    #[test]
    fn test_g2p_result_into_phonemes() {
        let result = G2PResult::new(alloc::vec![Phoneme::PlosiveK, Phoneme::VowelAsh], 0.9);
        let phonemes = result.into_phonemes();
        assert_eq!(phonemes, alloc::vec![Phoneme::PlosiveK, Phoneme::VowelAsh]);
    }

    #[test]
    fn test_g2p_result_serde_roundtrip() {
        let result = G2PResult::new(alloc::vec![Phoneme::PlosiveK, Phoneme::VowelAsh], 0.75);
        let json = serde_json::to_string(&result).unwrap();
        let result2: G2PResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result, result2);
    }

    #[test]
    fn test_lookup_source_serde_roundtrip() {
        let sources = [
            LookupSource::UserOverlay,
            LookupSource::BaseDictionary,
            LookupSource::G2PModel { confidence: 0.8 },
        ];
        for source in &sources {
            let json = serde_json::to_string(source).unwrap();
            let source2: LookupSource = serde_json::from_str(&json).unwrap();
            assert_eq!(source, &source2);
        }
    }

    #[test]
    fn test_fallback_dict_lookup_from_dict() {
        let dict = PronunciationDict::english_minimal();
        let fallback = FallbackDict::new(dict, MockG2PModel::new(0.5));

        // "hello" is in the minimal dictionary.
        let phonemes = fallback.lookup("hello");
        assert!(phonemes.is_some());
    }

    #[test]
    fn test_fallback_dict_lookup_from_g2p() {
        let dict = PronunciationDict::english_minimal();
        let fallback = FallbackDict::new(dict, MockG2PModel::new(0.5));

        // "xyzzy" is not in any dictionary.
        let phonemes = fallback.lookup("xyzzy");
        assert!(phonemes.is_some());
        assert_eq!(phonemes.unwrap(), alloc::vec![Phoneme::VowelSchwa]);
    }

    #[test]
    fn test_fallback_dict_lookup_none_when_no_model_result() {
        let dict = PronunciationDict::english_minimal();
        let fallback = FallbackDict::new(dict, NoopModel);

        assert!(fallback.lookup("xyzzy").is_none());
    }

    #[test]
    fn test_lookup_with_source_user_overlay() {
        let mut dict = PronunciationDict::english_minimal();
        dict.insert_user("custom", &[Phoneme::VowelA]);
        let fallback = FallbackDict::new(dict, MockG2PModel::new(0.5));

        let (_, source) = fallback.lookup_with_source("custom").unwrap();
        assert_eq!(source, LookupSource::UserOverlay);
    }

    #[test]
    fn test_lookup_with_source_base_dict() {
        let dict = PronunciationDict::english_minimal();
        let fallback = FallbackDict::new(dict, MockG2PModel::new(0.5));

        let (_, source) = fallback.lookup_with_source("hello").unwrap();
        assert_eq!(source, LookupSource::BaseDictionary);
    }

    #[test]
    fn test_lookup_with_source_g2p() {
        let dict = PronunciationDict::english_minimal();
        let fallback = FallbackDict::new(dict, MockG2PModel::new(0.75));

        let (phonemes, source) = fallback.lookup_with_source("xyzzy").unwrap();
        assert_eq!(phonemes, alloc::vec![Phoneme::VowelSchwa]);
        assert_eq!(source, LookupSource::G2PModel { confidence: 0.75 });
    }

    #[test]
    fn test_promote_prediction() {
        let dict = PronunciationDict::english_minimal();
        let mut fallback = FallbackDict::new(dict, MockG2PModel::new(0.8));

        assert!(fallback.promote_prediction("newword"));
        // Now it's in user overlay — second promote is a no-op.
        assert!(!fallback.promote_prediction("newword"));
        assert_eq!(fallback.dict().user_len(), 1);
    }

    #[test]
    fn test_promote_prediction_skips_existing() {
        let dict = PronunciationDict::english_minimal();
        let mut fallback = FallbackDict::new(dict, MockG2PModel::new(0.8));

        // "hello" is already in base dict — should not promote.
        assert!(!fallback.promote_prediction("hello"));
        assert_eq!(fallback.dict().user_len(), 0);
    }

    #[test]
    fn test_promote_if_confident_above_threshold() {
        let dict = PronunciationDict::english_minimal();
        let mut fallback = FallbackDict::new(dict, MockG2PModel::new(0.8));

        assert!(fallback.promote_if_confident("newword", 0.7));
        assert_eq!(fallback.dict().user_len(), 1);
    }

    #[test]
    fn test_promote_if_confident_below_threshold() {
        let dict = PronunciationDict::english_minimal();
        let mut fallback = FallbackDict::new(dict, MockG2PModel::new(0.5));

        assert!(!fallback.promote_if_confident("newword", 0.7));
        assert_eq!(fallback.dict().user_len(), 0);
    }

    #[test]
    fn test_into_parts() {
        let dict = PronunciationDict::english_minimal();
        let fallback = FallbackDict::new(dict.clone(), MockG2PModel::new(0.5));
        let (recovered_dict, _model) = fallback.into_parts();
        assert_eq!(recovered_dict.len(), dict.len());
    }

    #[test]
    fn test_fallback_dict_lookup_entry() {
        let dict = PronunciationDict::english_minimal();
        let fallback = FallbackDict::new(dict, MockG2PModel::new(0.5));

        assert!(fallback.lookup_entry("hello").is_some());
        assert!(fallback.lookup_entry("xyzzy").is_none());
    }

    #[test]
    fn test_user_overlay_takes_precedence_over_g2p() {
        let mut dict = PronunciationDict::new();
        dict.insert_user("test", &[Phoneme::PlosiveT]);
        let fallback = FallbackDict::new(dict, MockG2PModel::new(0.9));

        let (phonemes, source) = fallback.lookup_with_source("test").unwrap();
        assert_eq!(phonemes, alloc::vec![Phoneme::PlosiveT]);
        assert_eq!(source, LookupSource::UserOverlay);
    }

    #[test]
    fn test_fst_model_serde_roundtrip() {
        let model = FstModel::new("/path/to/model.fst", FstNotation::Arpabet);
        let json = serde_json::to_string(&model).unwrap();
        let model2: FstModel = serde_json::from_str(&json).unwrap();
        assert_eq!(model.model_path(), model2.model_path());
        assert_eq!(model.notation(), model2.notation());
    }

    #[test]
    fn test_fst_notation_serde_roundtrip() {
        for notation in [FstNotation::Arpabet, FstNotation::Ipa] {
            let json = serde_json::to_string(&notation).unwrap();
            let notation2: FstNotation = serde_json::from_str(&json).unwrap();
            assert_eq!(notation, notation2);
        }
    }
}
