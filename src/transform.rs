//! ICU Transliteration / Transform filter.
//!
//! Provides a general-purpose text transformation filter using rule-based
//! transliteration. Supports common transforms like:
//! - Script conversion (Latin → Cyrillic, Katakana → Hiragana, etc.)
//! - Accent/diacritic removal (NFD + strip combining marks)
//! - Custom rule-based rewriting
//!
//! Equivalent to Elasticsearch's `icu_transform` token filter.
//!
//! Note: ICU4X v2 does not yet ship a full `Transliterator` API. This
//! implementation provides the most commonly used built-in transforms via
//! direct Unicode algorithms. For arbitrary ICU transform IDs, users should
//! add custom rules.

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;

use icu_normalizer::ComposingNormalizer;
use icu_normalizer::DecomposingNormalizer;

use pizza_engine::analysis::{Token, TokenFilter};

/// Built-in transform identifiers modeled after ICU's transform IDs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IcuTransformId {
    /// Any-Latin: convert any script to Latin (via NFD + strip non-Latin + NFC)
    AnyLatin,
    /// Any-NFD: canonical decomposition
    Nfd,
    /// Any-NFC: canonical composition
    Nfc,
    /// Any-NFKD: compatibility decomposition
    Nfkd,
    /// Any-NFKC: compatibility composition
    Nfkc,
    /// Latin-ASCII: remove accents from Latin text (NFD + strip combining marks)
    LatinAscii,
    /// Katakana-Hiragana: convert katakana to hiragana
    KatakanaHiragana,
    /// Hiragana-Katakana: convert hiragana to katakana
    HiraganaKatakana,
    /// Fullwidth-Halfwidth: convert fullwidth to halfwidth
    FullwidthHalfwidth,
    /// Halfwidth-Fullwidth: convert halfwidth to fullwidth
    HalfwidthFullwidth,
    /// Any-Latin followed by Latin-ASCII (common pipeline for search normalization)
    AnyLatinAscii,
    /// Custom rule-based transform (simplified regex-like replacement)
    Custom(Vec<(String, String)>),
}

/// General-purpose ICU text transformation filter.
///
/// Applies a named transform to each token. This is equivalent to
/// Elasticsearch's `icu_transform` token filter.
#[derive(Clone, Debug)]
pub struct IcuTransformFilter {
    transform: IcuTransformId,
}

impl IcuTransformFilter {
    /// Create a transform filter with the specified transform.
    pub fn new(transform: IcuTransformId) -> Self {
        Self { transform }
    }

    /// Parse a transform ID string (case-insensitive) into the enum.
    pub fn from_id(id: &str) -> Option<Self> {
        let lower = id.to_lowercase().replace(['-', '_'], "");
        let transform = match lower.as_str() {
            "anylatin" => IcuTransformId::AnyLatin,
            "nfd" | "anynfd" => IcuTransformId::Nfd,
            "nfc" | "anynfc" => IcuTransformId::Nfc,
            "nfkd" | "anynfkd" => IcuTransformId::Nfkd,
            "nfkc" | "anynfkc" => IcuTransformId::Nfkc,
            "latinascii" => IcuTransformId::LatinAscii,
            "katakanahiragana" => IcuTransformId::KatakanaHiragana,
            "hiraganakat" | "hiraganakatakana" => IcuTransformId::HiraganaKatakana,
            "fullwidthhalfwidth" => IcuTransformId::FullwidthHalfwidth,
            "halfwidthfullwidth" => IcuTransformId::HalfwidthFullwidth,
            "anylatinascii" | "anylatinlatinascii" => IcuTransformId::AnyLatinAscii,
            _ => return None,
        };
        Some(Self { transform })
    }

    fn apply_transform(&self, input: &str) -> String {
        match &self.transform {
            IcuTransformId::Nfd => {
                let normalizer = DecomposingNormalizer::new_nfd();
                normalizer.normalize(input)
            }
            IcuTransformId::Nfc => {
                let normalizer = ComposingNormalizer::new_nfc();
                normalizer.normalize(input)
            }
            IcuTransformId::Nfkd => {
                let normalizer = DecomposingNormalizer::new_nfkd();
                normalizer.normalize(input)
            }
            IcuTransformId::Nfkc => {
                let normalizer = ComposingNormalizer::new_nfkc();
                normalizer.normalize(input)
            }
            IcuTransformId::LatinAscii => {
                // NFD decompose then strip combining marks
                let normalizer = DecomposingNormalizer::new_nfd();
                let decomposed = normalizer.normalize(input);
                strip_combining_marks(&decomposed)
            }
            IcuTransformId::AnyLatin => {
                // Best-effort: handle Katakana/Hiragana → romaji, Cyrillic → Latin
                // then strip combining marks
                let mut s = String::with_capacity(input.len());
                for ch in input.chars() {
                    if is_katakana(ch) {
                        s.push_str(&katakana_char_to_latin(ch));
                    } else if is_hiragana(ch) {
                        let kata = hiragana_to_katakana_char(ch);
                        s.push_str(&katakana_char_to_latin(kata));
                    } else if is_cyrillic(ch) {
                        s.push_str(&cyrillic_to_latin(ch));
                    } else {
                        s.push(ch);
                    }
                }
                // Then NFD + strip combining marks for any remaining accents
                let normalizer = DecomposingNormalizer::new_nfd();
                let decomposed = normalizer.normalize(&s);
                strip_combining_marks(&decomposed)
            }
            IcuTransformId::AnyLatinAscii => {
                // AnyLatin then LatinAscii
                let latin = IcuTransformFilter::new(IcuTransformId::AnyLatin).apply_transform(input);
                IcuTransformFilter::new(IcuTransformId::LatinAscii).apply_transform(&latin)
            }
            IcuTransformId::KatakanaHiragana => {
                input.chars().map(|ch| {
                    if is_katakana(ch) {
                        katakana_to_hiragana_char(ch)
                    } else {
                        ch
                    }
                }).collect()
            }
            IcuTransformId::HiraganaKatakana => {
                input.chars().map(|ch| {
                    if is_hiragana(ch) {
                        hiragana_to_katakana_char(ch)
                    } else {
                        ch
                    }
                }).collect()
            }
            IcuTransformId::FullwidthHalfwidth => {
                input.chars().map(fullwidth_to_halfwidth).collect()
            }
            IcuTransformId::HalfwidthFullwidth => {
                input.chars().map(halfwidth_to_fullwidth).collect()
            }
            IcuTransformId::Custom(rules) => {
                let mut result = input.to_string();
                for (from, to) in rules {
                    result = result.replace(from.as_str(), to.as_str());
                }
                result
            }
        }
    }
}

impl TokenFilter for IcuTransformFilter {
    fn filter<'a>(&self, token: &mut Token<'a>) -> (bool, Option<Vec<Token<'a>>>) {
        let input = token.term.as_ref();
        let transformed = self.apply_transform(input);
        if transformed != input {
            token.term = Cow::Owned(transformed);
        }
        (false, None)
    }
}

// ─── Unicode Helpers ───────────────────────────────────────────────────────

fn strip_combining_marks(s: &str) -> String {
    s.chars()
        .filter(|&ch| {
            let c = ch as u32;
            !((0x0300..=0x036F).contains(&c)     // Combining Diacritical Marks
                || (0x1AB0..=0x1AFF).contains(&c) // Extended
                || (0x1DC0..=0x1DFF).contains(&c) // Supplement
                || (0x20D0..=0x20FF).contains(&c) // For Symbols
                || (0xFE20..=0xFE2F).contains(&c)) // Half Marks
        })
        .collect()
}

fn is_katakana(ch: char) -> bool {
    ('\u{30A0}'..='\u{30FF}').contains(&ch)
}

fn is_hiragana(ch: char) -> bool {
    ('\u{3040}'..='\u{309F}').contains(&ch)
}

fn is_cyrillic(ch: char) -> bool {
    ('\u{0400}'..='\u{04FF}').contains(&ch)
}

fn katakana_to_hiragana_char(ch: char) -> char {
    if ('\u{30A1}'..='\u{30F6}').contains(&ch) {
        char::from_u32(ch as u32 - 0x60).unwrap_or(ch)
    } else {
        ch
    }
}

fn hiragana_to_katakana_char(ch: char) -> char {
    if ('\u{3041}'..='\u{3096}').contains(&ch) {
        char::from_u32(ch as u32 + 0x60).unwrap_or(ch)
    } else {
        ch
    }
}

/// Best-effort single katakana → Latin transliteration.
fn katakana_char_to_latin(ch: char) -> String {
    match ch {
        'ア' => "a".into(), 'イ' => "i".into(), 'ウ' => "u".into(),
        'エ' => "e".into(), 'オ' => "o".into(),
        'カ' => "ka".into(), 'キ' => "ki".into(), 'ク' => "ku".into(),
        'ケ' => "ke".into(), 'コ' => "ko".into(),
        'サ' => "sa".into(), 'シ' => "shi".into(), 'ス' => "su".into(),
        'セ' => "se".into(), 'ソ' => "so".into(),
        'タ' => "ta".into(), 'チ' => "chi".into(), 'ツ' => "tsu".into(),
        'テ' => "te".into(), 'ト' => "to".into(),
        'ナ' => "na".into(), 'ニ' => "ni".into(), 'ヌ' => "nu".into(),
        'ネ' => "ne".into(), 'ノ' => "no".into(),
        'ハ' => "ha".into(), 'ヒ' => "hi".into(), 'フ' => "fu".into(),
        'ヘ' => "he".into(), 'ホ' => "ho".into(),
        'マ' => "ma".into(), 'ミ' => "mi".into(), 'ム' => "mu".into(),
        'メ' => "me".into(), 'モ' => "mo".into(),
        'ヤ' => "ya".into(), 'ユ' => "yu".into(), 'ヨ' => "yo".into(),
        'ラ' => "ra".into(), 'リ' => "ri".into(), 'ル' => "ru".into(),
        'レ' => "re".into(), 'ロ' => "ro".into(),
        'ワ' => "wa".into(), 'ヲ' => "wo".into(), 'ン' => "n".into(),
        'ガ' => "ga".into(), 'ギ' => "gi".into(), 'グ' => "gu".into(),
        'ゲ' => "ge".into(), 'ゴ' => "go".into(),
        'ザ' => "za".into(), 'ジ' => "ji".into(), 'ズ' => "zu".into(),
        'ゼ' => "ze".into(), 'ゾ' => "zo".into(),
        'ダ' => "da".into(), 'ヂ' => "di".into(), 'ヅ' => "du".into(),
        'デ' => "de".into(), 'ド' => "do".into(),
        'バ' => "ba".into(), 'ビ' => "bi".into(), 'ブ' => "bu".into(),
        'ベ' => "be".into(), 'ボ' => "bo".into(),
        'パ' => "pa".into(), 'ピ' => "pi".into(), 'プ' => "pu".into(),
        'ペ' => "pe".into(), 'ポ' => "po".into(),
        'ー' => "".into(),   // long vowel mark
        'ッ' => "".into(),   // small tsu (gemination)
        'ァ' => "a".into(), 'ィ' => "i".into(), 'ゥ' => "u".into(),
        'ェ' => "e".into(), 'ォ' => "o".into(),
        _ => {
            let mut s = String::new();
            s.push(ch);
            s
        }
    }
}

/// Best-effort Cyrillic → Latin transliteration (ISO 9 / scholarly).
fn cyrillic_to_latin(ch: char) -> String {
    match ch {
        'А' => "A".into(), 'а' => "a".into(),
        'Б' => "B".into(), 'б' => "b".into(),
        'В' => "V".into(), 'в' => "v".into(),
        'Г' => "G".into(), 'г' => "g".into(),
        'Д' => "D".into(), 'д' => "d".into(),
        'Е' => "E".into(), 'е' => "e".into(),
        'Ё' => "Yo".into(), 'ё' => "yo".into(),
        'Ж' => "Zh".into(), 'ж' => "zh".into(),
        'З' => "Z".into(), 'з' => "z".into(),
        'И' => "I".into(), 'и' => "i".into(),
        'Й' => "J".into(), 'й' => "j".into(),
        'К' => "K".into(), 'к' => "k".into(),
        'Л' => "L".into(), 'л' => "l".into(),
        'М' => "M".into(), 'м' => "m".into(),
        'Н' => "N".into(), 'н' => "n".into(),
        'О' => "O".into(), 'о' => "o".into(),
        'П' => "P".into(), 'п' => "p".into(),
        'Р' => "R".into(), 'р' => "r".into(),
        'С' => "S".into(), 'с' => "s".into(),
        'Т' => "T".into(), 'т' => "t".into(),
        'У' => "U".into(), 'у' => "u".into(),
        'Ф' => "F".into(), 'ф' => "f".into(),
        'Х' => "Kh".into(), 'х' => "kh".into(),
        'Ц' => "Ts".into(), 'ц' => "ts".into(),
        'Ч' => "Ch".into(), 'ч' => "ch".into(),
        'Ш' => "Sh".into(), 'ш' => "sh".into(),
        'Щ' => "Shch".into(), 'щ' => "shch".into(),
        'Ъ' => "".into(), 'ъ' => "".into(),     // hard sign
        'Ы' => "Y".into(), 'ы' => "y".into(),
        'Ь' => "".into(), 'ь' => "".into(),     // soft sign
        'Э' => "E".into(), 'э' => "e".into(),
        'Ю' => "Yu".into(), 'ю' => "yu".into(),
        'Я' => "Ya".into(), 'я' => "ya".into(),
        _ => {
            let mut s = String::new();
            s.push(ch);
            s
        }
    }
}

/// Fullwidth → Halfwidth conversion (U+FF01..U+FF5E → U+0021..U+007E).
fn fullwidth_to_halfwidth(ch: char) -> char {
    let c = ch as u32;
    if (0xFF01..=0xFF5E).contains(&c) {
        char::from_u32(c - 0xFEE0).unwrap_or(ch)
    } else if ch == '\u{3000}' {
        ' ' // ideographic space → ASCII space
    } else {
        ch
    }
}

/// Halfwidth → Fullwidth conversion (U+0021..U+007E → U+FF01..U+FF5E).
fn halfwidth_to_fullwidth(ch: char) -> char {
    let c = ch as u32;
    if (0x0021..=0x007E).contains(&c) {
        char::from_u32(c + 0xFEE0).unwrap_or(ch)
    } else if ch == ' ' {
        '\u{3000}' // ASCII space → ideographic space
    } else {
        ch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(filter: &IcuTransformFilter, input: &str, expected: &str) {
        let mut token = Token::new(input, 0, input.len() as u32, 0);
        filter.filter(&mut token);
        assert_eq!(
            token.term.as_ref(), expected,
            "transform({:?}) = {:?}, expected {:?}",
            input, token.term.as_ref(), expected
        );
    }

    #[test]
    fn test_latin_ascii_removes_accents() {
        let f = IcuTransformFilter::new(IcuTransformId::LatinAscii);
        check(&f, "café", "cafe");
        check(&f, "naïve", "naive");
        check(&f, "résumé", "resume");
        check(&f, "über", "uber");
    }

    #[test]
    fn test_nfkc_normalization() {
        let f = IcuTransformFilter::new(IcuTransformId::Nfkc);
        check(&f, "ﬁ", "fi"); // fi ligature → fi
        check(&f, "Ⅲ", "III"); // Roman numeral → III
    }

    #[test]
    fn test_katakana_to_hiragana() {
        let f = IcuTransformFilter::new(IcuTransformId::KatakanaHiragana);
        check(&f, "カタカナ", "かたかな");
        check(&f, "トウキョウ", "とうきょう");
    }

    #[test]
    fn test_hiragana_to_katakana() {
        let f = IcuTransformFilter::new(IcuTransformId::HiraganaKatakana);
        check(&f, "ひらがな", "ヒラガナ");
    }

    #[test]
    fn test_fullwidth_to_halfwidth() {
        let f = IcuTransformFilter::new(IcuTransformId::FullwidthHalfwidth);
        check(&f, "ＡＢＣ", "ABC");
        check(&f, "１２３", "123");
        check(&f, "Ｈｅｌｌｏ", "Hello");
    }

    #[test]
    fn test_halfwidth_to_fullwidth() {
        let f = IcuTransformFilter::new(IcuTransformId::HalfwidthFullwidth);
        check(&f, "ABC", "ＡＢＣ");
        check(&f, "123", "１２３");
    }

    #[test]
    fn test_cyrillic_to_latin() {
        let f = IcuTransformFilter::new(IcuTransformId::AnyLatin);
        check(&f, "Москва", "Moskva");
        check(&f, "привет", "privet");
    }

    #[test]
    fn test_katakana_to_latin() {
        let f = IcuTransformFilter::new(IcuTransformId::AnyLatin);
        check(&f, "トウキョウ", "toukiyou");
    }

    #[test]
    fn test_from_id_parsing() {
        assert!(IcuTransformFilter::from_id("Latin-ASCII").is_some());
        assert!(IcuTransformFilter::from_id("Any-Latin").is_some());
        assert!(IcuTransformFilter::from_id("Katakana-Hiragana").is_some());
        assert!(IcuTransformFilter::from_id("nfkc").is_some());
        assert!(IcuTransformFilter::from_id("nonexistent").is_none());
    }

    #[test]
    fn test_custom_rules() {
        let rules = vec![
            ("oe".to_string(), "ö".to_string()),
            ("ue".to_string(), "ü".to_string()),
        ];
        let f = IcuTransformFilter::new(IcuTransformId::Custom(rules));
        check(&f, "Goethe", "Göthe");
        check(&f, "Mueller", "Müller");
    }

    #[test]
    fn test_identity_for_ascii() {
        let f = IcuTransformFilter::new(IcuTransformId::LatinAscii);
        check(&f, "hello", "hello");
        check(&f, "world123", "world123");
    }

    #[test]
    fn test_nfd_decomposition() {
        let f = IcuTransformFilter::new(IcuTransformId::Nfd);
        // é (U+00E9) decomposes to e + U+0301
        let mut token = Token::new("é", 0, 2, 0);
        f.filter(&mut token);
        let chars: Vec<char> = token.term.chars().collect();
        assert_eq!(chars.len(), 2);
        assert_eq!(chars[0], 'e');
        assert_eq!(chars[1], '\u{0301}');
    }
}
