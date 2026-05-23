# pizza-analysis-icu

ICU-based Unicode text analysis for the [Pizza](https://pizza.rs) search engine. Provides Unicode-standard tokenization, normalization, case folding, and collation using the [ICU4X](https://github.com/unicode-org/icu4x) library.

## Components

| Name | Type | Description |
|------|------|-------------|
| `icu_tokenizer` | Tokenizer | Unicode word segmentation following UAX#29 rules |
| `icu_normalizer` | Token Filter | NFKC_Casefold normalization |
| `icu_folding` | Token Filter | Unicode case folding (accent removal + lowercasing) |
| `icu_collation` | Token Filter | Locale-aware sort key generation for proper ordering |

## Usage

### ICU Tokenizer

Segments text using Unicode UAX#29 word boundary rules. Handles complex scripts (Thai, Khmer, Lao, Myanmar, CJK) correctly without language-specific dictionaries.

```json
{
  "analyzer": {
    "type": "custom",
    "tokenizer": "icu_tokenizer",
    "filter": ["icu_folding"]
  }
}
```

### ICU Folding

Converts characters to their ASCII equivalents where possible, while removing diacritics and performing case folding. Based on Unicode's NFKC_Casefold with additional accent stripping.

```json
{
  "analyzer": {
    "type": "custom",
    "tokenizer": "standard",
    "filter": ["icu_folding"]
  }
}
```

**Example**: `Ménü` → `menu`, `Ⅷ` → `viii`

### ICU Normalizer

Applies Unicode NFKC_Casefold normalization to tokens. This decomposes compatibility characters and applies case folding.

```json
{
  "filter": ["icu_normalizer"]
}
```

**Example**: `ﬁ` → `fi`, `Ω` → `ω`

### ICU Collation

Generates binary sort keys for locale-aware ordering. Useful for case-insensitive, accent-insensitive sorting.

```json
{
  "filter": ["icu_collation"]
}
```

## When to Use

- **Multi-language indexes** — `icu_tokenizer` handles all Unicode scripts via UAX#29
- **Accent-insensitive search** — `icu_folding` strips diacritics while preserving base characters
- **Complex scripts** — Thai, Khmer, Lao, Myanmar where whitespace doesn't separate words
- **Locale-aware sorting** — `icu_collation` produces correct sort ordering per locale

## Data Sources

This crate uses [ICU4X](https://github.com/unicode-org/icu4x) v2, which embeds official Unicode CLDR/ICU data. No external data files are needed.

## Dependencies

- `icu_normalizer` 2.x — Unicode normalization
- `icu_segmenter` 2.x — Word/sentence segmentation
- `icu_collator` 2.x — Collation sort keys
- `icu_casemap` 2.x — Case mapping/folding

## License

Apache-2.0
