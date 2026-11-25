// Basic OCR Support (PaddleOCR V6)
use ocr_rs::{OcrEngine, RecognizeOptions, RotatedTextMode};
use std::process::{Command, Stdio};
use arboard::Clipboard;
use image::{DynamicImage, RgbaImage};

fn launch_goldendict(query: &str) -> std::io::Result<()> {
    #[cfg(not(windows))]
    let bin = "goldendict";

    #[cfg(windows)]
    let bin = [
        r"C:\Program Files\GoldenDict-ng\goldendict.exe",
        r"C:\Program Files\GoldenDict\goldendict.exe",
    ]
    .into_iter()
    .find(|p| std::path::Path::new(p).exists())
    .unwrap_or("goldendict");

    Command::new(bin)
        .arg(query)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}



pub fn trigger_ocr(tier: &str, vertical: bool) -> Result<(), Box<dyn std::error::Error>> {

    let tier = "medium"; // or "small"

    let engine = OcrEngine::new(
    	format!("models/PP-OCRv6_{tier}_det.mnn"),
    	format!("models/PP-OCRv6_{tier}_rec.mnn"),
    	format!("models/ppocr_keys_v6_{tier}.txt"),
    	None,
    )?;

    // Grab image directly from the clipboard
    let img = Clipboard::new()?.get_image()?;
    let rgba = RgbaImage::from_raw(
    	img.width as u32,
    	img.height as u32,
    	img.bytes.into_owned(),
    )
    .ok_or("Invalid clipboard image")?;
    let image = DynamicImage::ImageRgba8(rgba);


    // Mixed horizontal and vertical text
    let results = if vertical {
        let opts = RecognizeOptions::new().with_rotated_text_mode(RotatedTextMode::Robust);
        engine.recognize_with_options(&image, &opts)?
    } else {
        engine.recognize(&image)?
    };

    let text = results
    	.into_iter()
    	.map(|r| r.text)
    	.collect::<Vec<_>>()
    	.join("");


    let query = text.trim();
    if !query.is_empty() {
       launch_goldendict(query)?;
    }
    
    Ok(())


}
