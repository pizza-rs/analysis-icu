<div align="center">

# 🌐 pizza-analysis-icu

**ICU-based text analysis plugin for [INFINI Pizza](https://pizza.rs)**

[![Crate](https://img.shields.io/badge/crate-pizza--analysis--icu-blue)](https://github.com/pizza-rs/analysis-icu)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)

</div>

---

## Overview

Provides Unicode-standard text analysis based on ICU algorithms, including
UAX#29 word-break tokenization, Unicode normalization (NFC/NFD/NFKC/NFKD),
case folding, and locale-aware collation. Essential for multilingual search
where consistent Unicode handling is required.

## Components

| Type | Name | Description |
|:-----|:-----|:------------|
| Tokenizer | `icu_tokenizer` | UAX#29 word-break segmentation for all Unicode scripts |
| TokenFilter | `icu_folding` | Unicode case folding + accent removal (NFKC_Casefold) |
| TokenFilter | `icu_normalizer` | Configurable Unicode normalization (NFC/NFD/NFKC/NFKD/NFKC_Casefold) |
| TokenFilter | `icu_collation` | Locale-aware sort key generation for collation-based sorting |

### Normalization Modes

| Mode | Description |
|:-----|:------------|
| `Nfc` | Canonical composition (NFC) |
| `Nfd` | Canonical decomposition (NFD) |
| `Nfkc` | Compatibility composition (NFKC) |
| `Nfkd` | Compatibility decomposition (NFKD) |
| `NfkcCasefold` | NFKC + Unicode case folding (default) |

### Why ICU?

- **UAX#29 tokenizer** correctly splits Thai, Lao, Khmer, Myanmar (no spaces between words)
- **Folding** reduces diacritics (café → cafe) and fullwidth → ASCII (Ａ → A)
- **Collation** handles locale-specific sort order (German: ä sorts with a; Swedish: ä sorts after z)

## Example

```rust
use pizza_engine::analysis::Tokenizer;
use pizza_analysis_icu::IcuTokenizer;

let tk = IcuTokenizer::new();
let tokens = tk.tokenize("สวัสดีครับ"); // Thai - no spaces
// UAX#29 splits correctly: ["สวัสดี", "ครับ"]
```

## Installation

```toml
[dependencies]
pizza-analysis-icu = "0.1"
```

Or via `pizza-analysis-all`:

```toml
[dependencies]
pizza-analysis-all = { version = "0.1", features = ["icu"] }
```

## License

MIT

---

<div align="center">
<sub>Part of the <a href="https://pizza.rs">INFINI Pizza</a> ecosystem</sub>
</div>
