//! ICU-based Unicode word segmentation tokenizer.
//!
//! Uses ICU4X's word segmenter for proper Unicode word boundary detection
//! following UAX#29 rules. This handles complex scripts (Thai, Khmer, Lao,
//! Burmese, etc.) correctly.

use alloc::borrow::Cow;

use icu_segmenter::{WordSegmenter, options::WordBreakInvariantOptions};

use pizza_engine::analysis::{Token, Tokenizer};

/// Unicode word segmentation tokenizer using ICU4X.
///
/// Equivalent to Elasticsearch's `icu_tokenizer`.
/// Handles Thai, Khmer, Lao, Burmese, and other complex scripts
/// that require dictionary-based segmentation.
#[derive(Clone)]
pub struct IcuTokenizer {
    _private: (),
}

impl IcuTokenizer {
    /// Create a new ICU tokenizer.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for IcuTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for IcuTokenizer {
    fn tokenize<'a>(&self, text: &'a str) -> Vec<Token<'a>> {
        let segmenter = WordSegmenter::new_auto(Default::default());
        let breakpoints: Vec<usize> = segmenter.segment_str(text).collect();

        let mut tokens = Vec::new();
        let mut position = 0u32;

        for window in breakpoints.windows(2) {
            let start = window[0];
            let end = window[1];
            let segment = &text[start..end];

            // Skip whitespace-only and punctuation-only segments
            if segment.chars().all(|ch| ch.is_whitespace() || ch.is_ascii_punctuation()) {
                continue;
            }

            // Skip if segment is only Unicode punctuation/symbols
            if segment.chars().all(|ch| {
                ch.is_whitespace()
                    || matches!(
                        unicode_category(ch),
                        UnicodeCategory::Punctuation | UnicodeCategory::Symbol
                    )
            }) {
                continue;
            }

            let start_offset = text[..start].chars().count() as u32;
            let end_offset = text[..end].chars().count() as u32;

            tokens.push(Token {
                term: Cow::Borrowed(segment),
                start_offset,
                end_offset,
                position,
            });
            position += 1;
        }

        tokens
    }
}

#[derive(PartialEq)]
enum UnicodeCategory {
    Letter,
    Number,
    Punctuation,
    Symbol,
    Other,
}

fn unicode_category(ch: char) -> UnicodeCategory {
    if ch.is_alphabetic() {
        UnicodeCategory::Letter
    } else if ch.is_numeric() {
        UnicodeCategory::Number
    } else if ch.is_ascii_punctuation() {
        UnicodeCategory::Punctuation
    } else {
        // Check Unicode general categories for CJK punctuation, symbols
        let c = ch as u32;
        if (0x2000..=0x206F).contains(&c)  // General Punctuation
            || (0x3000..=0x303F).contains(&c)  // CJK Symbols and Punctuation
            || (0xFE30..=0xFE4F).contains(&c)  // CJK Compatibility Forms
            || (0xFF01..=0xFF0F).contains(&c)  // Fullwidth punctuation
            || (0xFF1A..=0xFF20).contains(&c)
            || (0xFF3B..=0xFF40).contains(&c)
            || (0xFF5B..=0xFF65).contains(&c)
        {
            UnicodeCategory::Punctuation
        } else if (0x2100..=0x214F).contains(&c)  // Letterlike Symbols
            || (0x2190..=0x21FF).contains(&c)  // Arrows
            || (0x2200..=0x22FF).contains(&c)  // Mathematical Operators
        {
            UnicodeCategory::Symbol
        } else {
            UnicodeCategory::Other
        }
    }
}
