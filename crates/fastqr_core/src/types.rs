use compact_str::CompactString;

use crate::{bit_grid::BitGrid, error::QrError};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Version(u8);

impl Version {
    pub const MIN: Self = Self(1);
    pub const MAX: Self = Self(40);

    pub fn new(value: u8) -> Result<Self, QrError> {
        if (1..=40).contains(&value) {
            Ok(Self(value))
        } else {
            Err(QrError::InvalidVersion(value))
        }
    }

    #[inline]
    pub const fn value(self) -> u8 {
        self.0
    }

    #[inline]
    pub const fn size(self) -> usize {
        self.0 as usize * 4 + 17
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MaskPattern(u8);

impl MaskPattern {
    pub fn new(value: u8) -> Result<Self, QrError> {
        if value < 8 {
            Ok(Self(value))
        } else {
            Err(QrError::InvalidMask(value))
        }
    }

    #[inline]
    pub const fn value(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCorrectionLevel {
    Low,
    Medium,
    Quartile,
    High,
}

impl ErrorCorrectionLevel {
    #[inline]
    pub const fn ordinal(self) -> usize {
        match self {
            Self::Low => 0,
            Self::Medium => 1,
            Self::Quartile => 2,
            Self::High => 3,
        }
    }

    #[inline]
    pub const fn format_bits(self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 0,
            Self::Quartile => 3,
            Self::High => 2,
        }
    }

    pub fn from_format_bits(bits: u8) -> Result<Self, QrError> {
        match bits {
            1 => Ok(Self::Low),
            0 => Ok(Self::Medium),
            3 => Ok(Self::Quartile),
            2 => Ok(Self::High),
            _ => Err(QrError::InvalidFormatInformation),
        }
    }

    #[inline]
    pub const fn higher(self) -> Option<Self> {
        match self {
            Self::Low => Some(Self::Medium),
            Self::Medium => Some(Self::Quartile),
            Self::Quartile => Some(Self::High),
            Self::High => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DataMode {
    Numeric,
    Alphanumeric,
    Byte,
}

impl DataMode {
    #[inline]
    pub const fn mode_bits(self) -> u8 {
        match self {
            Self::Numeric => 0x1,
            Self::Alphanumeric => 0x2,
            Self::Byte => 0x4,
        }
    }

    #[inline]
    pub const fn char_count_bits(self, version: Version) -> u8 {
        let group = match version.value() {
            1..=9 => 0,
            10..=26 => 1,
            _ => 2,
        };
        match self {
            Self::Numeric => [10, 12, 14][group],
            Self::Alphanumeric => [9, 11, 13][group],
            Self::Byte => [8, 16, 16][group],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EncodeOptions {
    pub min_version: Version,
    pub max_version: Version,
    pub min_error_correction: ErrorCorrectionLevel,
    pub boost_error_correction: bool,
    pub mask: Option<MaskPattern>,
}

impl Default for EncodeOptions {
    fn default() -> Self {
        Self {
            min_version: Version::MIN,
            max_version: Version::MAX,
            min_error_correction: ErrorCorrectionLevel::Medium,
            boost_error_correction: true,
            mask: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QrCode {
    version: Version,
    error_correction: ErrorCorrectionLevel,
    mask: MaskPattern,
    modules: BitGrid,
}

impl QrCode {
    pub(crate) fn new(
        version: Version,
        error_correction: ErrorCorrectionLevel,
        mask: MaskPattern,
        modules: BitGrid,
    ) -> Self {
        Self {
            version,
            error_correction,
            mask,
            modules,
        }
    }

    #[inline]
    pub fn version(&self) -> Version {
        self.version
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.modules.size()
    }

    #[inline]
    pub fn error_correction(&self) -> ErrorCorrectionLevel {
        self.error_correction
    }

    #[inline]
    pub fn mask(&self) -> MaskPattern {
        self.mask
    }

    #[inline]
    pub fn module(&self, x: usize, y: usize) -> bool {
        self.modules.get(x, y)
    }

    #[inline]
    pub fn modules(&self) -> &BitGrid {
        &self.modules
    }

    pub fn modules_row_major(&self) -> Vec<u8> {
        self.modules.to_row_major_bytes()
    }

    pub fn to_svg_string(&self, border: usize) -> String {
        let size = self.size() + border * 2;
        let modules = self.modules();
        let mut path = String::with_capacity(modules.count_dark() * 8 + self.size());
        for y in 0..self.size() {
            let mut x = 0;
            while x < self.size() {
                if !modules.get(x, y) {
                    x += 1;
                    continue;
                }
                let start = x;
                while x < self.size() && modules.get(x, y) {
                    x += 1;
                }
                let _ = core::fmt::Write::write_fmt(
                    &mut path,
                    format_args!(
                        "M{},{}h{}v1h-{}z",
                        start + border,
                        y + border,
                        x - start,
                        x - start
                    ),
                );
            }
        }
        let mut svg = String::with_capacity(path.len() + 128);
        let _ = core::fmt::Write::write_fmt(
            &mut svg,
            format_args!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {size} {size}\" shape-rendering=\"crispEdges\"><rect width=\"100%\" height=\"100%\" fill=\"#fff\"/><path d=\"{path}\" fill=\"#000\"/></svg>"
            ),
        );
        svg
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedQr {
    pub version: Version,
    pub error_correction: ErrorCorrectionLevel,
    pub mask: MaskPattern,
    pub bytes: Box<[u8]>,
    pub text: Option<CompactString>,
}
