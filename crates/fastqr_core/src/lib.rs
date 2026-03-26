#![forbid(unsafe_code)]

mod bit_buffer;
mod bit_grid;
mod error;
mod gf256;
mod reed_solomon;
mod tables;
mod types;

pub mod decode;
pub mod encode;

pub use bit_grid::BitGrid;
pub use error::QrError;
pub use types::{
    DataMode, DecodedQr, EncodeOptions, ErrorCorrectionLevel, MaskPattern, QrCode, Version,
};

pub use decode::{decode_matrix, decode_modules};
pub use encode::{encode_bytes, encode_text};
