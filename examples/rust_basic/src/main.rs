use fastqr_core::{EncodeOptions, encode_text};
use fastqr_image::{RenderOptions, decode_file, write_to_path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let payload = "https://github.com/mates-inc/fastqr";
    let code = encode_text(payload, EncodeOptions::default())?;

    let output = std::env::temp_dir().join("fastqr-rust-basic.png");
    write_to_path(
        &code,
        &output,
        RenderOptions {
            scale: 10,
            border: 4,
            ..RenderOptions::default()
        },
    )?;

    let decoded = decode_file(&output)?;
    println!("rendered: {}", output.display());
    if let Some(text) = decoded.text {
        println!("decoded: {text}");
    }

    Ok(())
}
