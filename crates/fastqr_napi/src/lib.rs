use fastqr_core::{EncodeOptions, ErrorCorrectionLevel, encode_text};
use fastqr_image::{
    DecodeOptions, RasterFormat, RenderOptions, decode_bytes, decode_rgba, write_to_bytes,
};
use napi::{Error, Result, bindgen_prelude::Buffer};
use napi_derive::napi;

#[napi(object)]
pub struct NapiQrCode {
    pub version: u8,
    pub error_correction: String,
    pub mask: u8,
    pub size: u32,
    pub modules: Buffer,
}

#[napi(object)]
pub struct NapiDecodedQr {
    pub version: u8,
    pub error_correction: String,
    pub mask: u8,
    pub text: Option<String>,
    pub bytes: Buffer,
}

#[napi(object)]
pub struct NapiRenderOptions {
    pub scale: Option<u32>,
    pub border: Option<u32>,
}

#[napi(js_name = "encodeText")]
pub fn encode_text_node(text: String, error_correction: Option<String>) -> Result<NapiQrCode> {
    let mut options = EncodeOptions::default();
    if let Some(level) = error_correction {
        options.min_error_correction = parse_ecc(&level)?;
    }
    let code = encode_text(&text, options).map_err(to_napi_error)?;
    Ok(NapiQrCode {
        version: code.version().value(),
        error_correction: ecc_to_str(code.error_correction()).into(),
        mask: code.mask().value(),
        size: code.size() as u32,
        modules: Buffer::from(code.modules_row_major()),
    })
}

#[napi(js_name = "renderPng")]
pub fn render_png(text: String, render: Option<NapiRenderOptions>) -> Result<Buffer> {
    render_image(text, render, RasterFormat::Png)
}

#[napi(js_name = "renderJpeg")]
pub fn render_jpeg(text: String, render: Option<NapiRenderOptions>) -> Result<Buffer> {
    render_image(text, render, RasterFormat::Jpeg)
}

#[napi(js_name = "renderWebp")]
pub fn render_webp(text: String, render: Option<NapiRenderOptions>) -> Result<Buffer> {
    render_image(text, render, RasterFormat::WebP)
}

#[napi(js_name = "decodeImage")]
pub fn decode_image(bytes: Buffer) -> Result<NapiDecodedQr> {
    let decoded = decode_bytes(bytes.as_ref(), DecodeOptions::default()).map_err(to_napi_error)?;
    Ok(to_napi_decoded(decoded))
}

#[napi(js_name = "decodeRgba")]
pub fn decode_rgba_node(bytes: Buffer, width: u32, height: u32) -> Result<NapiDecodedQr> {
    let decoded = decode_rgba(
        width as usize,
        height as usize,
        bytes.as_ref(),
        DecodeOptions::default(),
    )
    .map_err(to_napi_error)?;
    Ok(to_napi_decoded(decoded))
}

fn render_image(
    text: String,
    render: Option<NapiRenderOptions>,
    format: RasterFormat,
) -> Result<Buffer> {
    let code = encode_text(&text, EncodeOptions::default()).map_err(to_napi_error)?;
    let render = render.unwrap_or(NapiRenderOptions {
        scale: None,
        border: None,
    });
    let bytes = write_to_bytes(
        &code,
        format,
        RenderOptions {
            scale: render.scale.unwrap_or(8),
            border: render.border.unwrap_or(4),
            ..RenderOptions::default()
        },
    )
    .map_err(to_napi_error)?;
    Ok(Buffer::from(bytes))
}

fn parse_ecc(value: &str) -> Result<ErrorCorrectionLevel> {
    match value.to_ascii_uppercase().as_str() {
        "L" | "LOW" => Ok(ErrorCorrectionLevel::Low),
        "M" | "MEDIUM" => Ok(ErrorCorrectionLevel::Medium),
        "Q" | "QUARTILE" => Ok(ErrorCorrectionLevel::Quartile),
        "H" | "HIGH" => Ok(ErrorCorrectionLevel::High),
        _ => Err(Error::from_reason("unknown error correction level")),
    }
}

fn ecc_to_str(level: ErrorCorrectionLevel) -> &'static str {
    match level {
        ErrorCorrectionLevel::Low => "L",
        ErrorCorrectionLevel::Medium => "M",
        ErrorCorrectionLevel::Quartile => "Q",
        ErrorCorrectionLevel::High => "H",
    }
}

fn to_napi_decoded(decoded: fastqr_core::DecodedQr) -> NapiDecodedQr {
    NapiDecodedQr {
        version: decoded.version.value(),
        error_correction: ecc_to_str(decoded.error_correction).into(),
        mask: decoded.mask.value(),
        text: decoded.text.map(Into::into),
        bytes: Buffer::from(decoded.bytes.into_vec()),
    }
}

fn to_napi_error(error: impl ToString) -> Error {
    Error::from_reason(error.to_string())
}
