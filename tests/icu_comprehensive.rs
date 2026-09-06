//! Comprehensive tests for the `pizza-analysis-icu` crate.

use std::borrow::Cow;

use pizza_analysis_icu::*;
use pizza_engine::analysis::AnalysisFactory;
use pizza_engine::analysis::Normalizer;
use pizza_engine::analysis::Token;
use pizza_engine::analysis::TokenFilter;
use pizza_engine::analysis::Tokenizer;

// ─── Helpers ───────────────────────────────────────────────────────────────────

fn terms<'a>(tokens: &'a [Token<'a>]) -> Vec<&'a str> {
    tokens.iter().map(|t| t.term.as_ref()).collect()
}

fn make_token(term: &str) -> Token<'_> {
    Token::new(term, 0, term.len() as u32, 0)
}

fn apply_filter<'a>(filter: &dyn TokenFilter, term: &'a str) -> String {
    let mut token = make_token(term);
    filter.filter(&mut token);
    token.term.into_owned()
}

// ═══════════════════════════════════════════════════════════════════════════════
// mod icu_tokenizer — UAX#29 word segmentation
// ═══════════════════════════════════════════════════════════════════════════════

mod icu_tokenizer {
    use super::*;

    #[test]
    fn english_simple() {
        let tok = IcuTokenizer::new();
        let tokens = tok.tokenize("Hello world");
        assert_eq!(terms(&tokens), vec!["Hello", "world"]);
    }

    #[test]
    fn english_multiple_words() {
        let tok = IcuTokenizer::new();
        let tokens = tok.tokenize("The quick brown fox jumps");
        assert_eq!(
            terms(&tokens),
            vec!["The", "quick", "brown", "fox", "jumps"]
        );
    }

    #[test]
    fn empty_string() {
        let tok = IcuTokenizer::new();
        let tokens = tok.tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn whitespace_only() {
        let tok = IcuTokenizer::new();
        let tokens = tok.tokenize("   \t\n  ");
        assert!(tokens.is_empty());
    }

    #[test]
    fn numbers() {
        let tok = IcuTokenizer::new();
        let tokens = tok.tokenize("123 456");
        let t = terms(&tokens);
        assert!(t.contains(&"123"));
        assert!(t.contains(&"456"));
    }

    #[test]
    fn punctuation_stripped() {
        let tok = IcuTokenizer::new();
        let tokens = tok.tokenize("hello, world!");
        let t = terms(&tokens);
        assert!(t.contains(&"hello"));
        assert!(t.contains(&"world"));
        // Punctuation itself should not appear as a token
        assert!(!t.iter().any(|s| *s == "," || *s == "!"));
    }

    #[test]
    fn cjk_mixed_text() {
        let tok = IcuTokenizer::new();
        let tokens = tok.tokenize("Hello世界");
        // Should segment CJK characters (at least produce some tokens)
        assert!(!tokens.is_empty());
    }

    #[test]
    fn cjk_chinese() {
        let tok = IcuTokenizer::new();
        let tokens = tok.tokenize("中文测试");
        assert!(!tokens.is_empty());
    }

    #[test]
    fn thai_text() {
        let tok = IcuTokenizer::new();
        let tokens = tok.tokenize("สวัสดีครับ");
        // Thai should be segmented (ICU dictionary-based)
        assert!(!tokens.is_empty());
    }

    #[test]
    fn arabic_text() {
        let tok = IcuTokenizer::new();
        let tokens = tok.tokenize("مرحبا بالعالم");
        let t = terms(&tokens);
        assert!(
            t.len() >= 2,
            "Arabic should produce at least 2 tokens: {:?}",
            t
        );
    }

    #[test]
    fn positions_sequential() {
        let tok = IcuTokenizer::new();
        let tokens = tok.tokenize("one two three");
        for (i, token) in tokens.iter().enumerate() {
            assert_eq!(token.position, i as u32, "position mismatch at index {}", i);
        }
    }

    #[test]
    fn offsets_nonzero_for_later_tokens() {
        let tok = IcuTokenizer::new();
        let tokens = tok.tokenize("aaa bbb");
        assert!(tokens.len() >= 2);
        assert_eq!(tokens[0].start_offset, 0);
        assert!(tokens[1].start_offset > 0);
    }

    #[test]
    fn single_word() {
        let tok = IcuTokenizer::new();
        let tokens = tok.tokenize("hello");
        assert_eq!(terms(&tokens), vec!["hello"]);
    }

    #[test]
    fn default_trait() {
        let tok = IcuTokenizer::default();
        let tokens = tok.tokenize("test");
        assert_eq!(terms(&tokens), vec!["test"]);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// mod icu_normalizer — Pre-tokenization Unicode normalization
// ═══════════════════════════════════════════════════════════════════════════════

mod icu_normalizer {
    use super::*;

    #[test]
    fn nfc_composes_decomposed() {
        // e + combining acute accent (U+0301) → é (U+00E9)
        let norm = IcuNormalizer::new(IcuNormMode::Nfc);
        let mut text = "e\u{0301}".to_string();
        norm.normalize(&mut text);
        assert_eq!(text, "é");
    }

    #[test]
    fn nfc_already_composed() {
        let norm = IcuNormalizer::new(IcuNormMode::Nfc);
        let mut text = "café".to_string();
        norm.normalize(&mut text);
        assert_eq!(text, "café");
    }

    #[test]
    fn nfkc_ligature() {
        let norm = IcuNormalizer::new(IcuNormMode::Nfkc);
        let mut text = "ﬁ".to_string();
        norm.normalize(&mut text);
        assert_eq!(text, "fi");
    }

    #[test]
    fn nfkc_roman_numeral() {
        let norm = IcuNormalizer::new(IcuNormMode::Nfkc);
        let mut text = "Ⅲ".to_string();
        norm.normalize(&mut text);
        assert_eq!(text, "III");
    }

    #[test]
    fn nfkc_casefold() {
        let norm = IcuNormalizer::new(IcuNormMode::NfkcCasefold);
        let mut text = "Héllo".to_string();
        norm.normalize(&mut text);
        assert_eq!(text, text.to_lowercase());
    }

    #[test]
    fn nfkc_casefold_convenience() {
        let norm = IcuNormalizer::nfkc_casefold();
        let mut text = "ABC".to_string();
        norm.normalize(&mut text);
        assert_eq!(text, "abc");
    }

    #[test]
    fn nfd_decomposes() {
        let norm = IcuNormalizer::new(IcuNormMode::Nfd);
        let mut text = "é".to_string();
        norm.normalize(&mut text);
        // NFD should decompose é into e + combining acute
        assert!(text.contains('e'));
        assert!(text.len() > 1);
    }

    #[test]
    fn ascii_passthrough() {
        let norm = IcuNormalizer::new(IcuNormMode::Nfc);
        let mut text = "hello".to_string();
        norm.normalize(&mut text);
        assert_eq!(text, "hello");
    }

    #[test]
    fn empty_string() {
        let norm = IcuNormalizer::new(IcuNormMode::Nfc);
        let mut text = String::new();
        norm.normalize(&mut text);
        assert_eq!(text, "");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// mod icu_normalization_filter — Per-token normalization
// ═══════════════════════════════════════════════════════════════════════════════

mod icu_normalization_filter {
    use super::*;

    #[test]
    fn nfc_token() {
        let filter = IcuNormalizationFilter::new(IcuNormFilterMode::Nfc);
        let result = apply_filter(&filter, "e\u{0301}");
        assert_eq!(result, "é");
    }

    #[test]
    fn nfkc_token_ligature() {
        let filter = IcuNormalizationFilter::new(IcuNormFilterMode::Nfkc);
        let result = apply_filter(&filter, "ﬁ");
        assert_eq!(result, "fi");
    }

    #[test]
    fn nfkc_casefold_token() {
        let filter = IcuNormalizationFilter::nfkc_casefold();
        let result = apply_filter(&filter, "HELLO");
        assert_eq!(result, "hello");
    }

    #[test]
    fn nfd_token() {
        let filter = IcuNormalizationFilter::new(IcuNormFilterMode::Nfd);
        let result = apply_filter(&filter, "é");
        // Should decompose
        assert_ne!(result, "é");
        assert!(result.starts_with('e'));
    }

    #[test]
    fn filter_returns_keep() {
        let filter = IcuNormalizationFilter::new(IcuNormFilterMode::Nfc);
        let mut token = make_token("hello");
        let (remove, extra) = filter.filter(&mut token);
        assert!(!remove, "filter should not request removal");
        assert!(extra.is_none(), "filter should not produce extra tokens");
    }

    #[test]
    fn ascii_passthrough() {
        let filter = IcuNormalizationFilter::new(IcuNormFilterMode::Nfkc);
        let result = apply_filter(&filter, "hello");
        assert_eq!(result, "hello");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// mod icu_folding — Unicode case folding + accent removal
// ═══════════════════════════════════════════════════════════════════════════════

mod icu_folding {
    use super::*;

    #[test]
    fn accent_removal_cafe() {
        let filter = IcuFoldingFilter::new();
        let result = apply_filter(&filter, "café");
        assert_eq!(result, "cafe");
    }

    #[test]
    fn accent_removal_resume() {
        let filter = IcuFoldingFilter::new();
        let result = apply_filter(&filter, "résumé");
        assert_eq!(result, "resume");
    }

    #[test]
    fn german_eszett() {
        let filter = IcuFoldingFilter::new();
        let result = apply_filter(&filter, "ß");
        assert_eq!(result, "ss");
    }

    #[test]
    fn uppercase_to_lowercase() {
        let filter = IcuFoldingFilter::new();
        let result = apply_filter(&filter, "HELLO");
        assert_eq!(result, "hello");
    }

    #[test]
    fn mixed_case_accent() {
        let filter = IcuFoldingFilter::new();
        let result = apply_filter(&filter, "Ü");
        assert_eq!(result, "u");
    }

    #[test]
    fn already_ascii() {
        let filter = IcuFoldingFilter::new();
        let result = apply_filter(&filter, "hello");
        assert_eq!(result, "hello");
    }

    #[test]
    fn ligature_fi() {
        let filter = IcuFoldingFilter::new();
        let result = apply_filter(&filter, "ﬁ");
        assert_eq!(result, "fi");
    }

    #[test]
    fn empty_string() {
        let filter = IcuFoldingFilter::new();
        let result = apply_filter(&filter, "");
        assert_eq!(result, "");
    }

    #[test]
    fn default_trait() {
        let filter = IcuFoldingFilter::default();
        let result = apply_filter(&filter, "café");
        assert_eq!(result, "cafe");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// mod icu_collation — Locale-aware sort key generation
// ═══════════════════════════════════════════════════════════════════════════════

mod icu_collation {
    use super::*;

    #[test]
    fn generates_sort_key_hex() {
        let filter = IcuCollationFilter::new("root");
        let result = apply_filter(&filter, "hello");
        // Sort keys are hex-encoded bytes
        assert!(!result.is_empty());
        assert!(
            result.chars().all(|c| c.is_ascii_hexdigit()),
            "sort key should be hex: {}",
            result
        );
    }

    #[test]
    fn different_inputs_different_keys() {
        let filter = IcuCollationFilter::new("root");
        let key_a = apply_filter(&filter, "apple");
        let key_b = apply_filter(&filter, "banana");
        assert_ne!(key_a, key_b);
    }

    #[test]
    fn german_locale() {
        let filter = IcuCollationFilter::german();
        let result = apply_filter(&filter, "straße");
        assert!(!result.is_empty());
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn same_input_same_key() {
        let filter = IcuCollationFilter::new("root");
        let key1 = apply_filter(&filter, "test");
        let key2 = apply_filter(&filter, "test");
        assert_eq!(key1, key2);
    }

    #[test]
    fn ordering_preserved() {
        let filter = IcuCollationFilter::new("root");
        let key_a = apply_filter(&filter, "a");
        let key_b = apply_filter(&filter, "b");
        assert!(key_a < key_b, "sort key for 'a' should be less than 'b'");
    }

    #[test]
    fn empty_string() {
        let filter = IcuCollationFilter::new("root");
        let result = apply_filter(&filter, "");
        // Empty input should still produce a sort key (terminator bytes)
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// mod icu_transform — Transliteration (extensive)
// ═══════════════════════════════════════════════════════════════════════════════

mod icu_transform {
    use super::*;

    // --- Latin-ASCII ---

    #[test]
    fn latin_ascii_cafe() {
        let filter = IcuTransformFilter::new(IcuTransformId::LatinAscii);
        assert_eq!(apply_filter(&filter, "café"), "cafe");
    }

    #[test]
    fn latin_ascii_naive() {
        let filter = IcuTransformFilter::new(IcuTransformId::LatinAscii);
        assert_eq!(apply_filter(&filter, "naïve"), "naive");
    }

    #[test]
    fn latin_ascii_uber() {
        let filter = IcuTransformFilter::new(IcuTransformId::LatinAscii);
        assert_eq!(apply_filter(&filter, "über"), "uber");
    }

    #[test]
    fn latin_ascii_resume() {
        let filter = IcuTransformFilter::new(IcuTransformId::LatinAscii);
        assert_eq!(apply_filter(&filter, "résumé"), "resume");
    }

    #[test]
    fn latin_ascii_passthrough() {
        let filter = IcuTransformFilter::new(IcuTransformId::LatinAscii);
        assert_eq!(apply_filter(&filter, "hello"), "hello");
    }

    // --- Katakana ↔ Hiragana ---

    #[test]
    fn katakana_to_hiragana() {
        let filter = IcuTransformFilter::new(IcuTransformId::KatakanaHiragana);
        assert_eq!(apply_filter(&filter, "カタカナ"), "かたかな");
    }

    #[test]
    fn katakana_to_hiragana_tokyo() {
        let filter = IcuTransformFilter::new(IcuTransformId::KatakanaHiragana);
        assert_eq!(apply_filter(&filter, "トウキョウ"), "とうきょう");
    }

    #[test]
    fn hiragana_to_katakana() {
        let filter = IcuTransformFilter::new(IcuTransformId::HiraganaKatakana);
        assert_eq!(apply_filter(&filter, "ひらがな"), "ヒラガナ");
    }

    #[test]
    fn roundtrip_katakana_hiragana() {
        let kata = IcuTransformFilter::new(IcuTransformId::KatakanaHiragana);
        let hira = IcuTransformFilter::new(IcuTransformId::HiraganaKatakana);
        let original = "カタカナ";
        let hiragana = apply_filter(&kata, original);
        let back = apply_filter(&hira, &hiragana);
        assert_eq!(back, original);
    }

    // --- Any-Latin ---

    #[test]
    fn any_latin_cyrillic() {
        let filter = IcuTransformFilter::new(IcuTransformId::AnyLatin);
        assert_eq!(apply_filter(&filter, "Москва"), "Moskva");
    }

    #[test]
    fn any_latin_katakana() {
        let filter = IcuTransformFilter::new(IcuTransformId::AnyLatin);
        let result = apply_filter(&filter, "トウキョウ");
        // Katakana → romaji should produce Latin text
        assert!(
            result.is_ascii(),
            "Any-Latin katakana should produce ASCII: {}",
            result
        );
    }

    #[test]
    fn any_latin_ascii_passthrough() {
        let filter = IcuTransformFilter::new(IcuTransformId::AnyLatin);
        assert_eq!(apply_filter(&filter, "hello"), "hello");
    }

    // --- Fullwidth ↔ Halfwidth ---

    #[test]
    fn fullwidth_to_halfwidth_letters() {
        let filter = IcuTransformFilter::new(IcuTransformId::FullwidthHalfwidth);
        assert_eq!(apply_filter(&filter, "ＡＢＣ"), "ABC");
    }

    #[test]
    fn fullwidth_to_halfwidth_digits() {
        let filter = IcuTransformFilter::new(IcuTransformId::FullwidthHalfwidth);
        assert_eq!(apply_filter(&filter, "１２３"), "123");
    }

    #[test]
    fn fullwidth_to_halfwidth_mixed() {
        let filter = IcuTransformFilter::new(IcuTransformId::FullwidthHalfwidth);
        assert_eq!(apply_filter(&filter, "Ｈｅｌｌｏ"), "Hello");
    }

    #[test]
    fn halfwidth_to_fullwidth_letters() {
        let filter = IcuTransformFilter::new(IcuTransformId::HalfwidthFullwidth);
        assert_eq!(apply_filter(&filter, "ABC"), "ＡＢＣ");
    }

    #[test]
    fn halfwidth_to_fullwidth_digits() {
        let filter = IcuTransformFilter::new(IcuTransformId::HalfwidthFullwidth);
        assert_eq!(apply_filter(&filter, "123"), "１２３");
    }

    #[test]
    fn roundtrip_fullwidth_halfwidth() {
        let fw2hw = IcuTransformFilter::new(IcuTransformId::FullwidthHalfwidth);
        let hw2fw = IcuTransformFilter::new(IcuTransformId::HalfwidthFullwidth);
        let original = "ＡＢＣ";
        let half = apply_filter(&fw2hw, original);
        let back = apply_filter(&hw2fw, &half);
        assert_eq!(back, original);
    }

    // --- NFD / NFKC normalization transforms ---

    #[test]
    fn nfd_decomposes() {
        let filter = IcuTransformFilter::new(IcuTransformId::Nfd);
        let result = apply_filter(&filter, "é");
        // NFD decomposes é into e + combining accent
        assert!(result.starts_with('e'));
        assert!(result.len() > 1);
    }

    #[test]
    fn nfkc_normalizes_ligature() {
        let filter = IcuTransformFilter::new(IcuTransformId::Nfkc);
        assert_eq!(apply_filter(&filter, "ﬁ"), "fi");
    }

    #[test]
    fn nfkc_normalizes_roman_numeral() {
        let filter = IcuTransformFilter::new(IcuTransformId::Nfkc);
        assert_eq!(apply_filter(&filter, "Ⅲ"), "III");
    }

    #[test]
    fn nfc_composes() {
        let filter = IcuTransformFilter::new(IcuTransformId::Nfc);
        let result = apply_filter(&filter, "e\u{0301}");
        assert_eq!(result, "é");
    }

    // --- Custom rules ---

    #[test]
    fn custom_rules_simple() {
        let rules = vec![
            ("foo".to_string(), "bar".to_string()),
            ("baz".to_string(), "qux".to_string()),
        ];
        let filter = IcuTransformFilter::new(IcuTransformId::Custom(rules));
        assert_eq!(apply_filter(&filter, "foo"), "bar");
        assert_eq!(apply_filter(&filter, "baz"), "qux");
        assert_eq!(apply_filter(&filter, "hello"), "hello");
    }

    #[test]
    fn custom_rules_chained() {
        let rules = vec![("alpha".to_string(), "beta".to_string())];
        let filter = IcuTransformFilter::new(IcuTransformId::Custom(rules));
        assert_eq!(apply_filter(&filter, "alpha-test"), "beta-test");
    }

    // --- from_id() parsing ---

    #[test]
    fn from_id_latin_ascii() {
        let f = IcuTransformFilter::from_id("Latin-ASCII").unwrap();
        assert_eq!(apply_filter(&f, "café"), "cafe");
    }

    #[test]
    fn from_id_any_latin() {
        let f = IcuTransformFilter::from_id("Any-Latin").unwrap();
        assert_eq!(apply_filter(&f, "Москва"), "Moskva");
    }

    #[test]
    fn from_id_katakana_hiragana() {
        let f = IcuTransformFilter::from_id("Katakana-Hiragana").unwrap();
        assert_eq!(apply_filter(&f, "カタカナ"), "かたかな");
    }

    #[test]
    fn from_id_hiragana_katakana() {
        let f = IcuTransformFilter::from_id("Hiragana-Katakana").unwrap();
        assert_eq!(apply_filter(&f, "ひらがな"), "ヒラガナ");
    }

    #[test]
    fn from_id_fullwidth_halfwidth() {
        let f = IcuTransformFilter::from_id("Fullwidth-Halfwidth").unwrap();
        assert_eq!(apply_filter(&f, "ＡＢＣ"), "ABC");
    }

    #[test]
    fn from_id_nfd() {
        let f = IcuTransformFilter::from_id("NFD").unwrap();
        let result = apply_filter(&f, "é");
        assert!(result.starts_with('e') && result.len() > 1);
    }

    #[test]
    fn from_id_nfkc() {
        let f = IcuTransformFilter::from_id("NFKC").unwrap();
        assert_eq!(apply_filter(&f, "ﬁ"), "fi");
    }

    #[test]
    fn from_id_unknown_returns_none() {
        assert!(IcuTransformFilter::from_id("Unknown-Transform").is_none());
    }

    #[test]
    fn from_id_case_insensitive() {
        assert!(IcuTransformFilter::from_id("latin-ascii").is_some());
        assert!(IcuTransformFilter::from_id("LATIN-ASCII").is_some());
        assert!(IcuTransformFilter::from_id("Latin-Ascii").is_some());
    }

    // --- Any-Latin-ASCII pipeline ---

    #[test]
    fn any_latin_ascii_pipeline() {
        let filter = IcuTransformFilter::new(IcuTransformId::AnyLatinAscii);
        let result = apply_filter(&filter, "Москва");
        assert!(
            result.is_ascii(),
            "AnyLatinAscii should produce ASCII: {}",
            result
        );
    }

    // --- Identity for ASCII ---

    #[test]
    fn identity_ascii_latin_ascii() {
        let filter = IcuTransformFilter::new(IcuTransformId::LatinAscii);
        assert_eq!(apply_filter(&filter, "simple"), "simple");
    }

    #[test]
    fn identity_ascii_nfc() {
        let filter = IcuTransformFilter::new(IcuTransformId::Nfc);
        assert_eq!(apply_filter(&filter, "ascii"), "ascii");
    }

    #[test]
    fn identity_ascii_nfkc() {
        let filter = IcuTransformFilter::new(IcuTransformId::Nfkc);
        assert_eq!(apply_filter(&filter, "ascii"), "ascii");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// mod icu_register — Registration into AnalysisFactory
// ═══════════════════════════════════════════════════════════════════════════════

mod icu_register {
    use super::*;

    #[test]
    fn register_all_tokenizer() {
        let mut factory = AnalysisFactory::new();
        register_all(&mut factory);
        assert!(
            factory.get_tokenizer("icu_tokenizer").is_some(),
            "icu_tokenizer should be registered"
        );
    }

    #[test]
    fn register_all_folding() {
        let mut factory = AnalysisFactory::new();
        register_all(&mut factory);
        assert!(
            factory.get_token_filter("icu_folding").is_some(),
            "icu_folding should be registered"
        );
    }

    #[test]
    fn register_all_normalizer() {
        let mut factory = AnalysisFactory::new();
        register_all(&mut factory);
        assert!(
            factory.get_token_filter("icu_normalizer").is_some(),
            "icu_normalizer should be registered"
        );
    }

    #[test]
    fn register_all_collation() {
        let mut factory = AnalysisFactory::new();
        register_all(&mut factory);
        assert!(
            factory.get_token_filter("icu_collation").is_some(),
            "icu_collation should be registered"
        );
    }

    #[test]
    fn register_all_transform() {
        let mut factory = AnalysisFactory::new();
        register_all(&mut factory);
        assert!(
            factory.get_token_filter("icu_transform").is_some(),
            "icu_transform should be registered"
        );
    }

    #[test]
    fn registered_tokenizer_works() {
        let mut factory = AnalysisFactory::new();
        register_all(&mut factory);
        let tok = factory.get_tokenizer("icu_tokenizer").unwrap();
        let tokens = tok.tokenize("Hello world");
        assert_eq!(
            tokens.iter().map(|t| t.term.as_ref()).collect::<Vec<_>>(),
            vec!["Hello", "world"]
        );
    }

    #[test]
    fn registered_folding_filter_works() {
        let mut factory = AnalysisFactory::new();
        register_all(&mut factory);
        let f = factory.get_token_filter("icu_folding").unwrap();
        let mut token = make_token("café");
        f.filter(&mut token);
        assert_eq!(token.term.as_ref(), "cafe");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// mod icu_pipeline — Integration: full analysis pipeline
// ═══════════════════════════════════════════════════════════════════════════════

mod icu_pipeline {
    use super::*;

    #[test]
    fn tokenize_then_fold() {
        let tok = IcuTokenizer::new();
        let fold = IcuFoldingFilter::new();

        let tokens = tok.tokenize("Résumé café");
        let folded: Vec<String> = tokens
            .into_iter()
            .map(|mut t| {
                fold.filter(&mut t);
                t.term.into_owned()
            })
            .collect();

        assert_eq!(folded, vec!["resume", "cafe"]);
    }

    #[test]
    fn normalize_then_tokenize() {
        let norm = IcuNormalizer::nfkc_casefold();
        let tok = IcuTokenizer::new();

        let mut text = "HELLO WORLD".to_string();
        norm.normalize(&mut text);
        let tokens = tok.tokenize(&text);
        let t = terms(&tokens);

        assert_eq!(t, vec!["hello", "world"]);
    }

    #[test]
    fn tokenize_then_normalize_filter() {
        let tok = IcuTokenizer::new();
        let nf = IcuNormalizationFilter::nfkc_casefold();

        let tokens = tok.tokenize("HELLO WORLD");
        let normalized: Vec<String> = tokens
            .into_iter()
            .map(|mut t| {
                nf.filter(&mut t);
                t.term.into_owned()
            })
            .collect();

        assert_eq!(normalized, vec!["hello", "world"]);
    }

    #[test]
    fn full_pipeline_normalize_tokenize_fold() {
        let norm = IcuNormalizer::new(IcuNormMode::Nfkc);
        let tok = IcuTokenizer::new();
        let fold = IcuFoldingFilter::new();

        // Start with text containing ligature and accent
        let mut text = "ﬁrst café".to_string();
        norm.normalize(&mut text);
        // After NFKC: "first café" (ligature expanded)

        let tokens = tok.tokenize(&text);
        let result: Vec<String> = tokens
            .into_iter()
            .map(|mut t| {
                fold.filter(&mut t);
                t.term.into_owned()
            })
            .collect();

        assert_eq!(result, vec!["first", "cafe"]);
    }

    #[test]
    fn pipeline_with_transform() {
        let tok = IcuTokenizer::new();
        let transform = IcuTransformFilter::new(IcuTransformId::LatinAscii);

        let tokens = tok.tokenize("résumé naïve");
        let transformed: Vec<String> = tokens
            .into_iter()
            .map(|mut t| {
                transform.filter(&mut t);
                t.term.into_owned()
            })
            .collect();

        assert_eq!(transformed, vec!["resume", "naive"]);
    }

    #[test]
    fn pipeline_cyrillic_to_ascii() {
        let tok = IcuTokenizer::new();
        let transform = IcuTransformFilter::new(IcuTransformId::AnyLatinAscii);

        let tokens = tok.tokenize("Москва");
        let result: Vec<String> = tokens
            .into_iter()
            .map(|mut t| {
                transform.filter(&mut t);
                t.term.into_owned()
            })
            .collect();

        assert!(!result.is_empty());
        assert!(
            result[0].is_ascii(),
            "Cyrillic should be transliterated to ASCII: {}",
            result[0]
        );
    }

    #[test]
    fn pipeline_collation_after_fold() {
        let fold = IcuFoldingFilter::new();
        let coll = IcuCollationFilter::new("root");

        let mut token = make_token("café");
        fold.filter(&mut token);
        assert_eq!(token.term.as_ref(), "cafe");

        coll.filter(&mut token);
        // After collation, term should be hex sort key
        assert!(
            token.term.as_ref().chars().all(|c| c.is_ascii_hexdigit()),
            "expected hex sort key, got: {}",
            token.term
        );
    }
}
