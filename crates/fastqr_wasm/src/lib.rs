use fastqr_core::{EncodeOptions, ErrorCorrectionLevel, MaskPattern, QrCode, Version, encode_text};
use fastqr_image::{DecodeOptions, RenderOptions, decode_rgba, render_to_rgba};
use js_sys::{Object, Reflect, Uint8Array};
use wasm_bindgen::{Clamped, JsCast, JsValue, prelude::wasm_bindgen};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlVideoElement, ImageData};

#[wasm_bindgen]
pub struct WasmQrCode {
    inner: QrCode,
}

#[wasm_bindgen]
impl WasmQrCode {
    #[wasm_bindgen(getter)]
    pub fn size(&self) -> u32 {
        self.inner.size() as u32
    }

    #[wasm_bindgen(getter)]
    pub fn version(&self) -> u8 {
        self.inner.version().value()
    }

    #[wasm_bindgen(getter)]
    pub fn mask(&self) -> u8 {
        self.inner.mask().value()
    }

    #[wasm_bindgen(js_name = "errorCorrection")]
    pub fn error_correction(&self) -> String {
        ecc_to_str(self.inner.error_correction()).into()
    }

    #[wasm_bindgen(js_name = "modules")]
    pub fn modules(&self) -> Uint8Array {
        Uint8Array::from(self.inner.modules_row_major().as_slice())
    }

    #[wasm_bindgen(js_name = "renderRgba")]
    pub fn render_rgba(&self, render: JsValue, border: u32) -> Result<Uint8Array, JsValue> {
        let (_, render) = parse_render_options(&render, border)?;
        let rgba = render_to_rgba(&self.inner, render).map_err(to_js_error)?;
        Ok(Uint8Array::from(rgba.as_slice()))
    }

    #[wasm_bindgen(js_name = "renderSvg")]
    pub fn render_svg(&self, border: usize) -> String {
        self.inner.to_svg_string(border)
    }
}

#[wasm_bindgen(js_name = "encodeText")]
pub fn encode_text_wasm(text: &str, options: JsValue) -> Result<WasmQrCode, JsValue> {
    let options = parse_encode_options(&options)?;
    let code = encode_text(text, options).map_err(to_js_error)?;
    Ok(WasmQrCode { inner: code })
}

#[wasm_bindgen(js_name = "decodeRgba")]
pub fn decode_rgba_wasm(
    rgba: &[u8],
    width: u32,
    height: u32,
    options: JsValue,
) -> Result<JsValue, JsValue> {
    let decoded = decode_rgba(
        width as usize,
        height as usize,
        rgba,
        parse_decode_options(&options)?,
    )
    .map_err(to_js_error)?;
    decoded_to_js(decoded)
}

#[wasm_bindgen(js_name = "renderToCanvas")]
pub fn render_to_canvas(
    canvas: HtmlCanvasElement,
    text: &str,
    render: JsValue,
    border: u32,
) -> Result<(), JsValue> {
    let (encode, render) = parse_render_options(&render, border)?;
    let code = encode_text(text, encode).map_err(to_js_error)?;
    let size = checked_canvas_size(code.size(), render.scale, render.border)?;
    canvas.set_width(size);
    canvas.set_height(size);
    let context = canvas_context_2d(&canvas)?;
    let rgba = render_to_rgba(&code, render).map_err(to_js_error)?;
    let data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&rgba), size, size)?;
    context.put_image_data(&data, 0.0, 0.0)?;
    Ok(())
}

#[wasm_bindgen(js_name = "decodeCanvas")]
pub fn decode_canvas(canvas: HtmlCanvasElement, options: JsValue) -> Result<JsValue, JsValue> {
    let context = canvas_context_2d(&canvas)?;
    let data = context.get_image_data(0.0, 0.0, canvas.width() as f64, canvas.height() as f64)?;
    let rgba = data.data().0;
    let decoded = decode_rgba(
        canvas.width() as usize,
        canvas.height() as usize,
        &rgba,
        parse_decode_options(&options)?,
    )
    .map_err(to_js_error)?;
    decoded_to_js(decoded)
}

#[wasm_bindgen(js_name = "decodeVideoFrame")]
pub fn decode_video_frame(
    video: HtmlVideoElement,
    canvas: HtmlCanvasElement,
    options: JsValue,
) -> Result<JsValue, JsValue> {
    let width = video.video_width();
    let height = video.video_height();
    if width == 0 || height == 0 {
        return Err(JsValue::from_str("video element has no frame yet"));
    }
    canvas.set_width(width);
    canvas.set_height(height);
    let context = canvas_context_2d(&canvas)?;
    context.draw_image_with_html_video_element(&video, 0.0, 0.0)?;
    decode_canvas(canvas, options)
}

fn canvas_context_2d(canvas: &HtmlCanvasElement) -> Result<CanvasRenderingContext2d, JsValue> {
    let context = canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("2D canvas context is unavailable"))?;
    Ok(context.dyn_into::<CanvasRenderingContext2d>()?)
}

fn parse_encode_options(value: &JsValue) -> Result<EncodeOptions, JsValue> {
    let mut options = EncodeOptions::default();
    if value.is_null() || value.is_undefined() {
        return Ok(options);
    }
    if let Some(level) = value.as_string() {
        options.min_error_correction = parse_ecc(&level)?;
        return Ok(options);
    }
    if !value.is_object() {
        return Err(JsValue::from_str(
            "encode options must be an error correction string or an object",
        ));
    }
    if let Some(level) =
        string_property(value, "minErrorCorrection")?.or(string_property(value, "errorCorrection")?)
    {
        options.min_error_correction = parse_ecc(&level)?;
    }
    if let Some(version) = u8_property(value, "minVersion", 1, 40)? {
        options.min_version = Version::new(version).map_err(to_js_error)?;
    }
    if let Some(version) = u8_property(value, "maxVersion", 1, 40)? {
        options.max_version = Version::new(version).map_err(to_js_error)?;
    }
    if let Some(boost) = bool_property(value, "boostErrorCorrection")? {
        options.boost_error_correction = boost;
    }
    if let Some(mask) = u8_property(value, "mask", 0, 7)? {
        options.mask = Some(MaskPattern::new(mask).map_err(to_js_error)?);
    }
    Ok(options)
}

fn parse_decode_options(value: &JsValue) -> Result<DecodeOptions, JsValue> {
    let mut options = DecodeOptions::default();
    if value.is_null() || value.is_undefined() {
        return Ok(options);
    }
    if !value.is_object() {
        return Err(JsValue::from_str("decode options must be an object"));
    }
    if let Some(try_invert) = bool_property(value, "tryInvert")? {
        options.try_invert = try_invert;
    }
    if let Some(max_pixels) = usize_property(value, "maxPixels", 1)? {
        options.max_pixels = Some(max_pixels);
    }
    Ok(options)
}

fn parse_render_options(
    value: &JsValue,
    positional_border: u32,
) -> Result<(EncodeOptions, RenderOptions), JsValue> {
    let mut render = RenderOptions::default();
    if value.is_null() || value.is_undefined() {
        return Ok((EncodeOptions::default(), render));
    }
    if let Some(scale) = value.as_f64() {
        if !scale.is_finite() || scale.fract() != 0.0 || scale < 0.0 || scale > f64::from(u32::MAX)
        {
            return Err(JsValue::from_str(
                "render scale must be a non-negative integer",
            ));
        }
        render.scale = scale as u32;
        render.border = positional_border;
        return Ok((EncodeOptions::default(), render));
    }
    if !value.is_object() {
        return Err(JsValue::from_str(
            "render options must be a scale number or an object",
        ));
    }
    let encode = parse_encode_options(value)?;
    if let Some(scale) = u32_property(value, "scale", 0)? {
        render.scale = scale;
    }
    if let Some(border) = u32_property(value, "border", 0)? {
        render.border = border;
    }
    if let Some(dark) = string_property(value, "dark")? {
        render.dark = parse_rgba(&dark)?;
    }
    if let Some(light) = string_property(value, "light")? {
        render.light = parse_rgba(&light)?;
    }
    Ok((encode, render))
}

fn property(object: &JsValue, name: &str) -> Result<JsValue, JsValue> {
    Reflect::get(object, &JsValue::from_str(name))
}

fn string_property(object: &JsValue, name: &str) -> Result<Option<String>, JsValue> {
    let value = property(object, name)?;
    if value.is_null() || value.is_undefined() {
        return Ok(None);
    }
    value
        .as_string()
        .map(Some)
        .ok_or_else(|| JsValue::from_str(&format!("{name} must be a string")))
}

fn bool_property(object: &JsValue, name: &str) -> Result<Option<bool>, JsValue> {
    let value = property(object, name)?;
    if value.is_null() || value.is_undefined() {
        return Ok(None);
    }
    value
        .as_bool()
        .map(Some)
        .ok_or_else(|| JsValue::from_str(&format!("{name} must be a boolean")))
}

fn u8_property(
    object: &JsValue,
    name: &str,
    minimum: u8,
    maximum: u8,
) -> Result<Option<u8>, JsValue> {
    let value = property(object, name)?;
    if value.is_null() || value.is_undefined() {
        return Ok(None);
    }
    let number = value
        .as_f64()
        .ok_or_else(|| JsValue::from_str(&format!("{name} must be a number")))?;
    if !number.is_finite()
        || number.fract() != 0.0
        || number < f64::from(minimum)
        || number > f64::from(maximum)
    {
        return Err(JsValue::from_str(&format!(
            "{name} must be an integer from {minimum} through {maximum}"
        )));
    }
    Ok(Some(number as u8))
}

fn u32_property(object: &JsValue, name: &str, minimum: u32) -> Result<Option<u32>, JsValue> {
    let value = property(object, name)?;
    if value.is_null() || value.is_undefined() {
        return Ok(None);
    }
    let number = value
        .as_f64()
        .ok_or_else(|| JsValue::from_str(&format!("{name} must be a number")))?;
    if !number.is_finite()
        || number.fract() != 0.0
        || number < f64::from(minimum)
        || number > f64::from(u32::MAX)
    {
        return Err(JsValue::from_str(&format!(
            "{name} must be an integer from {minimum} through {}",
            u32::MAX
        )));
    }
    Ok(Some(number as u32))
}

fn usize_property(object: &JsValue, name: &str, minimum: usize) -> Result<Option<usize>, JsValue> {
    let value = property(object, name)?;
    if value.is_null() || value.is_undefined() {
        return Ok(None);
    }
    let number = value
        .as_f64()
        .ok_or_else(|| JsValue::from_str(&format!("{name} must be a number")))?;
    if !number.is_finite() || number.fract() != 0.0 || number < minimum as f64 {
        return Err(JsValue::from_str(&format!(
            "{name} must be an integer greater than or equal to {minimum}"
        )));
    }
    Ok(Some(number as usize))
}

fn parse_rgba(value: &str) -> Result<[u8; 4], JsValue> {
    let hex = value.strip_prefix('#').unwrap_or(value);
    match hex.len() {
        6 => Ok([
            parse_hex_channel(&hex[0..2])?,
            parse_hex_channel(&hex[2..4])?,
            parse_hex_channel(&hex[4..6])?,
            255,
        ]),
        8 => Ok([
            parse_hex_channel(&hex[0..2])?,
            parse_hex_channel(&hex[2..4])?,
            parse_hex_channel(&hex[4..6])?,
            parse_hex_channel(&hex[6..8])?,
        ]),
        _ => Err(JsValue::from_str("expected #RRGGBB or #RRGGBBAA color")),
    }
}

fn parse_hex_channel(value: &str) -> Result<u8, JsValue> {
    u8::from_str_radix(value, 16).map_err(|_| JsValue::from_str("invalid hex color channel"))
}

fn checked_canvas_size(modules: usize, scale: u32, border: u32) -> Result<u32, JsValue> {
    if scale == 0 {
        return Err(JsValue::from_str("render scale must be greater than zero"));
    }
    let side = modules
        .checked_add(
            usize::try_from(border)
                .map_err(|_| JsValue::from_str("render border is too large"))?
                .checked_mul(2)
                .ok_or_else(|| JsValue::from_str("render border is too large"))?,
        )
        .and_then(|modules| modules.checked_mul(scale as usize))
        .ok_or_else(|| JsValue::from_str("render dimensions are too large"))?;
    u32::try_from(side).map_err(|_| JsValue::from_str("render dimensions are too large"))
}

fn decoded_to_js(decoded: fastqr_core::DecodedQr) -> Result<JsValue, JsValue> {
    let object = Object::new();
    Reflect::set(
        &object,
        &JsValue::from_str("version"),
        &JsValue::from_f64(decoded.version.value() as f64),
    )?;
    Reflect::set(
        &object,
        &JsValue::from_str("errorCorrection"),
        &JsValue::from_str(ecc_to_str(decoded.error_correction)),
    )?;
    Reflect::set(
        &object,
        &JsValue::from_str("mask"),
        &JsValue::from_f64(decoded.mask.value() as f64),
    )?;
    Reflect::set(
        &object,
        &JsValue::from_str("bytes"),
        &Uint8Array::from(decoded.bytes.as_ref()),
    )?;
    let text = decoded
        .text
        .as_deref()
        .map(JsValue::from_str)
        .unwrap_or(JsValue::NULL);
    Reflect::set(&object, &JsValue::from_str("text"), &text)?;
    Ok(object.into())
}

fn parse_ecc(value: &str) -> Result<ErrorCorrectionLevel, JsValue> {
    match value.to_ascii_uppercase().as_str() {
        "L" | "LOW" => Ok(ErrorCorrectionLevel::Low),
        "M" | "MEDIUM" => Ok(ErrorCorrectionLevel::Medium),
        "Q" | "QUARTILE" => Ok(ErrorCorrectionLevel::Quartile),
        "H" | "HIGH" => Ok(ErrorCorrectionLevel::High),
        _ => Err(JsValue::from_str("unknown error correction level")),
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

fn to_js_error(error: impl ToString) -> JsValue {
    JsValue::from_str(&error.to_string())
}
