mod binary;
mod decode;
mod detect;
mod format;
mod render;

#[cfg(test)]
mod tests;

use std::{fmt, path::PathBuf};

use fastqr_core::QrError;

pub use decode::{
    decode_bytes, decode_bytes_with_format, decode_dynamic_image, decode_file,
    decode_file_with_options, decode_luma, decode_rgba,
};
pub use render::{
    encode_bytes_to_image, encode_text_to_image, render_to_image, render_to_rgba, write_to_bytes,
    write_to_path,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RasterFormat {
    Png,
    Jpeg,
    WebP,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderOptions {
    pub scale: u32,
    pub border: u32,
    pub dark: [u8; 4],
    pub light: [u8; 4],
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            scale: 8,
            border: 4,
            dark: [0, 0, 0, 255],
            light: [255, 255, 255, 255],
        }
    }
}

/// Default decode guard for browser/camera-sized inputs.
///
/// The default permits up to a 2048 x 2048 image. Callers that handle trusted
/// offline images can set [`DecodeOptions::max_pixels`] to a larger value or
/// `None`.
pub const DEFAULT_DECODE_MAX_PIXELS: usize = 2048 * 2048;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodeOptions {
    /// Try decoding the inverted binary image when the normal image has no
    /// decodable QR candidate.
    pub try_invert: bool,
    /// Maximum decoded image area accepted by the raster decoder.
    ///
    /// This guard is checked for raw buffers and is also passed to the image
    /// decoder before decoding PNG/JPEG/WebP bytes. `None` disables fastqr's
    /// pixel-area guard.
    pub max_pixels: Option<usize>,
}

impl Default for DecodeOptions {
    fn default() -> Self {
        Self {
            try_invert: true,
            max_pixels: Some(DEFAULT_DECODE_MAX_PIXELS),
        }
    }
}

#[derive(Debug)]
pub enum RasterError {
    Image(image::ImageError),
    InvalidBuffer,
    InvalidDimensions,
    MissingExtension(PathBuf),
    Qr(QrError),
    Detector(&'static str),
}

impl fmt::Display for RasterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Image(error) => write!(f, "{error}"),
            Self::InvalidBuffer => f.write_str("invalid image buffer dimensions"),
            Self::InvalidDimensions => f.write_str("invalid or unsupported image dimensions"),
            Self::MissingExtension(path) => {
                write!(
                    f,
                    "could not infer image format from path {}",
                    path.display()
                )
            }
            Self::Qr(error) => write!(f, "{error}"),
            Self::Detector(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for RasterError {}

impl From<image::ImageError> for RasterError {
    fn from(value: image::ImageError) -> Self {
        Self::Image(value)
    }
}

impl From<QrError> for RasterError {
    fn from(value: QrError) -> Self {
        Self::Qr(value)
    }
}
