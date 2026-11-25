// Basic OCR Support
use rusto::{DetectTextBuilderExt, DetectTextResult, ImageSource, InitializeConfig, RustO};
use std::process::Command;

pub fn ocr_to_goldendict(img_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    
    rusto::init();
    let mut ocr = RustO::initialize(InitializeConfig::default())?;

    let result = ocr
        .detect_text_builder(ImageSource::Path(img_path.into()))
        .as_lines()
        .run()?;

    if let DetectTextResult::Structured(lines) = result {
        let sentence: String = lines.into_iter().map(|l| l.text).collect();
        if !sentence.is_empty() {
            Command::new("goldendict").arg(&sentence).spawn()?;
        }
    }

    Ok(())
}
