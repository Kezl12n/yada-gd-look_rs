use crate::format_token;
use std::collections::HashMap;

pub fn handle_unmatched(
    remains: &str,
    use_pinyin: bool,
    reading_map: &HashMap<String, String>,
    debug: bool,
) -> (usize, String) {
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
        if debug {
            eprintln!("[ASCII], '{}'", ascii_word);
        }

        let html = format!(r#"<a href="gdlookup:{0}">{0}</a> "#, ascii_word);
        // return
        (ascii_len, html)
    }
    // Bring all emojis! (starting with the cyclone)
    else if (first_char as u32) >= 0x1F300 {
        let html = format!(r#"<a href="gdlookup:{0}">{0}</a> "#, first_char);
        (first_char.len_utf8(), html)
    }
    // Bring the question/exclamation marks in!
    else if matches!(first_char, '?' | '？' | '!' | '！') {
        let encoded = match first_char {
            '?' => "%3F".to_string(),
            _ => first_char.to_string(),
        };

        let html = format!(r#"<a href="gdlookup:{}">{}</a>"#, encoded, first_char);
        (first_char.len_utf8(), html)
    } else {
        if debug {
            eprintln!("[CHAR], '{}'", first_char);
        }

        let len = first_char.len_utf8();

        // For hanzi characters & punctuation (important for mono)

        let html = if !first_char.is_whitespace()
            && !first_char.is_ascii_punctuation()
            && !"“”《》（）！？、，。".contains(first_char)
        {
            format_token(&remains[..len], use_pinyin, reading_map)
        } else {
            first_char.to_string()
        };
        (len, html)
    }
}
