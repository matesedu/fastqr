use fastqr_core::{EncodeOptions, ErrorCorrectionLevel, QrCode, encode_text};
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
    pub fn render_rgba(&self, scale: u32, border: u32) -> Uint8Array {
        let rgba = render_to_rgba(
            &self.inner,
            RenderOptions {
                scale,
                border,
                ..RenderOptions::default()
            },
        );
        Uint8Array::from(rgba.as_slice())
    }

    #[wasm_bindgen(js_name = "renderSvg")]
    pub fn render_svg(&self, border: usize) -> String {
        self.inner.to_svg_string(border)
    }
}

#[wasm_bindgen(js_name = "encodeText")]
pub fn encode_text_wasm(
    text: &str,
    error_correction: Option<String>,
) -> Result<WasmQrCode, JsValue> {
    let mut options = EncodeOptions::default();
    if let Some(level) = error_correction {
        options.min_error_correction = parse_ecc(&level)?;
    }
    let code = encode_text(text, options).map_err(to_js_error)?;
    Ok(WasmQrCode { inner: code })
}

#[wasm_bindgen(js_name = "decodeRgba")]
pub fn decode_rgba_wasm(rgba: &[u8], width: u32, height: u32) -> Result<JsValue, JsValue> {
    let decoded = decode_rgba(
        width as usize,
        height as usize,
        rgba,
        DecodeOptions::default(),
    )
    .map_err(to_js_error)?;
    decoded_to_js(decoded)
}

#[wasm_bindgen(js_name = "renderToCanvas")]
pub fn render_to_canvas(
    canvas: HtmlCanvasElement,
    text: &str,
    scale: u32,
    border: u32,
) -> Result<(), JsValue> {
    let code = encode_text(text, EncodeOptions::default()).map_err(to_js_error)?;
    let size = (code.size() as u32 + border * 2) * scale;
    canvas.set_width(size);
    canvas.set_height(size);
    let context = canvas_context_2d(&canvas)?;
    let rgba = render_to_rgba(
        &code,
        RenderOptions {
            scale,
            border,
            ..RenderOptions::default()
        },
    );
    let data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&rgba), size, size)?;
    context.put_image_data(&data, 0.0, 0.0)?;
    Ok(())
}

#[wasm_bindgen(js_name = "decodeCanvas")]
pub fn decode_canvas(canvas: HtmlCanvasElement) -> Result<JsValue, JsValue> {
    let context = canvas_context_2d(&canvas)?;
    let data = context.get_image_data(0.0, 0.0, canvas.width() as f64, canvas.height() as f64)?;
    let rgba = data.data().0;
    let decoded = decode_rgba(
        canvas.width() as usize,
        canvas.height() as usize,
        &rgba,
        DecodeOptions::default(),
    )
    .map_err(to_js_error)?;
    decoded_to_js(decoded)
}

#[wasm_bindgen(js_name = "decodeVideoFrame")]
pub fn decode_video_frame(
    video: HtmlVideoElement,
    canvas: HtmlCanvasElement,
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
    decode_canvas(canvas)
}

fn canvas_context_2d(canvas: &HtmlCanvasElement) -> Result<CanvasRenderingContext2d, JsValue> {
    let context = canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("2D canvas context is unavailable"))?;
    Ok(context.dyn_into::<CanvasRenderingContext2d>()?)
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
    let text = decoded.text.map(String::from).unwrap_or_default();
    Reflect::set(
        &object,
        &JsValue::from_str("text"),
        &JsValue::from_str(&text),
    )?;
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
