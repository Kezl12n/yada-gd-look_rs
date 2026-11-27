use clap::Parser;
use memmap2::Mmap;
use pinyin::Pinyin;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use yada::DoubleArray;
use yada::builder::DoubleArrayBuilder;
use zhconv::{Variant, zhconv};

// local files
mod char_handler;
mod features;
mod parsers;

const CSS: &str = include_str!("style.css");

#[derive(clap::ValueEnum, Clone, Copy, Debug, Default, PartialEq)]
pub enum PinyinStd {
    #[default]
    Cn, // Mainland Putonghua (default)
    Tw, // Taiwan Guoyu
}

#[derive(Parser)]
#[command(
    name = "yada-gd-look",
    about = "Ultra-fast Mandarin & Double-Array Trie lookup"
)]

struct Cli {
    /// Remove spacing
    #[arg(long)]
    no_space: bool,

    /// Build dict.da from source dictionary
    #[arg(long)]
    build: bool,

    /// For debugging
    #[arg(long)]
    debug: bool,

    /// Custom input dict file (.u8, .txt, .tsv) for --build
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Custom dict directory
    #[arg(long)]
    dict_dir: Option<PathBuf>,

    /// Target word (%GDWORD% in GoldenDict)
    #[arg(short, long)]
    word: Option<String>,

    /// Full sentence context (%GDSEARCH% in GoldenDict)
    #[arg(short, long)]
    sentence: Option<String>,

    /// Optional Traditional Chinese conversion mode (default: none)
    #[arg(long)]
    conv: Option<String>,

    /// Split into single characters for Classical Chinese
    #[arg(long)]
    mono: bool,

    /// Emit all overlapping dictionary subwords inside compounds
    #[arg(long)]
    subwords: bool,

    /// Show tone-marked Pinyin above Chinese words using <ruby> tags
    #[arg(long)]
    pinyin: bool,

    /// Trigger OCR Scan (Path to the screenshot image)
    #[cfg(feature = "ocr")]
    #[arg(short = 'o', long, value_name = "IMAGE")]
    ocr: Option<String>,

    /// OCR model tier (tiny, small, medium)
    #[cfg(feature = "ocr")]
    #[arg(short = 'm', long, value_enum, default_value_t = OcrModel::Medium)]
    model: OcrModel,

    /// Choose a regional pronunciation standard (pinyin_tw.bin must be built)
    #[arg(long, value_enum, default_value_t = PinyinStd::Cn)]
    pub pinyin_std: PinyinStd,

    /// Start in vertical reading mode by default
    #[arg(short, long)]
    vertical: bool,

    /// Dictionary profile name (loads dicts/<profile>/ and caches <profile>.da)
    #[arg(short = 'P', long, default_value = "default")]
    profile: String,
}

// use mtime
fn needs_rebuild(
    // AsRef = anything that can be cheaply treated as a file system (Path)
    // IntoIterator = anything to loop,
    // item = each element yielded by that loop
    caches: &[impl AsRef<Path>],
    dict_files: impl IntoIterator<Item = impl AsRef<Path>>,
) -> bool {
    let mut oldest_cache = None;

    for cache in caches {
        let Ok(mtime) = fs::metadata(cache).and_then(|m| m.modified()) else {
            return true; // cache file doesn't exist yet
        };
        oldest_cache = Some(match oldest_cache {
            Some(prev) if prev < mtime => prev,
            _ => mtime,
        });
    }

    let Some(oldest_cache) = oldest_cache else {
        return false; // no caches specified
    };

    dict_files.into_iter().any(|file| {
        fs::metadata(file)
            .and_then(|m| m.modified())
            .map(|mtime| mtime > oldest_cache)
            .unwrap_or(true)
    })
}

#[cfg(feature = "ocr")]
#[derive(clap::ValueEnum, Clone, Debug)]
enum OcrModel {
    #[value(name = "tiny")]
    Tiny,
    #[value(name = "small")]
    Small,
    #[value(name = "medium")]
    Medium,
}

#[cfg(feature = "ocr")]
impl OcrModel {
    fn as_preset(&self) -> &'static str {
        match self {
            OcrModel::Tiny => "ppocrv6-tiny",
            OcrModel::Small => "ppocrv6-small",
            OcrModel::Medium => "ppocrv6-medium",
        }
    }
}

// Directory path
fn get_dict_dir(override_dir: Option<PathBuf>) -> PathBuf {
    if let Some(dir) = override_dir {
        return dir;
    }

    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(xdg).join("yada-gd-look");
    }

    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".local/share/yada-gd-look");
    }

    // for Windows
    #[cfg(target_os = "windows")]
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        return PathBuf::from(local_app_data).join("yada-gd-look");
    }

    // fallback
    PathBuf::from(".")
}

// include rare/historical char
fn is_cjk(c: char) -> bool {
    matches!(
        c,
        '\u{4E00}'..='\u{9FFF}'   // CJK Unified Ideographs (Main)
        | '\u{3400}'..='\u{4DBF}' // CJK Extension A (Rare characters)
        | '\u{F900}'..='\u{FAFF}' // CJK Compatibility Ideographs
        | '\u{20000}'..='\u{323AF}' // CJK Extensions B–I
    )
}

// Spacing settings (/w pinyin)
fn format_token(
    word: &str,
    // for pinyin (including dict & fallback)
    show_pinyin: bool,

    // for dict pinyin
    pinyin_map: &std::collections::HashMap<String, String>,
) -> String {
    // prevent broken links on quotes & html layout
    let escaped = word
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;");

    if !show_pinyin || !word.chars().any(is_cjk) {
        return format!("<a href=\"gdlookup:{0}\">{1}</a>", escaped, escaped);
    }

    // pinyin variables
    let (py, src) = match pinyin_map.get(&word.to_ascii_lowercase()) {
        Some(p) => (p.clone(), 0), // `p` is just the `&String` found inside `pinyin_map`
        None => (Pinyin::phrase(word).to_string(), 1),
    };

    // pinyin: dict first (#0), pinyin-converter as fallback (#1)
    format!(
        "<ruby data-src=\"{src}\"><a href=\"gdlookup:{escaped}\">{escaped}</a><rt>{py}</rt></ruby>"
    )
}

// filter dicts by allowing letters with tone marks (pinyin)
fn has_tone_marks(pinyin: &str) -> bool {
    const TONE_CHARS: &str = "āáǎàēéěèīíǐìōóǒòūúǔùǖǘǚǜĀÁǍÀĒÉĚÈĪÍǏÌŌÓǑÒŪÚǓÙǕǗǙǛ";
    pinyin.chars().any(|c| TONE_CHARS.contains(c))
}

// Main
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    #[cfg(feature = "ocr")]
    if let Some(image) = cli.ocr {
        if let Err(e) = features::ocr::ocr_to_goldendict(&image, cli.model.as_preset()) {
            eprintln!("OCR fout: {e}");
            std::process::exit(1);
        }
        return;
    }

    // make discoverable no matter what
    let base_dir = get_dict_dir(cli.dict_dir);
    let dicts_dir = base_dir.join("dicts");

    let cn_dir = dicts_dir.join("cn");
    let tw_dir = dicts_dir.join("tw");

    // dict content
    let mut words: Vec<String> = Vec::new();
    let mut pinyin_map = std::collections::HashMap::new();

    // multi profile (dict.da & pinyin.bin)
    let (da_path, py_path, profile_dir) = if cli.profile == "default" {
        // for specific pinyin.bin files
        let py_file = match cli.pinyin_std {
            PinyinStd::Cn => "pinyin.bin",
            PinyinStd::Tw => "pinyin_tw.bin",
        };

        let sub_dir = match cli.pinyin_std {
            PinyinStd::Cn => dicts_dir.join("cn"),
            PinyinStd::Tw => dicts_dir.join("tw"),
        };

        (base_dir.join("dict.da"), base_dir.join(py_file), sub_dir)
    } else {
        let p = &cli.profile;
        (
            base_dir.join(format!("{p}.da")),
            base_dir.join(format!("pinyin_{p}.bin")),
            dicts_dir.join(p),
        )
    };

    fs::create_dir_all(&base_dir).expect("Failed to create dict directory");
    let mut files_to_read: Vec<PathBuf> = Vec::new();

    // Helper closure to collect files from a list of directories
    let collect_files = |dirs: &[&PathBuf]| -> Vec<PathBuf> {
        let mut out = Vec::new();
        // reading through arg (--input) and borrow custom by ref
        if let Some(ref custom) = cli.input {
            out.push(custom.clone());
        } else {
            // reading through words.txt
            let default_words = base_dir.join("words.txt");
            if default_words.exists() {
                out.push(default_words);
            }
            // reading through dicts dir (Default)
            for dir in dirs {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            out.push(path);
                        }
                    }
                }
            }
        }
        out
    };

    // all dicts
    let std_dir = match cli.pinyin_std {
        PinyinStd::Cn => &cn_dir,
        PinyinStd::Tw => &tw_dir,
    };

    // collect all dicts/ dictionaries and the CN, TW subfolders
    let all_files = if cli.profile == "default" {
        (
            collect_files(&[&dicts_dir, &std_dir]),
            collect_files(&[&dicts_dir, &profile_dir]),
        )
    } else {
        // custom profile only
        fs::create_dir_all(&profile_dir).ok();
        let files = collect_files(&[&profile_dir]);
        (files.clone(), files)
    };

    if cli.build
        || needs_rebuild(
            &[&da_path, &py_path],
            all_files.0.iter().chain(&all_files.1),
        )
    {
        fs::create_dir_all(&dicts_dir).ok();

        fs::create_dir_all(&cn_dir).ok();
        fs::create_dir_all(&tw_dir).ok();

        // Collect dict files (including profile_dir!) and add to files_to_read
        files_to_read.extend(all_files.1.iter().cloned());
        if cli.debug {
            eprintln!("Found {} files: {:?}", files_to_read.len(), all_files);
        }

        if files_to_read.is_empty() {
            let target = if cli.profile == "default" {
                &dicts_dir
            } else {
                &profile_dir
            };
            fs::create_dir_all(target).ok();

            panic!(
                "No dictionaries found! Put .txt, .u8, .tsv, .idx or .dict.yaml files in {:?}",
                target
            );
        }

        // Iterate over all collected dictionary files (words.txt, --input, and dicts/*)
        for path in &files_to_read {
            // StarDict (.idx) support
            if path.extension().is_some_and(|e| e == "idx") {
                parsers::stardict::parse_idx(path, &mut words, cli.debug);
                continue;
            }

            // Read file as UTF-8 text; silently skip binary or unreadable files (like dict.da)
            let Ok(raw) = fs::read_to_string(path) else {
                continue;
            };

            // Tracks whether we are inside a Rime .dict.yaml metadata header (--- to ...)
            let mut in_yaml = false;

            // Modify lines
            for line in raw.lines().map(str::trim) {
                // Skip Rime .dict.yaml frontmatter (--- ... ...)
                if line == "---" {
                    in_yaml = true;
                    continue;
                }
                if line == "..." {
                    in_yaml = false;
                    continue;
                }

                if in_yaml || line.is_empty() || line.starts_with('#') {
                    continue;
                }

                if !parsers::cedict::parse_cedict(
                    &line,
                    &mut words,
                    &mut pinyin_map,
                    cli.pinyin_std,
                    cli.debug,
                ) {
                    // Rime .dict.yaml, TSV, or plain wordlist (word\tpinyin\tfreq) (index 0)
                    let mut cols = line.split('\t');
                    let w = cols.next().unwrap_or("").trim();
                    if !w.is_empty() {
                        let key = w.to_ascii_lowercase();
                        words.push(key.clone());

                        // grab from index 1 (extract dict's pinyin)
                        if let Some(py) = cols.next().map(str::trim).filter(|s| !s.is_empty()) {
                            if has_tone_marks(py) {
                                // filter by tone marks
                                let entry = pinyin_map.entry(key).or_default();
                                if !entry.split(" · ").any(|p| p == py) {
                                    if !entry.is_empty() {
                                        entry.push_str(" · ");
                                    }
                                    entry.push_str(py);
                                }
                            }
                        }
                    }
                }
            }

            for v in pinyin_map.values_mut() {
                if v.contains(" · ") {
                    let mut parts: Vec<&str> = v.split(" · ").collect();
                    parts.sort_by_key(|p| p.chars().next().is_some_and(|c| c.is_uppercase()));
                    *v = parts.join(" · ");
                }
            }

            if cli.debug {
                eprintln!("Loaded: {:?}", path.file_name().unwrap_or_default());
                eprintln!("DEBUG: pinyin_map loaded {} entries", pinyin_map.len());
                eprintln!("DEBUG: sample '中国' -> {:?}", pinyin_map.get("中国"));
            }
        }

        // sort by raw UTF-8 bytes & remove duplicates
        words.sort_unstable();
        words.dedup();

        // make growable buffer of raw bytes (8-bit, 32-bit)
        let keyset: Vec<(&[u8], u32)> = words.iter().map(|w| (w.as_bytes(), 0)).collect();

        if keyset.is_empty() {
            anyhow::bail!("No dictionary entries found in {:?}", all_files);
        }

        let da_bytes = DoubleArrayBuilder::build(&keyset).expect("Failed to build DAT");

        let da_tmp = da_path.with_extension("da.tmp");

        // swap atomically so active GoldenDict lookups never crash!
        fs::write(&da_tmp, &da_bytes).expect("Failed to write temp file");
        fs::rename(&da_tmp, &da_path).expect("Failed to swap .da file");

        let py_data: String = pinyin_map
            .iter()
            .map(|(k, v)| format!("{k}\t{v}\n"))
            .collect();

        let py_tmp = py_path.with_extension("bin.tmp");
        fs::write(&py_tmp, &py_data).unwrap();
        fs::rename(&py_tmp, &py_path).unwrap();

        // ensure the naming is always accurate and inserted
        if cli.build {
            println!(
                "Built {} & {}!",
                da_path.file_name().unwrap().to_string_lossy(),
                py_path.file_name().unwrap().to_string_lossy(),
            );
            return Ok(());
        }
    }

    // Open files
    let da_mmap = unsafe { Mmap::map(&File::open(&da_path).unwrap()).unwrap() };
    let da = DoubleArray::new(da_mmap).expect("Invalid dict.da");

    let py_raw = fs::read_to_string(&py_path).unwrap_or_default();
    let pinyin_map: std::collections::HashMap<String, String> = py_raw
        .lines()
        .filter_map(|l| l.split_once('\t'))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let sentence = cli
        .sentence
        .or(cli.word)
        .unwrap_or_else(|| "⚠️ | 输入无效 (Invalid input)".to_string());

    // Conversion settings
    let sentence = match cli.conv.as_deref() {
        Some("tw") => zhconv(&sentence, Variant::ZhTW),
        Some("hk") => zhconv(&sentence, Variant::ZhHK),
        Some("hant") => zhconv(&sentence, Variant::ZhHant),
        Some("cn") => zhconv(&sentence, Variant::ZhCN),
        _ => sentence,
    };

    // Case-Insensitive Searching
    let low_sentence = sentence.to_ascii_lowercase();

    // byte cursor starting at index 0
    let mut pos = 0;

    // for GoldenDict
    let mut sentence_html = String::new();
    let mut subwords_html = String::new();

    // hashset for grid to reduce duplicates
    let mut seen_boxes = std::collections::HashSet::new();

    // ensuring the lowercase sent matches as well
    while pos < low_sentence.len() {
        let remains = &sentence[pos..];
        let remains_low = &low_sentence[pos..];

        // longest match with check
        let valid_match = if cli.mono {
            None
        } else {
            da.common_prefix_search(remains_low.as_bytes())
                .filter(|&(_, byte_len)| {
                    let matched = &remains[..byte_len];
                    // If match ends with ASCII alphanumeric, ensure next char isn't also ASCII alphanumeric
                    if matched
                        .bytes()
                        .last()
                        .is_some_and(|b| b.is_ascii_alphanumeric())
                    {
                        !remains
                            .as_bytes()
                            .get(byte_len)
                            .is_some_and(|b| b.is_ascii_alphanumeric())
                    } else {
                        true
                    }
                })
                .last()
        };

        // longest match
        if let Some((_, byte_len)) = valid_match {
            let full_word = &remains[..byte_len];

            // sent wrapper
            sentence_html.push_str(&format_token(full_word, cli.pinyin, &pinyin_map));
            if !cli.no_space {
                sentence_html.push(' ');
            }

            let is_cjk_compound = full_word.chars().count() > 1
                && full_word
                    .chars()
                    .all(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c));

            // split in subwords
            if cli.subwords && is_cjk_compound {
                let full_word_low = &remains_low[..byte_len];

                // for ordering and deduplicating subwords
                let mut subs: Vec<&str> = Vec::new();

                if seen_boxes.insert(full_word) {
                    // sub wrapper
                    subwords_html.push_str("<div class=\"token-box\">");
                    subwords_html.push_str(&format_token(full_word, cli.pinyin, &pinyin_map));

                    // Step through every character starting position inside the compound
                    for (sub_offset, _) in full_word.char_indices() {
                        for (_, sub_len) in
                            da.common_prefix_search(full_word_low[sub_offset..].as_bytes())
                        {
                            // Skip the full compound itself so it isn't printed twice
                            if sub_len < byte_len {
                                let sub = &full_word[sub_offset..sub_offset + sub_len];
                                if !subs.contains(&sub) {
                                    subs.push(sub);
                                }
                            }
                        }
                    }

                    // longest -> shortest (left-to-right order)
                    subs.sort_by_key(|s| std::cmp::Reverse(s.chars().count()));

                    // Render all sorted subwords at once
                    for sub in subs {
                        subwords_html.push_str(&format_token(sub, cli.pinyin, &pinyin_map));
                    }

                    subwords_html.push_str("</div>");
                }
            }

            if cli.debug {
                eprintln!("[MATCH], '{}'", &remains[..byte_len]);
            }
            pos += byte_len;
        } else {
            let (step, html) =
                char_handler::handle_unmatched(remains, cli.pinyin, &pinyin_map, cli.debug);
            sentence_html.push_str(&html);
            pos += step; // jump forward by latin characters (3 or 4 bytes)
        }
    }

    let wrapper_class = if cli.no_space {
        "gd-bal no-space"
    } else {
        "gd-bal"
    };

    let checked_attr = if cli.vertical { "checked" } else { "" };

    // Print final HTML for GoldenDict-ng!
    if !subwords_html.is_empty() {
        println!(
            r#"<style>{}</style><div class="{}"><label class="v-btn"><input type="checkbox" class="v-toggle" {} hidden>直/横</label><div class="sentence">{}</div><div class="subwords">{}</div></div>"#,
            CSS, wrapper_class, checked_attr, sentence_html, subwords_html
        );
    } else {
        println!(
            r#"<style>{}</style><div class="{}"><label class="v-btn"><input type="checkbox" class="v-toggle" {} hidden>直/横</label><div class="sentence">{}</div></div>"#,
            CSS, wrapper_class, checked_attr, sentence_html
        );
    }

    Ok(())
}
