# yada-gd-look_rs

A fast, zero-bloat Mandarin Chinese lookup engine for **GoldenDict-ng**, written in Rust.

<img width="900" height="203" alt="demo" src="https://github.com/user-attachments/assets/2f2e1567-a670-4ed6-863c-e6414742a52e" />

## Design Principles


* **Zero Bloat (`yada` + `mmap`):** Dictionaries are compiled ahead of time into memory-mapped Double-Array tries (`yada`), delivering microsecond segmentation and ~0 ms startup latency.
* **Cross-Strait & Classical Aware:** Includes Simplified/Traditional conversion, and a dedicated **mono-character mode** for reading Classical Chinese (*文言文*) without modern word-segmentation interference.
* **Reproducible:** Declaratively built and packaged with **Nix flakes**.

---

## Features
* **Dictionary-Backed Sentence Segmentation:** Segments full sentences against your compiled `yada` trie so every clickable token matches real dictionary headwords.
* **Clean CN / TW Pinyin Standards:** Separate `pinyin.bin` and `pinyin_tw.bin` builds parsed from CC-CEDICT keeping Mainland readings unpolluted by Taiwan variants (`和` -> `hé · hè · hú · huó · huò`, `垃圾` -> `lā jī`) while preserving dedicated Taiwan pronunciation ordering (`hàn`, `lè sè`).
* **Multi-Profile Support (`-P` / `--profile`):** Isolate specialized dictionaries (e.g., `default`, `medical`, `classical`, `slang`) into separate directories and `.da` cache artifacts.
* **Smart `mtime` Auto-Rebuild:** Automatically detects modified or newly added dictionary files via filesystem modification timestamps (`mtime`) with zero file-hashing overhead.
* **Multi-Format Parsing:** Supports **Rime YAML dictionaries (`*.dict.yaml`)**, StarDict (`.idx`), CC-CEDICT (`.u8`), and plain-text wordlists (`.txt`).



---

## Quick Start

### Installation
```bash
cd yada-gd-look_rs
cargo build --release
```

> **Nix / Flake Tip:** There are optional features included in the flake's default e.g. `buildFeatures = [ "ocr" ];`. If you want a base build **without** optional dependencies, override it in your flake:
> ```nix
> # Minimal build (no features)
> yada-gd-look_rs.override { features = [ ]; }
>
> # Select specific features
> yada-gd-look_rs.override { features = [ "ocr" "other-feature" ]; }
> ```

## Directory Layout

By default, dictionaries and compiled caches are stored under `~/.local/share/yada-gd-look/`:

```text
~/.local/share/yada-gd-look/
└─ dicts/
    ├── cn/                 # Mainland (CN) Pinyin sources (e.g., cedict_ts.u8)
    ├── tw/                 # Taiwan (TW) Pinyin sources
    └── <profile>/          # Profile-specific dictionaries (e.g., default/, medical/)
```


### 1. Build Headword dictionary (`dict.da`) & Pinyin `.bin` Files 
1. - **Default / Mainland CN (`dicts/cn/`):** Drop your source dictionary files (`.u8`, `.dict.yaml`, `.tsv`, `.txt`, `.idx`) into `~/.local/share/yada-gd-look/dicts/cn/` (or directly into `dicts/` for shared files) and run:
  ```bash
  yada-gd-look_rs --build
  ```
- **Taiwan TW (`dicts/tw/`):** Place TW-specific sources into `~/.local/share/yada-gd-look/dicts/tw/` and build with:
  ```bash
  yada-gd-look_rs --build --pinyin-std tw
  ```

2. **Universal Shared (`dicts/` directory):** Drop shared source files (`.u8`, `.dict.yaml`, `.tsv`, `.txt`, `.idx`) directly into `dicts/` (loaded across all profiles and Pinyin standards), or place specialized files into `dicts/cn/`, `dicts/tw/`, or `dicts/<profile>/`
   ```bash
   yada-gd-look_rs --build
   ```
3. **Direct Input (`--input`):** Compile a single specific dictionary file directly:
   ```bash
   yada-gd-look_rs --build --input /path/to/your/dict
   ```
4. **Embedded Fallback (`src/words.txt`):** Compiles from the built-in `src/words.txt` wordlist when no external dictionary files are supplied.

### Use a Custom Dictionary Profile
Loads dictionaries from `dicts/<profile>/` and caches to `<profile>.da`:
```bash
yada-gd-look_rs -P medical "心肌梗死"
# or
yada-gd-look_rs --profile classical "之乎者也"
```


> **Note (CC-CEDICT):** Pass `--pinyin-std cn` or `--pinyin-std tw` to choose pronunciation behavior (`cn` excludes Taiwan pronunciations; `tw` includes both and prioritizes Taiwan).

#### Supported Input Formats
* **Dictionaries with Pinyin:** CC-CEDICT (`.u8`), plain text (`.txt`), and `.tsv` files containing **tone-marked Pinyin** (`hé`, `lā jī`) or **numbered Pinyin** (`he2`, `la1 ji1`).
* **Headword-Only Dictionaries (No Pinyin):** **StarDict** (.idx) dictionaries and **RIME Schemas** (dict.yaml) are supported, yet most of them may not include a supported pinyin format. 


#### Pinyin Extraction & Fallback (`pinyin.bin` / `pinyin_tw.bin`)

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
| `--conv <Cn\|Cn-Trad\|Tw\|Hk>` | Optional Traditional/Simplified Chinese conversion mode (default: `none`) |
| `--pinyin-std <cn\|tw>` | Select Mainland PRC or Taiwan pronunciation standard |
| `--build` | Build `dict.da` and Pinyin binaries from a source dictionary |
| `-i, --input <FILE>` | Custom input dictionary file (`.u8`, `.txt`, `.tsv`) for `--build` |
| `--dict-dir <DIR>` | Custom directory for compiled `dict.da` |
| `--debug` | Print debug segmentation and pronunciation details |
| `-o, --ocr <IMAGE>` | Trigger scan *(requires `--features ocr`)* |
| `--model <TIER>` | PP-OCRv6 model tier: `medium`, `small` or `tiny` (default: `medium`) |
| `--debug` | Print debug segmentation and pronunciation details |
| `-v, --vertical` | Ensure vertical reading is always enabled |
| `--profile <NAME>` | `-P` | `default` | Dictionary profile name (`dicts/<profile>/` -> `<profile>.da`). |


## OCR Setup (Optional)

`yada-gd-look_rs` includes an optional pure-Rust PaddleOCR (`rusto-rs`) integration that extracts Simplified and Traditional Chinese text (both horizontal and vertical) from a screenshot and sends it directly to GoldenDict-ng without cluttering your clipboard.

### 1. Build with OCR Support

The OCR feature is disabled by default to keep the base binary lightweight. Enable it at compile time:

```bash
cargo build --release --features ocr
```

> **Note:** On the first run, `rusto-rs` automatically downloads and caches the pp-ocr model (default: medium). No need to do anything.
 

### 2. OCR Hotkey Usage

Pass a screenshot path directly to `-o` (or `--ocr`):

```bash
yada-gd-look_rs -o /tmp/ocr.png
```

---

## Screenshot use /w examples

### Why bring your own screenshot tool?
yada-gd-look_rs intentionally avoids bundling a heavy cross-platform screen-capture UI. Accepting a standard image file path keeps the binary minimal, avoids Wayland/X11/macOS/Windows permission headaches, and lets you plug OCR into whatever screenshot hotkey you already use.

#### Linux (Niri / Wayland) 

Bind a one-hotkey drag-to-OCR shortcut (this requires `grim` and `slurp`) in `~/.config/niri/config.kdl`:

```kdl
binds {
    // Screenshot region and send recognized Chinese text straight to GoldenDict-ng
    Mod+O { spawn "sh" "-c" "grim -g \"\$(slurp)\" /tmp/ocr.png && yada-gd-look_rs -o /tmp/ocr.png"; }
}
```

#### General Desktop (GNOME / KDE / X11)

Bind a custom global shortcut to capture a region to a temp file and run OCR:

```bash
flameshot gui -r > /tmp/ocr.png && yada-gd-look_rs -o /tmp/ocr.png
```

#### macOS

```bash
screencapture -i /tmp/ocr.png && yada-gd-look_rs -o /tmp/ocr.png
```

#### Windows (ShareX)

1. Open **ShareX** → **Task settings** → **After capture tasks** and enable **Save image to file** and **Run external program**.
2. Point the external program to `yada-gd-look_rs.exe` with arguments `-o "$img"`.

---

## Credits & Acknowledgments
Special thanks to the developers for their awesome work on:
* **[`gd-marisa`](https://github.com/Ajatt-Tools/gd-tools#gd-marisa) (Functionality & Architecture):** For causing inspiration behind the sentence segmentation and headword lookup functionality.
* **[`gd-mandarin`](https://github.com/Ajatt-Tools/gd-tools#gd-mandarin) (Design & Layout):** For the design and layout used as a beginning model.
