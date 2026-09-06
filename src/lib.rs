#![cfg_attr(not(feature = "std"), no_std)]
//! ICU-based Unicode analysis for Pizza search engine.
//!
//! Provides Unicode-aware text analysis using the [ICU4X](https://github.com/unicode-org/icu4x)
//! library (pure Rust implementation of ICU).
//!
//! # Components
//!
//! - [`IcuTokenizer`] — Unicode word segmentation tokenizer (UAX#29 via ICU)
//! - [`IcuNormalizer`] — Unicode normalization normalizer (NFC/NFKC/NFKC_CF)
//! - [`IcuNormalizationFilter`] — Per-token Unicode normalization
//! - [`IcuFoldingFilter`] — Unicode case folding (accent removal + lowercasing)
//! - [`IcuCollationFilter`] — Locale-aware sort key generation
extern crate alloc;
mod collation;
mod folding;
mod normalization_filter;
mod normalizer;
mod tokenizer;
mod transform;

pub use collation::IcuCollationFilter;
pub use folding::IcuFoldingFilter;
pub use normalization_filter::IcuNormFilterMode;
pub use normalization_filter::IcuNormalizationFilter;
pub use normalizer::IcuNormMode;
pub use normalizer::IcuNormalizer;
pub use tokenizer::IcuTokenizer;
pub use transform::IcuTransformFilter;
pub use transform::IcuTransformId;
pub mod register;
pub use register::register_all;
