use smallvec::SmallVec;

use crate::types::{DataMode, ErrorCorrectionLevel, Version};

pub const ALPHANUMERIC_CHARSET: &[u8; 45] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";

pub const ECC_CODEWORDS_PER_BLOCK: [[u8; 41]; 4] = [
    [
        0, 7, 10, 15, 20, 26, 18, 20, 24, 30, 18, 20, 24, 26, 30, 22, 24, 28, 30, 28, 28, 28, 28,
        30, 30, 26, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ],
    [
        0, 10, 16, 26, 18, 24, 16, 18, 22, 22, 26, 30, 22, 22, 24, 24, 28, 28, 26, 26, 26, 26, 28,
        28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28,
    ],
    [
        0, 13, 22, 18, 26, 18, 24, 18, 22, 20, 24, 28, 26, 24, 20, 30, 24, 28, 28, 26, 30, 28, 30,
        30, 30, 30, 28, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ],
    [
        0, 17, 28, 22, 16, 22, 28, 26, 26, 24, 28, 24, 28, 22, 24, 24, 30, 28, 28, 26, 28, 30, 24,
        30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30, 30,
    ],
];

pub const NUM_ERROR_CORRECTION_BLOCKS: [[u8; 41]; 4] = [
    [
        0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 4, 4, 4, 4, 4, 6, 6, 6, 6, 7, 8, 8, 9, 9, 10, 12, 12, 12, 13,
        14, 15, 16, 17, 18, 19, 19, 20, 21, 22, 24, 25,
    ],
    [
        0, 1, 1, 1, 2, 2, 4, 4, 4, 5, 5, 5, 8, 9, 9, 10, 10, 11, 13, 14, 16, 17, 17, 18, 20, 21,
        23, 25, 26, 28, 29, 31, 33, 35, 37, 38, 40, 43, 45, 47, 49,
    ],
    [
        0, 1, 1, 2, 2, 4, 4, 6, 6, 8, 8, 8, 10, 12, 16, 12, 17, 16, 18, 21, 20, 23, 23, 25, 27, 29,
        34, 34, 35, 38, 40, 43, 45, 48, 51, 53, 56, 59, 62, 65, 68,
    ],
    [
        0, 1, 1, 2, 4, 4, 4, 5, 6, 8, 8, 11, 11, 16, 16, 18, 16, 19, 21, 25, 25, 25, 34, 30, 32,
        35, 37, 40, 42, 45, 48, 51, 54, 57, 60, 63, 66, 70, 74, 77, 81,
    ],
];

#[inline]
pub fn alphanumeric_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'Z' => Some(byte - b'A' + 10),
        b' ' => Some(36),
        b'$' => Some(37),
        b'%' => Some(38),
        b'*' => Some(39),
        b'+' => Some(40),
        b'-' => Some(41),
        b'.' => Some(42),
        b'/' => Some(43),
        b':' => Some(44),
        _ => None,
    }
}

#[inline]
pub fn alphanumeric_char(value: u8) -> u8 {
    ALPHANUMERIC_CHARSET[value as usize]
}

#[inline]
pub fn num_raw_data_modules(version: Version) -> usize {
    let version = usize::from(version.value());
    let mut result = (16 * version + 128) * version + 64;
    if version >= 2 {
        let alignment_count = version / 7 + 2;
        result -= (25 * alignment_count - 10) * alignment_count - 55;
        if version >= 7 {
            result -= 36;
        }
    }
    result
}

#[inline]
pub fn num_data_codewords(version: Version, ecc: ErrorCorrectionLevel) -> usize {
    num_raw_data_modules(version) / 8
        - usize::from(ECC_CODEWORDS_PER_BLOCK[ecc.ordinal()][usize::from(version.value())])
            * usize::from(NUM_ERROR_CORRECTION_BLOCKS[ecc.ordinal()][usize::from(version.value())])
}

pub fn alignment_pattern_positions(version: Version) -> SmallVec<[u8; 7]> {
    let version = usize::from(version.value());
    let mut result = SmallVec::<[u8; 7]>::new();
    if version == 1 {
        return result;
    }

    let alignment_count = version / 7 + 2;
    let step = if version == 32 {
        26
    } else {
        ((version * 4 + alignment_count * 2 + 1) / (alignment_count * 2 - 2)) * 2
    };
    let size = version * 4 + 17;
    let last = size - 7;

    result.push(6);
    for index in 1..(alignment_count - 1) {
        let offset = (alignment_count - 1 - index) * step;
        result.push((last - offset) as u8);
    }
    result.push(last as u8);
    result
}

#[inline]
pub fn payload_bit_len(mode: DataMode, data_len: usize) -> usize {
    match mode {
        DataMode::Numeric => {
            let groups = data_len / 3;
            let rem = data_len % 3;
            groups * 10
                + match rem {
                    0 => 0,
                    1 => 4,
                    _ => 7,
                }
        }
        DataMode::Alphanumeric => (data_len / 2) * 11 + usize::from((data_len & 1) == 1) * 6,
        DataMode::Byte => data_len * 8,
    }
}

#[inline]
pub fn total_bit_len(version: Version, mode: DataMode, data_len: usize) -> usize {
    4 + usize::from(mode.char_count_bits(version)) + payload_bit_len(mode, data_len)
}
