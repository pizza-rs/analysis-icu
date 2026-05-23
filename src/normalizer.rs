//! ICU Unicode normalizer (pre-tokenization).
//!
//! Applies Unicode normalization (NFC, NFKC, or NFKC_Casefold) to the
//! full input text before tokenization.

use alloc::borrow::Cow;
use icu_normalizer::{ComposingNormalizer, DecomposingNormalizer};

use pizza_engine::analysis::Normalizer;

/// Unicode normalization mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IcuNormMode {
    /// NFC — Canonical Decomposition followed by Canonical Composition.
    Nfc,
    /// NFD — Canonical Decomposition.
    Nfd,
    /// NFKC — Compatibility Decomposition followed by Canonical Composition.
    Nfkc,
    /// NFKC_Casefold — NFKC + Case Folding (recommended for search).
    NfkcCasefold,
}

/// ICU Unicode normalizer for pre-tokenization text normalization.
///
/// Equivalent to Elasticsearch's `icu_normalizer` char filter.
#[derive(Clone)]
pub struct IcuNormalizer {
    mode: IcuNormMode,
}

impl IcuNormalizer {
    /// Create with the specified normalization form.
    pub fn new(mode: IcuNormMode) -> Self {
        Self { mode }
    }

    /// Create with NFKC_Casefold (best for search — normalizes + lowercases).
    pub fn nfkc_casefold() -> Self {
        Self::new(IcuNormMode::NfkcCasefold)
    }
}

impl Normalizer for IcuNormalizer {
    fn normalize(&self, text: &mut String) {
        let normalized = match self.mode {
            IcuNormMode::Nfc => {
                let normalizer = ComposingNormalizer::new_nfc();
                normalizer.normalize(text.as_str())
            }
            IcuNormMode::Nfkc => {
                let normalizer = ComposingNormalizer::new_nfkc();
                normalizer.normalize(text.as_str())
            }
            IcuNormMode::Nfd => {
                let normalizer = DecomposingNormalizer::new_nfd();
                normalizer.normalize(text.as_str())
            }
            IcuNormMode::NfkcCasefold => {
                // NFKC followed by case folding
                let normalizer = ComposingNormalizer::new_nfkc();
                let nfkc = normalizer.normalize(text.as_str());
                // Apply case folding via lowercase (ICU4X casemap)
                Cow::Owned(nfkc.to_lowercase())
            }
        };

        if normalized != *text {
            *text = normalized.into_owned();
        }
    }
}
