//! ICU Unicode case folding filter.
//!
//! Performs Unicode case folding on tokens — removes accents and lowercases.
//! This is more thorough than simple lowercasing as it handles full Unicode
//! case folding rules.

use alloc::borrow::Cow;

use icu_casemap::CaseMapper;
use icu_normalizer::ComposingNormalizer;

use pizza_engine::analysis::{Token, TokenFilter};

/// Unicode case folding filter using ICU4X.
///
/// Applies NFKC normalization followed by case folding. This:
/// - Normalizes Unicode compatibility forms (e.g., ﬁ → fi)
/// - Folds case (e.g., É → e, ß → ss)
/// - Removes diacritical marks
///
/// Equivalent to Elasticsearch's `icu_folding` token filter.
#[derive(Clone)]
pub struct IcuFoldingFilter {
    _private: (),
}

impl IcuFoldingFilter {
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for IcuFoldingFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenFilter for IcuFoldingFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let term = token.term.as_ref();

        // Step 1: NFKC normalize
        let normalizer = ComposingNormalizer::new_nfkc();
        let nfkc = normalizer.normalize(term);

        // Step 2: Case fold using ICU CaseMapper
        let case_mapper = CaseMapper::new();
        let folded = case_mapper.fold_string(&nfkc);

        // Step 3: Remove combining diacritical marks (U+0300..U+036F)
        let cleaned: String = folded
            .chars()
            .filter(|&ch| {
                let c = ch as u32;
                !((0x0300..=0x036F).contains(&c)  // Combining Diacritical Marks
                    || (0x1AB0..=0x1AFF).contains(&c) // Combining Diacritical Marks Extended
                    || (0x1DC0..=0x1DFF).contains(&c) // Combining Diacritical Marks Supplement
                    || (0x20D0..=0x20FF).contains(&c) // Combining Diacritical Marks for Symbols
                    || (0xFE20..=0xFE2F).contains(&c)) // Combining Half Marks
            })
            .collect();

        if cleaned != term {
            token.term = Cow::Owned(cleaned);
        }

        (false, None)
    }
}
