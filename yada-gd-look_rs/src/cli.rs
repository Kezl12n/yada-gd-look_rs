use clap::{Args, Parser, ArgAction};
use std::path::PathBuf;

#[derive(clap::ValueEnum, Clone, Copy, Debug, Default, PartialEq)]
pub enum PinyinStd {
    #[default]
    Cn, // Mainland Putonghua (default)
    Tw, // Taiwan Guoyu
}

#[derive(Parser, Debug)]
#[command(
    name = "yada-gd-look_rs",
    about = "Ultra-fast Mandarin & Double-Array Trie lookup"
)]

pub struct Cli {
     /// For debugging
    #[arg(long)]
    pub debug: bool,

    #[command(flatten, next_help_heading = "Lookup Options")]
    pub lookup: LookupArgs,

    #[command(flatten, next_help_heading = "Dictionary Options")]
    pub dict: DictArgs,

    #[command(flatten, next_help_heading = "Formatting Options")]
    pub format: FormatArgs,

    #[cfg(feature = "ocr")]
    #[command(flatten, next_help_heading = "OCR Options")]
    pub ocr: OcrArgs,
}

#[derive(Args, Debug)]
pub struct FormatArgs {
    /// Remove spacing
    #[arg(long)]
    pub no_space: bool,

    /// Optional Traditional Chinese conversion mode (default: none)
    #[arg(long)]
    pub conv: Option<String>,

    /// Split into single characters for Classical Chinese
    #[arg(long)]
    pub mono: bool,

    /// Show tone-marked Pinyin above Chinese words using <ruby> tags
    #[arg(long)]
    pub pinyin: bool,

    /// Choose a regional pronunciation standard (pinyin_tw.bin must be built)
    #[arg(long, value_enum, default_value_t = PinyinStd::Cn)]
    pub pinyin_std: PinyinStd,


    /// Enabled (`true`) by default. Passing `--no-cjk` sets it to `false` meaning subwords will also split latin characters
    #[arg(long = "no-cjk", action = clap::ArgAction::SetFalse)]
    pub cjk_compound: bool,

    /// Start in vertical reading mode by default
    #[arg(short, long)]
    pub vertical: bool,

    /// Emit all overlapping dictionary subwords inside compounds
    #[arg(long)]
    pub subwords: bool,
}


#[derive(Args, Debug)]
pub struct LookupArgs {
    /// Target word (%GDWORD% in GoldenDict)
    #[arg(short, long)]
    pub word: Option<String>,

    /// Full sentence context (%GDSEARCH% in GoldenDict)
    #[arg(short, long)]
    pub sentence: Option<String>,
}

#[derive(Args, Debug)]
pub struct DictArgs {
    /// Build dict.da from source dictionary
    #[arg(long)]
    pub build: bool,

    /// Custom dict file (.u8, .txt, .tsv) for --build
    #[arg(short, long)]
    pub input: Option<PathBuf>,

    /// Directly specify a path to dictionary folder
    #[arg(long)]
    pub dict_dir: Option<PathBuf>,

    #[arg(long)]
    pub dict_da: Option<PathBuf>,

    #[arg(long)]
    pub reading: Option<PathBuf>,

    #[arg(long = "no-reading-map", action = ArgAction::SetFalse)]
    pub use_reading_map: bool,

    /// Dictionary profile name (loads dicts/<profile>/ and caches <profile>.da)
    #[arg(short = 'P', long, default_value = "default")]
    pub profile: String,

}

   
#[cfg(feature = "ocr")]
#[derive(Args, Debug)]
pub struct OcrArgs {
    /// Trigger OCR Scan (Path to the screenshot image)
    #[arg(short = 'o', long, value_name = "IMAGE")]
    pub ocr: Option<String>,

    /// Disable mixed horizontal/vertical reading in OCR (only horizontal)
    #[arg(long = "no-layout-reading", action = clap::ArgAction::SetFalse)]
    pub layout_reading: bool,

    /// OCR model tier (tiny, small, medium)
    #[arg(short = 'm', long, value_enum, default_value_t = OcrModel::Medium)]
    pub model: OcrModel,
}

#[cfg(feature = "ocr")]
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OcrModel {
    #[value(name = "tiny")]
    Tiny,
    #[value(name = "small")]
    Small,
    #[value(name = "medium")]
    Medium,
}

#[cfg(feature = "ocr")]
impl OcrModel {
    pub fn as_preset(&self) -> &'static str {
        match self {
            OcrModel::Tiny => "ppocrv6-tiny",
            OcrModel::Small => "ppocrv6-small",
            OcrModel::Medium => "ppocrv6-medium",
        }
    }
}
