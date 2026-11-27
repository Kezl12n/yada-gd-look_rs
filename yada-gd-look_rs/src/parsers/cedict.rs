use super::num_tone::pinyin_to_toned;
use crate::cli::PinyinStd;
use std::collections::HashMap;

pub fn parse_cedict(
    line: &str,
    words: &mut Vec<String>,
    mut reading_map: Option<&mut HashMap<String, String>>,
    pinyin_std: PinyinStd,
    debug: bool,
) -> bool {
    // Not a CC-CEDICT line -> return `false` so `main.rs` runs the `else` fallback
    if !line.contains('[') || !line.contains('/') {
        return false;
    }

    // Special for CC-CEDICT format: Traditional Simplified [pinyin] /def/

    let mut parts = line.split_whitespace();
    if let (Some(trad), Some(simp)) = (parts.next(), parts.next()) {
        // mixed script support
        let trad_norm = trad.to_ascii_lowercase();
        let simp_norm = simp.to_ascii_lowercase();

        words.push(trad_norm.clone());
        words.push(simp_norm.clone());

        // Extract `[pinyin]` and convert numbers to tone marks
        if let Some((_, rest)) = line.split_once('[') {
            if let Some((raw_pinyin, after)) = rest.split_once(']') {
                let mut combined = pinyin_to_toned(raw_pinyin);

                // Scan `/.../` definitions for `Taiwan pr. [...]`, `also pr. [...]`, etc.
                let mut tail = after;
                let mut tw_alts = Vec::new();

                // split the two halves of tail
                while let Some((before_pr, after_pr)) = tail.split_once("pr. [") {
                    if let Some((alt_raw, next_tail)) = after_pr.split_once(']') {
                        let alt_toned = pinyin_to_toned(alt_raw);

                        if before_pr.ends_with("Taiwan ") {
                            if !tw_alts.contains(&alt_toned)
                                && !combined.split(" · ").any(|p| p == alt_toned)
                            {
                                tw_alts.push(alt_toned);
                            }
                        } else if !combined.split(" · ").any(|p| p == alt_toned) {
                            combined.push_str(" · ");
                            combined.push_str(&alt_toned);
                        }

                        // after last, a new trail comes in the next loop iteration
                        tail = next_tail;
                    } else {
                        break;
                    }
                }

                // Assemble final order based on Cn vs Tw:
                let has_tw_pr = !tw_alts.is_empty();
                if has_tw_pr {
                    if matches!(pinyin_std, PinyinStd::Tw) {
                        let tw_joined = tw_alts.join(" · ");
                        combined = format!("{tw_joined} · {combined}");
                    }
                }

                if debug && trad == "垃圾" {
                    eprintln!("DEBUG 垃圾: std={:?}, combined={:?}", pinyin_std, combined);
                }

                if debug && simp == "和" {
                    eprintln!("DEBUG 和: std={:?}, combined={:?}", pinyin_std, combined);
                }

                // add multiple pinyin readings
                for key in [trad_norm, simp_norm] {
                    if let Some(map) = reading_map.as_deref_mut() {
                        let entry = map.entry(key).or_default();
                        let is_multi_char = trad.chars().count() > 1;

                        for (idx, r) in combined.split(" · ").enumerate() {
                            if !entry.split(" · ").any(|p| p == r) {
                                if entry.is_empty() {
                                    entry.push_str(r);
                                } else if idx == 0
                                    && has_tw_pr
                                    && is_multi_char
                                    && matches!(pinyin_std, PinyinStd::Tw)
                                {
                                    // Force Taiwan pronunciation to index 0 even if an earlier line existed
                                    *entry = format!("{r} · {entry}");
                                } else {
                                    entry.push_str(" · ");
                                    entry.push_str(r);
                                }
                            }
                        }
                    }
                }
            }
        }
        return true;
    }

    false
}
