//! Register ICU analysis components into [`AnalysisFactory`].

use alloc::boxed::Box;

use pizza_engine::analysis::AnalysisFactory;

use crate::{IcuCollationFilter, IcuFoldingFilter, IcuNormalizationFilter, IcuNormFilterMode, IcuTokenizer};

/// Register ICU tokenizer and token filters.
///
/// Matches Elasticsearch's analysis-icu plugin registration:
/// - Tokenizer: `icu_tokenizer`
/// - Token Filters: `icu_folding`, `icu_normalizer`, `icu_collation`
pub fn register_all(factory: &mut AnalysisFactory) {
    factory.register_tokenizer("icu_tokenizer", Box::new(IcuTokenizer::new()));
    factory.register_token_filter("icu_folding", Box::new(IcuFoldingFilter::new()));
    factory.register_token_filter("icu_normalizer", Box::new(IcuNormalizationFilter::new(IcuNormFilterMode::NfkcCasefold)));
    factory.register_token_filter("icu_collation", Box::new(IcuCollationFilter::new("root")));
}
