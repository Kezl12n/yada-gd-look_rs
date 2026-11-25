use std::fs;
use std::path::Path;

pub fn parse_idx(path: &Path, words: &mut Vec<String>, debug: bool) {
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

        if debug {
            eprintln!("Loaded (.idx): {:?}", path.file_name().unwrap_or_default());
        }
    }
}
