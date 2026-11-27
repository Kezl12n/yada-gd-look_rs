// Basic OCR Support
use rusto::download::parse_config_with_auto_download;
use rusto::{DetectTextBuilderExt, DetectTextResult, ImageSource, RustO};
use std::process::Command;

pub fn ocr_to_goldendict(
    img_path: &str,
    model_preset: &str,
    layout_reading: bool, 
) -> Result<(), Box<dyn std::error::Error>> {
    rusto::init();

    let json_cfg = format!(r#"{{"model": "{model_preset}"}}"#);
    let config = parse_config_with_auto_download(&json_cfg)?;
    let mut ocr = RustO::initialize(config)?;

    let result = ocr
        .detect_text_builder(ImageSource::Path(img_path.into()))
        .with_layout_reading(layout_reading)
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
