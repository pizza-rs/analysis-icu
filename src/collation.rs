//! ICU collation token filter.
//!
//! Generates locale-aware sort keys from tokens using ICU collation.
//! Sort keys enable proper locale-sensitive ordering in the index.

use alloc::borrow::Cow;

use icu_collator::Collator;
use icu_collator::CollatorPreferences;
use icu_locale_core::locale;

use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;

/// Locale-aware collation sort key filter using ICU4X.
///
/// Replaces token text with its binary sort key, enabling correct
/// locale-sensitive sorting in the search index.
///
/// Equivalent to Elasticsearch's `icu_collation` token filter.
#[derive(Clone)]
pub struct IcuCollationFilter {
    language: String,
}

impl IcuCollationFilter {
    /// Create with a language/locale tag (e.g., "de", "ja", "ko").
    pub fn new(language: &str) -> Self {
        Self {
            language: language.to_string(),
        }
    }

    /// Create for German collation rules.
    pub fn german() -> Self {
        Self::new("de")
    }
}

impl TokenFilter for IcuCollationFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let term = token.term.as_ref();

        // Create collator with locale preferences
        let prefs: CollatorPreferences = match self.language.as_str() {
            "de" => locale!("de").into(),
            "ja" => locale!("ja").into(),
            "ko" => locale!("ko").into(),
            "zh" => locale!("zh").into(),
            _ => Default::default(),
        };

        let collator = Collator::try_new(prefs, Default::default());
        if let Ok(collator) = collator {
            // Get sort key bytes and encode as hex string for indexing
            let mut sort_key = alloc::vec::Vec::new();
            let _ = collator.write_sort_key_to(term, &mut sort_key);
            let hex: String = sort_key
                .iter()
                .map(|b| alloc::format!("{:02x}", b))
                .collect();
            token.term = Cow::Owned(hex);
        }

        (false, None)
    }
}
