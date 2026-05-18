use fastqr_core::{EncodeOptions, ErrorCorrectionLevel, MaskPattern, Version, encode_text};
use fastqr_image::{
    DecodeOptions, RasterFormat, RenderOptions, decode_bytes, decode_rgba, write_to_bytes,
};
use napi::{
    Error, Result,
    bindgen_prelude::{Buffer, Either},
};
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
pub struct NapiEncodeOptions {
    pub error_correction: Option<String>,
    pub min_error_correction: Option<String>,
    pub min_version: Option<u8>,
    pub max_version: Option<u8>,
    pub mask: Option<u8>,
    pub boost_error_correction: Option<bool>,
}

#[napi(object)]
pub struct NapiRenderOptions {
    pub error_correction: Option<String>,
    pub min_error_correction: Option<String>,
    pub min_version: Option<u8>,
    pub max_version: Option<u8>,
    pub mask: Option<u8>,
    pub boost_error_correction: Option<bool>,
    pub scale: Option<u32>,
    pub border: Option<u32>,
    pub dark: Option<String>,
    pub light: Option<String>,
}

#[napi(object)]
pub struct NapiDecodeOptions {
    pub try_invert: Option<bool>,
    pub max_pixels: Option<u32>,
}

#[napi(js_name = "encodeText")]
pub fn encode_text_node(
    text: String,
    options: Option<Either<String, NapiEncodeOptions>>,
) -> Result<NapiQrCode> {
    let options = parse_encode_argument(options)?;
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
pub fn decode_image(bytes: Buffer, options: Option<NapiDecodeOptions>) -> Result<NapiDecodedQr> {
    let decoded =
        decode_bytes(bytes.as_ref(), parse_decode_options(options)?).map_err(to_napi_error)?;
    Ok(to_napi_decoded(decoded))
}

#[napi(js_name = "decodeRgba")]
pub fn decode_rgba_node(
    bytes: Buffer,
    width: u32,
    height: u32,
    options: Option<NapiDecodeOptions>,
) -> Result<NapiDecodedQr> {
    let decoded = decode_rgba(
        width as usize,
        height as usize,
        bytes.as_ref(),
        parse_decode_options(options)?,
    )
    .map_err(to_napi_error)?;
    Ok(to_napi_decoded(decoded))
}

fn render_image(
    text: String,
    render: Option<NapiRenderOptions>,
    format: RasterFormat,
) -> Result<Buffer> {
    let (encode_options, render_options) = parse_render_options(render)?;
    let code = encode_text(&text, encode_options).map_err(to_napi_error)?;
    let bytes = write_to_bytes(&code, format, render_options).map_err(to_napi_error)?;
    Ok(Buffer::from(bytes))
}

fn parse_encode_argument(
    value: Option<Either<String, NapiEncodeOptions>>,
) -> Result<EncodeOptions> {
    match value {
        Some(Either::A(level)) => Ok(EncodeOptions {
            min_error_correction: parse_ecc(&level)?,
            ..EncodeOptions::default()
        }),
        Some(Either::B(options)) => parse_encode_options(
            options.error_correction,
            options.min_error_correction,
            options.min_version,
            options.max_version,
            options.mask,
            options.boost_error_correction,
        ),
        None => Ok(EncodeOptions::default()),
    }
}

fn parse_encode_options(
    error_correction: Option<String>,
    min_error_correction: Option<String>,
    min_version: Option<u8>,
    max_version: Option<u8>,
    mask: Option<u8>,
    boost_error_correction: Option<bool>,
) -> Result<EncodeOptions> {
    let mut options = EncodeOptions::default();
    if let Some(level) = min_error_correction.or(error_correction) {
        options.min_error_correction = parse_ecc(&level)?;
    }
    if let Some(version) = min_version {
        options.min_version = Version::new(version).map_err(to_napi_error)?;
    }
    if let Some(version) = max_version {
        options.max_version = Version::new(version).map_err(to_napi_error)?;
    }
    if let Some(mask) = mask {
        options.mask = Some(MaskPattern::new(mask).map_err(to_napi_error)?);
    }
    if let Some(boost) = boost_error_correction {
        options.boost_error_correction = boost;
    }
    Ok(options)
}

fn parse_render_options(
    render: Option<NapiRenderOptions>,
) -> Result<(EncodeOptions, RenderOptions)> {
    let Some(render) = render else {
        return Ok((EncodeOptions::default(), RenderOptions::default()));
    };
    let encode = parse_encode_options(
        render.error_correction,
        render.min_error_correction,
        render.min_version,
        render.max_version,
        render.mask,
        render.boost_error_correction,
    )?;
    let mut options = RenderOptions::default();
    if let Some(scale) = render.scale {
        options.scale = scale;
    }
    if let Some(border) = render.border {
        options.border = border;
    }
    if let Some(dark) = render.dark {
        options.dark = parse_rgba(&dark)?;
    }
    if let Some(light) = render.light {
        options.light = parse_rgba(&light)?;
    }
    Ok((encode, options))
}

fn parse_decode_options(options: Option<NapiDecodeOptions>) -> Result<DecodeOptions> {
    let mut decode = DecodeOptions::default();
    if let Some(options) = options {
        if let Some(try_invert) = options.try_invert {
            decode.try_invert = try_invert;
        }
        if let Some(max_pixels) = options.max_pixels {
            decode.max_pixels = Some(max_pixels as usize);
        }
    }
    Ok(decode)
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

fn parse_rgba(value: &str) -> Result<[u8; 4]> {
    let hex = value.strip_prefix('#').unwrap_or(value);
    let channels = match hex.len() {
        6 => [
            parse_hex_channel(&hex[0..2])?,
            parse_hex_channel(&hex[2..4])?,
            parse_hex_channel(&hex[4..6])?,
            255,
        ],
        8 => [
            parse_hex_channel(&hex[0..2])?,
            parse_hex_channel(&hex[2..4])?,
            parse_hex_channel(&hex[4..6])?,
            parse_hex_channel(&hex[6..8])?,
        ],
        _ => return Err(Error::from_reason("expected #RRGGBB or #RRGGBBAA color")),
    };
    Ok(channels)
}

fn parse_hex_channel(value: &str) -> Result<u8> {
    u8::from_str_radix(value, 16).map_err(|_| Error::from_reason("invalid hex color channel"))
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
