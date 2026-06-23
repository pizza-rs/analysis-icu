//! Register ICU analysis components into [`AnalysisFactory`].

use alloc::boxed::Box;

use pizza_engine::analysis::AnalysisFactory;

use crate::{IcuCollationFilter, IcuFoldingFilter, IcuNormalizationFilter, IcuNormFilterMode, IcuTokenizer, IcuTransformFilter, IcuTransformId};

/// Register ICU tokenizer and token filters.
///
/// Matches Elasticsearch's analysis-icu plugin registration:
/// - Tokenizer: `icu_tokenizer`
/// - Token Filters: `icu_folding`, `icu_normalizer`, `icu_collation`, `icu_transform`
pub fn register_all(factory: &mut AnalysisFactory) {
    factory.register_tokenizer_with("icu_tokenizer", || Box::new(IcuTokenizer::new()));
    factory.register_token_filter_with("icu_folding", || Box::new(IcuFoldingFilter::new()));
    factory.register_token_filter_with("icu_normalizer", || Box::new(IcuNormalizationFilter::new(IcuNormFilterMode::NfkcCasefold)));
    factory.register_token_filter_with("icu_collation", || Box::new(IcuCollationFilter::new("root")));
    factory.register_token_filter_with("icu_transform", || Box::new(IcuTransformFilter::new(IcuTransformId::AnyLatinAscii)));
}
