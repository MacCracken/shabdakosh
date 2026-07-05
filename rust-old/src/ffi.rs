//! C-compatible Foreign Function Interface for dictionary lookup.
//!
//! Provides an `extern "C"` API using opaque pointer patterns, enabling
//! integration with C/C++ TTS engines, Python bindings (via PyO3), and
//! other non-Rust consumers.
//!
//! # Memory Model
//!
//! - `shabdakosh_dict_*` functions create/destroy dictionary handles.
//! - `shabdakosh_lookup` returns a result handle with phoneme data.
//! - All handles must be freed by the corresponding `*_free` function.
//! - Passing null pointers to any function is safe (returns null/0).
//!
//! # Example (C)
//!
//! ```c
//! #include "shabdakosh.h"
//!
//! ShabdakoshDict* dict = shabdakosh_dict_english();
//! ShabdakoshLookupResult* result = shabdakosh_lookup(dict, "hello");
//! if (result) {
//!     size_t count = shabdakosh_result_phoneme_count(result);
//!     // ... use phoneme data ...
//!     shabdakosh_result_free(result);
//! }
//! shabdakosh_dict_free(dict);
//! ```

use std::ffi::{CStr, c_char};
use std::ptr;

use crate::PronunciationDict;

/// Opaque dictionary handle for C consumers.
pub type ShabdakoshDict = PronunciationDict;

/// Result of a dictionary lookup, containing phoneme data.
pub struct ShabdakoshLookupResult {
    /// The phoneme names as a list of C-compatible strings.
    phoneme_names: Vec<std::ffi::CString>,
    count: usize,
}

/// Creates the built-in English pronunciation dictionary.
///
/// Returns a heap-allocated dictionary handle. Must be freed with
/// [`shabdakosh_dict_free`].
///
/// # Safety
///
/// The returned pointer is valid until freed. Do not use after calling
/// `shabdakosh_dict_free`.
#[unsafe(no_mangle)]
pub extern "C" fn shabdakosh_dict_english() -> *mut ShabdakoshDict {
    Box::into_raw(Box::new(PronunciationDict::english()))
}

/// Creates a new empty dictionary.
///
/// # Safety
///
/// Same as [`shabdakosh_dict_english`].
#[unsafe(no_mangle)]
pub extern "C" fn shabdakosh_dict_new() -> *mut ShabdakoshDict {
    Box::into_raw(Box::new(PronunciationDict::new()))
}

/// Frees a dictionary handle.
///
/// # Safety
///
/// `dict` must be a pointer returned by `shabdakosh_dict_english` or
/// `shabdakosh_dict_new`, and must not have been freed already.
/// Passing null is safe (no-op).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shabdakosh_dict_free(dict: *mut ShabdakoshDict) {
    if !dict.is_null() {
        drop(unsafe { Box::from_raw(dict) });
    }
}

/// Returns the number of entries in the dictionary.
///
/// # Safety
///
/// `dict` must be a valid dictionary pointer. Returns 0 if null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shabdakosh_dict_len(dict: *const ShabdakoshDict) -> usize {
    if dict.is_null() {
        return 0;
    }
    unsafe { &*dict }.len()
}

/// Looks up a word in the dictionary.
///
/// Returns a heap-allocated result handle containing the phoneme data,
/// or null if the word is not found. Must be freed with
/// [`shabdakosh_result_free`].
///
/// # Safety
///
/// - `dict` must be a valid dictionary pointer.
/// - `word` must be a valid null-terminated UTF-8 string.
/// - Returns null if either pointer is null or the word is not found.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shabdakosh_lookup(
    dict: *const ShabdakoshDict,
    word: *const c_char,
) -> *mut ShabdakoshLookupResult {
    if dict.is_null() || word.is_null() {
        return ptr::null_mut();
    }

    let word_str = match unsafe { CStr::from_ptr(word) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let dict_ref = unsafe { &*dict };
    let phonemes = match dict_ref.lookup(word_str) {
        Some(p) => p,
        None => return ptr::null_mut(),
    };

    let phoneme_names: Vec<std::ffi::CString> = phonemes
        .iter()
        .filter_map(|p| {
            let ipa = crate::ipa::phoneme_to_ipa(p)?;
            std::ffi::CString::new(ipa).ok()
        })
        .collect();

    let count = phoneme_names.len();
    Box::into_raw(Box::new(ShabdakoshLookupResult {
        phoneme_names,
        count,
    }))
}

/// Returns the number of phonemes in a lookup result.
///
/// # Safety
///
/// `result` must be a valid result pointer. Returns 0 if null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shabdakosh_result_phoneme_count(
    result: *const ShabdakoshLookupResult,
) -> usize {
    if result.is_null() {
        return 0;
    }
    unsafe { &*result }.count
}

/// Returns a pointer to the IPA string of the phoneme at the given index.
///
/// The returned string is valid until the result is freed.
///
/// # Safety
///
/// - `result` must be a valid result pointer.
/// - `index` must be less than `shabdakosh_result_phoneme_count(result)`.
/// - Returns null if either condition is violated.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shabdakosh_result_phoneme_ipa(
    result: *const ShabdakoshLookupResult,
    index: usize,
) -> *const c_char {
    if result.is_null() {
        return ptr::null();
    }
    let result_ref = unsafe { &*result };
    match result_ref.phoneme_names.get(index) {
        Some(cstr) => cstr.as_ptr(),
        None => ptr::null(),
    }
}

/// Frees a lookup result handle.
///
/// # Safety
///
/// `result` must be a pointer returned by `shabdakosh_lookup`, and must
/// not have been freed already. Passing null is safe (no-op).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shabdakosh_result_free(result: *mut ShabdakoshLookupResult) {
    if !result.is_null() {
        drop(unsafe { Box::from_raw(result) });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_dict_lifecycle() {
        let dict = shabdakosh_dict_english();
        assert!(!dict.is_null());

        let len = unsafe { shabdakosh_dict_len(dict) };
        assert!(len >= 10000);

        unsafe { shabdakosh_dict_free(dict) };
    }

    #[test]
    fn test_ffi_dict_new() {
        let dict = shabdakosh_dict_new();
        assert!(!dict.is_null());

        let len = unsafe { shabdakosh_dict_len(dict) };
        assert_eq!(len, 0);

        unsafe { shabdakosh_dict_free(dict) };
    }

    #[test]
    fn test_ffi_lookup_hit() {
        let dict = shabdakosh_dict_english();
        let word = std::ffi::CString::new("hello").unwrap();

        let result = unsafe { shabdakosh_lookup(dict, word.as_ptr()) };
        assert!(!result.is_null());

        let count = unsafe { shabdakosh_result_phoneme_count(result) };
        assert!(count > 0);

        // Check first phoneme IPA.
        let ipa_ptr = unsafe { shabdakosh_result_phoneme_ipa(result, 0) };
        assert!(!ipa_ptr.is_null());
        let ipa = unsafe { CStr::from_ptr(ipa_ptr) }.to_str().unwrap();
        assert!(!ipa.is_empty());

        unsafe { shabdakosh_result_free(result) };
        unsafe { shabdakosh_dict_free(dict) };
    }

    #[test]
    fn test_ffi_lookup_miss() {
        let dict = shabdakosh_dict_english();
        let word = std::ffi::CString::new("zxqvbnm").unwrap();

        let result = unsafe { shabdakosh_lookup(dict, word.as_ptr()) };
        assert!(result.is_null());

        unsafe { shabdakosh_dict_free(dict) };
    }

    #[test]
    fn test_ffi_null_safety() {
        // All functions should handle null gracefully.
        unsafe {
            shabdakosh_dict_free(ptr::null_mut());
            assert_eq!(shabdakosh_dict_len(ptr::null()), 0);
            assert!(shabdakosh_lookup(ptr::null(), ptr::null()).is_null());
            assert_eq!(shabdakosh_result_phoneme_count(ptr::null()), 0);
            assert!(shabdakosh_result_phoneme_ipa(ptr::null(), 0).is_null());
            shabdakosh_result_free(ptr::null_mut());
        }
    }

    #[test]
    fn test_ffi_result_out_of_bounds() {
        let dict = shabdakosh_dict_english();
        let word = std::ffi::CString::new("hello").unwrap();
        let result = unsafe { shabdakosh_lookup(dict, word.as_ptr()) };
        assert!(!result.is_null());

        let ipa = unsafe { shabdakosh_result_phoneme_ipa(result, 999) };
        assert!(ipa.is_null());

        unsafe { shabdakosh_result_free(result) };
        unsafe { shabdakosh_dict_free(dict) };
    }
}
