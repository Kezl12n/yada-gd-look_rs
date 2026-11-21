// Pinyin numbers to tone conversion

fn to_toned(s: &str) -> String {
    let s = s
        .replace("u:", "ü")
        .replace("U:", "Ü")
        .replace('v', "ü")
        .replace('V', "Ü");
    let Some(t @ 1..=4) = s.bytes().last().and_then(|b| (b as char).to_digit(10)) else {
        return s.trim_end_matches('5').into();
    };
    let base = &s[..s.len() - 1];
    let idx = base
        .find(['a', 'e', 'A', 'E'])
        .or_else(|| base.find("ou").or_else(|| base.find("Ou")))
        .or_else(|| base.rfind(['a', 'e', 'i', 'o', 'u', 'ü', 'A', 'E', 'I', 'O', 'U', 'Ü']));

    let Some(i) = idx else { return base.into() };
    let v = base[i..].chars().next().unwrap();
    let marked = match v {
        'a' => ['ā', 'á', 'ǎ', 'à'],
        'e' => ['ē', 'é', 'ě', 'è'],
        'i' => ['ī', 'í', 'ǐ', 'ì'],
        'o' => ['ō', 'ó', 'ǒ', 'ò'],
        'u' => ['ū', 'ú', 'ǔ', 'ù'],
        'ü' => ['ǖ', 'ǘ', 'ǚ', 'ǜ'],
        'A' => ['Ā', 'Á', 'Ǎ', 'À'],
        'E' => ['Ē', 'É', 'Ě', 'È'],
        'I' => ['Ī', 'Í', 'Ǐ', 'Ì'],
        'O' => ['Ō', 'Ó', 'Ǒ', 'Ò'],
        'U' => ['Ū', 'Ú', 'Ǔ', 'Ù'],
        'Ü' => ['Ǖ', 'Ǘ', 'Ǚ', 'Ǜ'],
        _ => return base.into(),
    }[t as usize - 1];

    format!("{}{marked}{}", &base[..i], &base[i + v.len_utf8()..])
}

// numbers to toned
pub fn pinyin_to_toned(bracket_content: &str) -> String {
    bracket_content
        .split_inclusive(['1', '2', '3', '4', '5'])
        .map(to_toned) // Using your 12-line converter (with uppercase vowels added!)
        .collect()
}
