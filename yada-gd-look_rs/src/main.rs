use std::fs::{self, File};
use std::path::PathBuf;
use memmap2::Mmap;
use yada::builder::DoubleArrayBuilder;
use yada::DoubleArray;
use clap::Parser;


#[derive(Parser)]
#[command(name = "gd-yada", about = "Ultra-fast Mandarin & Double-Array Trie lookup")]
struct Cli {
    /// Build dict.da and defs.bin from source dictionary
    #[arg(long)]
    build: bool,

    /// Custom dict directory
    #[arg(long)]
    dict_dir: Option<PathBuf>,

    /// Target word (%GDWORD% in GoldenDict)
    #[arg(short, long)]
    word: Option<String>,

    /// Full sentence context (%GDSEARCH% in GoldenDict)
    #[arg(short, long)]
    sentence: Option<String>,
}

fn get_dict_dir(override_dir: Option<PathBuf>) -> PathBuf {
   if let Some(dir) = override_dir {
      return dir;
   }

   if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
      return PathBuf::from(xdg).join("gd-yada");
   }

   if let Ok(home) = std::env::var("HOME") {
      return PathBuf::from(home).join(".local/share/gd-yada");
   }

   // for Windows
   if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        return PathBuf::from(local_app_data).join("yada-gd-look");
   }
   
   // fallback
   PathBuf::from(".")
}


fn main() {
    let cli = Cli::parse();git rm --cached yada-gd-look_rs

    
    // make discoverable no matter what
    let base_dir = get_dict_dir(cli.dict_dir);
    let da_path = base_dir.join("dict.da");
    let defs_path = base_dir.join("defs.bin");

    if cli.build {
	 // Example definitions in exact ID order (ID 0, ID 1, ID 2...)
   	 let dict = [
             ("666", "666 | awesome / skilled"),
             ("yyds", "yyds | GOAT / eternal god"),
             ("图书馆", "tú shū guǎn | library"),
             ("尊嘟假嘟", "zūn dū jiǎ dū | Meme: really or fake?"),
   	 ];

   	 // make growable buffer of raw bytes (8-bit, 32-bit)
   	 let mut defs_bytes: Vec<u8> = Vec::new();
   	 let mut keyset: Vec<(&[u8], u32)> = Vec::new();

   	 for (word, def) in &dict {
     	      // exact byte offset for defs.bin
     	      let offset = defs_bytes.len() as u32;
     	      let def_raw = def.as_bytes();
     
	      defs_bytes.extend_from_slice(&(def_raw.len() as u32).to_le_bytes());
	      defs_bytes.extend_from_slice(def_raw);
     	      keyset.push((word.as_bytes(), offset));
   	 }

  	 // sort keyset
   	 keyset.sort_by_key(|k| k.0);
   	 let da_bytes = DoubleArrayBuilder::build(&keyset).expect("Failed to build DAT");


    	 let da_tmp = base_dir.join("dict.da.tmp");
         let defs_tmp = base_dir.join("defs.bin.tmp");

	 fs::create_dir_all(&base_dir).expect("Failed to create dict directory");

  	 // swap atomically so active GoldenDict lookups never crash!
   	 fs::write(&da_tmp, &da_bytes).expect("Failed to write temp file");
   	 fs::rename(&da_tmp, &da_path).expect("Failed to swap dict.da");

	 fs::write(&defs_tmp, &defs_bytes).unwrap();
    	 fs::rename(&defs_tmp, &defs_path).unwrap();
	 
	 println!("Built dict.da + defs.bin!");
	 return;
   }
   
   // Open dict files
   let da_mmap = unsafe { Mmap::map(&File::open(&da_path).unwrap()).unwrap() };
   let defs_mmap = unsafe { Mmap::map(&File::open(&defs_path).unwrap()).unwrap() };   
   let da = DoubleArray::new(da_mmap).expect("Invalid dict.da");

   let sentence = cli.sentence.or(cli.word).unwrap_or_else(|| {
        "主播操作666简直YYDS，我在图书馆看尊嘟假嘟！".to_string()
   });
   
   let low_sentence = sentence.to_ascii_lowercase();

   // byte cursor starting at index 0
   let mut pos = 0;

   // for GoldenDict
   let mut sentence_html = String::new();
   let mut cards_html = String::new();

   // 
   while pos < low_sentence.len() {
      let remains = &sentence[pos..];
      let remains_low = &low_sentence[pos..];

      // longest match
      if let Some((offset, byte_len)) = da.common_prefix_search(remains_low.as_bytes()).last() {
         let start = offset as usize;
	 let def_len = u32::from_le_bytes([
	     defs_mmap[start],
	     defs_mmap[start + 1],
	     defs_mmap[start + 2],
	     defs_mmap[start + 3]
	 ]) as usize;
	 let def_text = std::str::from_utf8(&defs_mmap[start + 4..start + 4 + def_len]).unwrap();

         sentence_html.push_str(&format!("<a href=\"bword://{0}\">{0}</a> ", &remains[..byte_len]));
	 
	 
         cards_html.push_str(&format!(
            "<div style=\"margin-top:8px; padding:6px; border-left:3px solid #4a90e2;\"><b>{}</b>: {}</div>",
            &remains[..byte_len], def_text
         ));

	 println!("[MATCH], '{}' (ID: {})", &remains[..byte_len], def_text);
	 pos += byte_len;
      }
      else {
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
	  sentence_html.push_str(&format!("<a href=\"bword://{0}\">{0}</a> ", ascii_word));

	  println!("[ASCII], '{}'", ascii_word);
	  pos += ascii_len; // jump forward by latin characters
	}
	else {
	  // For hanzi characters
	  println!("[CHAR], '{}'", first_char);
	  pos += first_char.len_utf8(); // jump forward by 3/4 bytes
	}
      }
   }

   // Print final HTML for GoldenDict-ng!
   println!("<div><div style=\"font-size:1.3em; margin-bottom:10px;\">{}</div>{}</div>", sentence_html, cards_html);
   

   // common prefixs
   //   let matches: Vec<_> = da.common_prefix_search(sentence.as_bytes()).collect();

   //println!("Matches at start of '{}': {:?}", sentence, matches);

}
