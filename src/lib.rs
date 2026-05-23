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
mod tokenizer;
mod normalizer;
mod normalization_filter;
mod folding;
mod collation;

pub use tokenizer::IcuTokenizer;
pub use normalizer::{IcuNormalizer, IcuNormMode};
pub use normalization_filter::{IcuNormalizationFilter, IcuNormFilterMode};
pub use folding::IcuFoldingFilter;
pub use collation::IcuCollationFilter;
pub mod register;
pub use register::register_all;
