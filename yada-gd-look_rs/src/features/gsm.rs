// Workaround scrip
use base64::{engine::general_purpose::STANDARD, Engine};
use obws::{requests::sources::TakeScreenshot, Client};
use ocr_rs::{Backend, OcrEngine, OcrEngineConfig};
use std::{env, time::Duration};
use strsim::normalized_levenshtein;
use tungstenite::{connect, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let models = env::current_exe()?.parent().unwrap().join("models");

    let cfg = OcrEngineConfig::new().with_backend(Backend::Vulkan);
    let ocr = OcrEngine::new(
        models.join("PP-OCRv6_small_det.mnn").to_str().unwrap(),
        models.join("PP-OCRv6_small_rec.mnn").to_str().unwrap(),
        models.join("ppocrv6_dict.txt").to_str().unwrap(),
        Some(cfg),
    )?;

    // Match your OBS Tools -> WebSocket Server Settings (use None::<&str> if auth is off)
    let obs = Client::connect("127.0.0.1", 4455, Some("your_obs_password")).await?;
    let (mut gsm_ws, _) = connect("ws://127.0.0.1:55499")?;
    let (mut last_raw, mut last_sent) = (String::new(), String::new());

    loop {
        if let Ok(b64) = obs.sources().take_screenshot(TakeScreenshot {
            source: "OCR".into(),
            format: "png",
            width: None,
            height: None,
            compression_quality: Some(0),
        }).await {
            let png = STANDARD.decode(b64.split(',').nth(1).unwrap_or(&b64))?;
            let img = image::load_from_memory(&png)?;

            let text: String = ocr.recognize(&img)?
                .into_iter()
                .filter(|r| r.confidence >= 0.75)
                .map(|r| r.text.trim().replace(' ', ""))
                .collect();

            if !text.is_empty()
                && text == last_raw
                && normalized_levenshtein(&text, &last_sent) < 0.85
            {
                gsm_ws.send(Message::Text(text.clone().into()))?;
                last_sent = text.clone();
            }
            last_raw = text;
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}
