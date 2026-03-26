use core::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QrError {
    DataTooLong,
    InvalidCharacter,
    InvalidFormatInformation,
    InvalidMask(u8),
    InvalidMatrixSize(usize),
    InvalidVersion(u8),
    MissingTerminator,
    UnsupportedMode(u8),
    Utf8,
    Checksum,
}

impl fmt::Display for QrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DataTooLong => f.write_str("payload does not fit the selected QR configuration"),
            Self::InvalidCharacter => {
                f.write_str("payload contains characters not supported by the selected mode")
            }
            Self::InvalidFormatInformation => {
                f.write_str("QR format information could not be decoded")
            }
            Self::InvalidMask(mask) => write!(f, "invalid mask pattern: {mask}"),
            Self::InvalidMatrixSize(size) => write!(f, "invalid QR matrix size: {size}"),
            Self::InvalidVersion(version) => write!(f, "invalid QR version: {version}"),
            Self::MissingTerminator => f.write_str("QR payload is truncated"),
            Self::UnsupportedMode(mode) => write!(f, "unsupported QR mode: {mode:#04b}"),
            Self::Utf8 => f.write_str("decoded payload is not valid UTF-8"),
            Self::Checksum => f.write_str("error correction check failed"),
        }
    }
}

impl std::error::Error for QrError {}
