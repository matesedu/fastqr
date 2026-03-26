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
    decode_bytes, decode_bytes_with_format, decode_dynamic_image, decode_file, decode_luma,
    decode_rgba,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodeOptions {
    pub try_invert: bool,
}

impl Default for DecodeOptions {
    fn default() -> Self {
        Self { try_invert: true }
    }
}

#[derive(Debug)]
pub enum RasterError {
    Image(image::ImageError),
    InvalidBuffer,
    MissingExtension(PathBuf),
    Qr(QrError),
    Detector(&'static str),
}

impl fmt::Display for RasterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Image(error) => write!(f, "{error}"),
            Self::InvalidBuffer => f.write_str("invalid image buffer dimensions"),
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
