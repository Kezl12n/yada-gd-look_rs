# yada-gd-look_rs

A fast, zero-bloat Mandarin Chinese companion for **GoldenDict-ng**, written in Rust.

## Design Principles

* **Zero Bloat (`yada` + `mmap`):** Dictionaries are compiled ahead of time into memory-mapped Double-Array tries (`yada`), delivering microsecond segmentation and ~0 ms startup latency.
* **Cross-Strait & Classical Aware:** Includes Simplified/Traditional conversion, and a dedicated **mono-character mode** for reading Classical Chinese (*文言文*) without modern word-segmentation interference.
* **Reproducible:** Declaratively built and packaged with **Nix flakes**.

---

## Features
* **Dictionary-Backed Sentence Segmentation:** Segments full sentences against your compiled `yada` trie so every clickable token matches real dictionary headwords.
* **Clean CN / TW Pinyin Standards:** Separate `pinyin.bin` and `pinyin_tw.bin` builds parsed from CC-CEDICT keeping Mainland readings unpolluted by Taiwan variants (`和` -> `hé · hè · hú · huó · huò`, `垃圾` -> `lā jī`) while preserving dedicated Taiwan pronunciation ordering (`hàn`, `lè sè`).
---



## Quick Start 

> **Nix / Flake Tip:** There are optional features included in the flake's default e.g. `buildFeatures = [ "ocr" ];`. If you want a base build **without** optional dependencies, override it in your flake:
> ```nix
> # Minimal build (no features)
> yada-gd-look_rs.override { features = [ ]; }
>
> # Select specific features
> yada-gd-look_rs.override { features = [ "ocr" "other-feature" ]; }
> ```


### 1. Build Headword dictionary (`dict.da`) & Pinyin `.bin` Files 

1. **Default (`dicts/` directory):** Drop all your source dictionary files (`.u8`, `.txt`, `.tsv`) into the `dicts/` folder and run:
   ```bash
   yada-gd-look_rs --build
   ```
2. **Direct Input (`--input`):** Compile a single specific dictionary file directly:
   ```bash
   yada-gd-look_rs --build --input /path/to/your/dict
   ```
3. **Embedded Fallback (`src/words.txt`):** Compiles from the built-in `src/words.txt` wordlist when no external dictionary files are supplied.

> **Note (CC-CEDICT):** Pass `--pinyin-std cn` or `--pinyin-std tw` to choose pronunciation behavior (`cn` excludes Taiwan pronunciations; `tw` includes both and prioritizes Taiwan).


### Pinyin Extraction & Fallback (`pinyin.bin` / `pinyin_tw.bin`)

During `--build`, `yada-gd-look_rs` extracts and normalizes pronunciations directly from your supported dictionaries:

* **Supported Dictionary Pinyin:** Automatically parses and normalizes entries that already contain **tone-marked Pinyin** (e.g., `hé`, `lā jī`) or **numbered Pinyin** (e.g., `he2`, `la1 ji1`), preserving multi-pronunciation ordering and regional (`cn` vs `tw`) variants.
* **`pinyin-converter` Fallback:** If a dictionary entry has no Pinyin column or uses an unsupported phonetic format (such as Zhuyin/Bopomofo) `yada-gd-look_rs` automatically falls back to **`pinyin-converter`** to generate tone-marked Pinyin from the Chinese characters so every headword still gets pronunciation coverage.



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
| `--mono` | Split into single characters for Classical Chinese (*文言文*) |
| `--no-space` | Remove spacing between segmented tokens |
| `--conv <MODE>` | Optional Traditional/Simplified Chinese conversion mode (default: `none`) |
| `--pinyin-std <cn\|tw>` | Select Mainland PRC or Taiwan pronunciation standard |
| `--build` | Build `dict.da` and Pinyin binaries from a source dictionary |
| `-i, --input <FILE>` | Custom input dictionary file (`.u8`, `.txt`, `.tsv`) for `--build` |
| `--dict-dir <DIR>` | Custom directory for compiled `dict.da` and Pinyin `.bin` files |
| `--debug` | Print debug segmentation and pronunciation details |
| `-o, --ocr` | Trigger PaddleOCR scan *(requires `--features ocr`)* |
| `-t, --ocr-tier <TIER>` | PP-OCRv6 model tier: `medium` or `small` (default: `medium`) |
| `-v, --vertical` | Enable rotated/vertical text detection (`Robust` mode) |
| `--debug` | Print debug segmentation and pronunciation details |



## OCR Setup (Optional)
`yada-gd-look_rs` includes an optional **PaddleOCR (PP-OCRv6/ocr-rs)** integration that reads screenshots directly from your clipboard and opens the recognized text in GoldenDict-ng.

### 1. Build with OCR Support
The OCR feature is disabled by default to keep the base binary lightweight. Enable it at compile time:

```bash
cargo build --release --features ocr
```


Place PP-OCRv6 `.mnn` detection/recognition models and charset files inside `models/` 

```text
models/
├── PP-OCRv6_medium_det.mnn
├── PP-OCRv6_medium_rec.mnn
└── ppocr_keys_v6_medium.txt
```

### OCR Hotkey Usage

Bind any of these commands to a global desktop shortcut:

```bash
yada-gd-look_rs -o             # Standard horizontal OCR (medium tier)
yada-gd-look_rs -ov            # Vertical / rotated text OCR (Robust mode)
yada-gd-look_rs -ov -t small   # Vertical OCR using the small model tier
```

---

`yada-gd-look_rs -o` reads the latest screenshot directly from your system clipboard, runs OCR, and sends the recognized text straight to GoldenDict-ng.

#### Linux (Niri / Wayland)
* **One-hotkey drag-to-OCR** (requires `grim`, `slurp`, and `wl-clipboard`) in `~/.config/niri/config.kdl`:
  ```kdl
  binds {
      // Horizontal text OCR
      Mod+O { spawn "sh" "-c" "grim -g \"\$(slurp)\" - | wl-copy && yada-gd-look_rs -o"; }

      // Vertical / rotated text OCR
      Mod+Shift+O { spawn "sh" "-c" "grim -g \"\$(slurp)\" - | wl-copy && yada-gd-look_rs -ov"; }
  }
  ```
* **General desktop (GNOME / KDE / X11):** Take a region screenshot to your clipboard using your normal screenshot shortcut, then run `yada-gd-look_rs -o` (or `-ov` for vertical text).

#### Windows
1. Press **`Win + Shift + S`** and drag a box around the text to copy it to your clipboard.
2. Run `yada-gd-look_rs.exe -o` (or `-ov` for vertical text).
   * **Tip:** Create a desktop shortcut to `yada-gd-look_rs.exe`, append `-o` to the **Target** field, set **Start in** to the folder containing `models/`, and assign a **Shortcut key** (e.g., `Ctrl + Alt + O`) in **Properties**.


---

## Credits
* **[`gd-marisa`](https://github.com/Ajatt-Tools/gd-tools#gd-marisa) (Functionality & Architecture):** Inspired by `gd-marisa`'s trie-backed sentence segmentation and clickable headword lookup workflow—rebuilt in pure Rust using Double-Array tries (`yada`) + `mmap` for microsecond Mandarin prefix matching with zero startup overhead.
* **[`gd-mandarin`](https://github.com/Ajatt-Tools/gd-tools#gd-mandarin) (Design & Layout):** The model of basic prototype.
