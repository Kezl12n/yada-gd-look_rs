use clap::Parser;
use memmap2::Mmap;
use pinyin::Pinyin;
use std::fs::{self, File};
use std::path::PathBuf;
use yada::DoubleArray;
use yada::builder::DoubleArrayBuilder;
use zhconv::{Variant, zhconv};

const CSS: &str = include_str!("style.css");

#[derive(Parser)]
#[command(
    name = "yada-gd-look",
    about = "Ultra-fast Mandarin & Double-Array Trie lookup"
)]
struct Cli {
    /// Build dict.da from source dictionary
    #[arg(long)]
    build: bool,

    // for debugging
    #[arg(long)]
    debug: bool,

    /// custom input dictionary file (.u8, .txt, .tsv) for --build
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
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        return PathBuf::from(local_app_data).join("yada-gd-look");
    }

    // fallback
    PathBuf::from(".")
}

// Pinyin settings
fn format_token(word: &str, show_pinyin: bool) -> String {
    let escaped = word.replace('<', "&lt;").replace('>', "&gt;");
    if !show_pinyin || !word.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c)) {
        return format!("<a href=\"gdlookup:{0}\">{1}</a> ", word, escaped);
    }

    let py = Pinyin::phrase(word).to_string();

    format!(
        "<ruby><a href=\"gdlookup:{0}\">{1}</a><rt>{2}</rt></ruby> ",
        word, escaped, py
    )
}

// Main
fn main() {
    let cli = Cli::parse();

    // make discoverable no matter what
    let base_dir = get_dict_dir(cli.dict_dir);
    let da_path = base_dir.join("dict.da");

    fs::create_dir_all(&base_dir).expect("Failed to create dict directory");

    if cli.build {
        let dicts_dir = base_dir.join("dicts");
        fs::create_dir_all(&dicts_dir).ok();

        let mut files_to_read: Vec<PathBuf> = Vec::new();

        // reading through arg (--input)
        if let Some(custom_input) = cli.input {
            files_to_read.push(custom_input);
        } else {
            // reading through words.txt
            let default_words = base_dir.join("words.txt");
            if default_words.exists() {
                files_to_read.push(default_words);
            }
            // reading through dicts dir (Default)
            let dicts_dir = base_dir.join("dicts");
            if let Ok(entries) = fs::read_dir(&dicts_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        files_to_read.push(path);
                    }
                }
            }
        }

        if files_to_read.is_empty() {
            fs::create_dir_all(base_dir.join("dicts")).ok();
            panic!(
                "No dictionaries found! Put .txt, .u8, .tsv, or .dict.yaml files in {:?}",
                base_dir.join("dicts")
            );
        }

        let mut words: Vec<String> = Vec::new();

        // Iterate over all collected dictionary files (words.txt, --input, and dicts/*)
        for path in &files_to_read {
            // StarDict (.idx) support
            if path.extension().is_some_and(|e| e == "idx") {
                if let Ok(bytes) = fs::read(path) {
                    let mut i = 0;
                    while let Some(nul) = bytes[i..].iter().position(|&b| b == 0) {
                        if let Ok(w) = std::str::from_utf8(&bytes[i..i + nul]) {
                            let w = w.trim();
                            if !w.is_empty() {
                                words.push(w.to_ascii_lowercase());
                            }
                        }
                        i += nul + 9; // 1 byte '\0' + 4-byte offset + 4-byte size
                    }
                    println!("Loaded (.idx): {:?}", path.file_name().unwrap_or_default());
                }
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

                // Skip commentator in CC-CEDICT
                if in_yaml || line.is_empty() || line.starts_with('#') {
                    continue;
                }

                // CC-CEDICT format: Traditional Simplified [pinyin] /def/
                if line.contains('[') && line.contains('/') {
                    let mut parts = line.split_whitespace();
                    if let Some(trad) = parts.next() {
                        words.push(trad.to_ascii_lowercase());
                    }
                    if let Some(simp) = parts.next() {
                        words.push(simp.to_ascii_lowercase());
                    }
                } else {
                    // Rime .dict.yaml, TSV, or plain wordlist (word\tpinyin\tfreq)
                    let w = line.split('\t').next().unwrap_or(line).trim();
                    if !w.is_empty() {
                        words.push(w.to_ascii_lowercase());
                    }
                }
            }

            if cli.debug {
                println!("Loaded: {:?}", path.file_name().unwrap_or_default());
            }
        }

        // sort by raw UTF-8 bytes & remove duplicates
        words.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        words.dedup();

        // make growable buffer of raw bytes (8-bit, 32-bit)
        let keyset: Vec<(&[u8], u32)> = words.iter().map(|w| (w.as_bytes(), 0)).collect();
        let da_bytes = DoubleArrayBuilder::build(&keyset).expect("Failed to build DAT");

        let da_tmp = base_dir.join("dict.da.tmp");

        // swap atomically so active GoldenDict lookups never crash!
        fs::write(&da_tmp, &da_bytes).expect("Failed to write temp file");
        fs::rename(&da_tmp, &da_path).expect("Failed to swap dict.da");

        println!("Built dict.da!");
        return;
    }

    // Open dict files
    let da_mmap = unsafe { Mmap::map(&File::open(&da_path).unwrap()).unwrap() };
    let da = DoubleArray::new(da_mmap).expect("Invalid dict.da");

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
        Some("sg") => zhconv(&sentence, Variant::ZhSG),
        _ => sentence,
    };

    // Case-Insensitive Searching
    let low_sentence = sentence.to_ascii_lowercase();

    // byte cursor starting at index 0
    let mut pos = 0;

    // for GoldenDict
    let mut sentence_html = String::new();
    let mut subwords_html = String::new();

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
            sentence_html.push_str(&format_token(full_word, cli.pinyin));

            let is_cjk_compound = full_word.chars().count() > 1
                && full_word
                    .chars()
                    .all(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c));

            // split in subwords
            if cli.subwords && is_cjk_compound {
                let full_word_low = &remains_low[..byte_len];

		// for ordering and deduplicating subwords
		let mut subs: Vec<&str> = Vec::new();


                // sub wrapper
                subwords_html.push_str("<div class=\"token-box\">");
                subwords_html.push_str(&format_token(full_word, cli.pinyin));

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
		    subwords_html.push_str(&format_token(sub, cli.pinyin));
		}

                subwords_html.push_str("</div>");
            }

            if cli.debug {
                eprintln!("[MATCH], '{}'", &remains[..byte_len]);
            }
            pos += byte_len;
        } else {
            // Grab 1 UTF-8 character
            let first_char = remains.chars().next().unwrap();

            if first_char.is_ascii_alphanumeric() {
                // Count total bytes of all consecutive English letters or digits (e.g. "Rust" or "2026")
                let ascii_len: usize = remains
                    .chars()
                    // iterate only alphanumeric characters
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                    .map(|c| c.len_utf8())
                    .sum();

                let ascii_word = &remains[..ascii_len];
                sentence_html.push_str(&format!("<a href=\"gdlookup:{0}\">{0}</a> ", ascii_word));

                if cli.debug {
                    eprintln!("[ASCII], '{}'", ascii_word);
                }

                pos += ascii_len; // jump forward by latin characters
            }
            // Bring all emojis! (starting with the cyclone)
            else if (first_char as u32) >= 0x1F300 {
                sentence_html.push_str(&format!("<a href=\"gdlookup:{0}\">{0}</a> ", first_char));
                pos += first_char.len_utf8();
            }
            // Bring the question/exclamation marks in!
            else if matches!(first_char, '?' | '？' | '!' | '！') {
                let encoded = match first_char {
                    '?' => "%3F".to_string(),
                    _ => first_char.to_string(),
                };
                sentence_html.push_str(&format!(
                    "<a href=\"gdlookup:{}\">{}</a>",
                    encoded, first_char
                ));
                pos += first_char.len_utf8();
            } else {
                // For hanzi characters & punctuation (important for mono)
                if !first_char.is_whitespace()
                    && !first_char.is_ascii_punctuation()
                    && !"《》 （）！？，。、".contains(first_char)
                {
                    sentence_html
                        .push_str(&format_token(&remains[..first_char.len_utf8()], cli.pinyin));
                } else {
                    sentence_html.push(first_char);
                }
                if cli.debug {
                    eprintln!("[CHAR], '{}'", first_char);
                }
                pos += first_char.len_utf8(); // jump forward by 3 or 4 bytes
            }
        }
    }

    // Print final HTML for GoldenDict-ng!
    if cli.subwords {
        println!(
            r#"<style>{}</style><div class="gd-bal"><div class="sentence">{}</div><div class="subwords">{}</div></div>"#,
            CSS, sentence_html, subwords_html
        );
    } else {
        println!(
            r#"<style>{}</style><div class="gd-bal"><div class="sentence">{}</div></div>"#,
            CSS, sentence_html
        );
    }
}
