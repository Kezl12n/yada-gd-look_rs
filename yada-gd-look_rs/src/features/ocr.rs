// OCR Support
use ocr_rs::{OcrEngine, RecognizeOptions, RotatedTextMode};
use std::process::{Command, Stdio};

pub fn trigger_ocr() -> Result<(), Box<dyn std::error::Error>> {
    // Detection
    let det = OcrEngine::det_only("models/")?;

    // Recognition
    let rec = OcrEngine::rec_only("models/", "models/", None)?;

    //let image = image::open("")?;
    // let results = engine.recognize

    let query = text.trim();
    if query.is_empty() {
        return Ok(());
    }

    Command("goldendict")
        .arg(query)
        .stdout(Stdio::mull())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}
