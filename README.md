# yada-gd-look_rs

A fast, zero-bloat Mandarin Chinese companion & Swiss army knife for **GoldenDict-ng**, written in Rust.

## Philosophy

* **Zero Bloat (`yada` + `mmap`):** No 8 GiB Python/PyTorch environments or sluggish background daemons. Dictionaries are compiled ahead of time into memory-mapped Double-Array tries (`yada`), delivering microsecond segmentation and ~0 ms startup latency.
* **Cross-Strait & Classical Aware:** Clean separation between Mainland PRC (`cn`) and Taiwan (`tw`) pronunciation standards, Simplified/Traditional handling, and a dedicated **mono-character mode** for reading Classical Chinese (*Wenyanwen*) without modern word-segmentation interference.
* **Reproducible:** Declaratively built and packaged with **Nix flakes**.

---

## Features
* **Dictionary-Backed Sentence Segmentation:** Segments full sentences against your compiled `yada` trie so every clickable token matches real dictionary headwords.
* **Clean CN / TW Pinyin Standards:** Separate `pinyin.bin` and `pinyin_tw.bin` builds parsed from CC-CEDICT—keeping Mainland readings unpolluted by Taiwan variants (`和` -> `hé · hè · hú · huó · huò`, `垃圾` -> `lā jī`) while preserving dedicated Taiwan pronunciation ordering (`hàn`, `lè sè`).
---

## Quick Start 

### 1. Build Dictionary & Pinyin `.bin` Files (`dict.da`)

Compile a source dictionary (`.u8`, `.txt`, `.tsv`) into `dict.da` and Pinyin binaries:

```bash
# Default build
yada-gd-look_rs --build --input cedict_ts.u8

# Custom output/dictionary directory & Taiwan Pinyin standard
yada-gd-look_rs --build --input cedict_ts.u8 --dict-dir ~/.local/share/yada-gd-look --pinyin-std tw
```

### 2. Debug Segmentation & Pinyin Output

Inspect how a sentence is segmented and mapped in your terminal:

```bash
yada-gd-look_rs --debug --word "和" --sentence "垃圾和"
```

---




### Generate Dictionary & Pinyin Binaries
Build separate `.bin` payloads for Mainland PRC (`cn`) or Taiwan (`tw`) standards:

```bash
# Mainland PRC standard
yada-gd-look_rs --build --pinyin-std cn

# Taiwan standard
yada-gd-look_rs --build --cedict --pinyin-std tw
```

### Debug Mode
Inspect segmentation and pronunciation ordering directly in your terminal:
```bash
yada-gd-look_rs --build --debug "垃圾和"
```

---

## GoldenDict-ng Integration

Add `yada-gd-look_rs` under **Edit → Dictionaries → Programs** (Type: `Html`) in GoldenDict-ng:

```bash
yada-gd-look_rs --word "%GDWORD%" --sentence "%GDSEARCH%"
```

* **Modern / Segmented Reading:** Splits sentences into clickable multi-character words with instant Pinyin hover tooltips.
* **Mono-Character Reading:** Pass `--mono` to inspect every character individually. (Handy for Classical Chinese)
---


### All CLI Flags

| Flag | Description |
| :--- | :--- |
| `-w, --word <WORD>` | Target word (`%GDWORD%` in GoldenDict-ng) |
| `-s, --sentence <SENTENCE>` | Full sentence context (`%GDSEARCH%` in GoldenDict-ng) |
| `--pinyin` | Show tone-marked Pinyin above Chinese words using `<ruby>` tags |
| `--subwords` | Emit all overlapping dictionary subwords inside compounds |
| `--mono` | Split into single characters for Classical Chinese (*Wenyanwen*) |
| `--no-space` | Remove spacing between segmented tokens |
| `--conv <MODE>` | Optional Traditional/Simplified Chinese conversion mode (default: `none`) |
| `--pinyin-std <cn\|tw>` | Select Mainland PRC or Taiwan pronunciation standard |
| `--build` | Build `dict.da` and Pinyin binaries from a source dictionary |
| `-i, --input <FILE>` | Custom input dictionary file (`.u8`, `.txt`, `.tsv`) for `--build` |
| `--dict-dir <DIR>` | Custom directory for compiled `dict.da` and Pinyin `.bin` files |
| `--debug` | Print debug segmentation and pronunciation details |



## OCR Setup (Optional)

Place PP-OCRv6 `.mnn` detection/recognition models and charset files inside `models/` (ignored by Git):

```text
models/
├── PP-OCRv6_medium_det.mnn
├── PP-OCRv6_medium_rec.mnn
└── ppocr_keys_v6_medium.txt
```

---

## Inspirations
* **[`gd-marisa`](https://github.com/Ajatt-Tools/gd-tools#gd-marisa) (Functionality & Architecture):** Inspired by `gd-marisa`'s trie-backed sentence segmentation and clickable headword lookup workflow—rebuilt in pure Rust using Double-Array tries (`yada`) + `mmap` for microsecond Mandarin prefix matching with zero startup overhead.
* **[`gd-mandarin`](https://github.com/Ajatt-Tools/gd-tools#gd-mandarin) (Design & Layout):** The model of basic prototype.
