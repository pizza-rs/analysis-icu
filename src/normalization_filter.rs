//! ICU per-token normalization filter.
//!
//! Applies Unicode normalization to each token individually.

use alloc::borrow::Cow;

use icu_normalizer::ComposingNormalizer;
use icu_normalizer::DecomposingNormalizer;

use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Normalization mode for the per-token filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IcuNormFilterMode {
    Nfc,
    Nfd,
    Nfkc,
    NfkcCasefold,
}

/// Per-token Unicode normalization filter.
///
/// Equivalent to Elasticsearch's `icu_normalizer` token filter.
#[derive(Clone)]
pub struct IcuNormalizationFilter {
    mode: IcuNormFilterMode,
}

impl IcuNormalizationFilter {
    pub fn new(mode: IcuNormFilterMode) -> Self {
        Self { mode }
    }

    pub fn nfkc_casefold() -> Self {
        Self::new(IcuNormFilterMode::NfkcCasefold)
    }
}

impl TokenFilter for IcuNormalizationFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let term = token.term.as_ref();

        let normalized = match self.mode {
            IcuNormFilterMode::Nfc => {
                let normalizer = ComposingNormalizer::new_nfc();
                normalizer.normalize(term)
            }
            IcuNormFilterMode::Nfkc => {
                let normalizer = ComposingNormalizer::new_nfkc();
                normalizer.normalize(term)
            }
            IcuNormFilterMode::Nfd => {
                let normalizer = DecomposingNormalizer::new_nfd();
                normalizer.normalize(term)
            }
            IcuNormFilterMode::NfkcCasefold => {
                let normalizer = ComposingNormalizer::new_nfkc();
                let nfkc = normalizer.normalize(term);
                Cow::Owned(nfkc.to_lowercase())
            }
        };

        if normalized != term {
            token.term = Cow::Owned(normalized.into_owned());
        }

        (false, None)
    }
}
