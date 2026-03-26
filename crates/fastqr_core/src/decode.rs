use compact_str::CompactString;

use crate::{
    BitGrid,
    encode::{format_information_bits, mask_bit},
    error::QrError,
    reed_solomon,
    tables::{
        ECC_CODEWORDS_PER_BLOCK, NUM_ERROR_CORRECTION_BLOCKS, alignment_pattern_positions,
        alphanumeric_char, num_raw_data_modules,
    },
    types::{DecodedQr, ErrorCorrectionLevel, MaskPattern, Version},
};

pub fn decode_matrix(code: &crate::QrCode) -> Result<DecodedQr, QrError> {
    decode_modules(code.modules())
}

pub fn decode_modules(modules: &BitGrid) -> Result<DecodedQr, QrError> {
    let version = version_from_size(modules.size())?;
    let (error_correction, mask) = decode_format_information(modules)?;
    let function_modules = build_function_modules(version);
    let codewords = read_codewords(modules, &function_modules, mask);
    let data = deinterleave_and_verify(version, error_correction, &codewords)?;
    let (payload, text) = decode_payload(&data, version)?;

    Ok(DecodedQr {
        version,
        error_correction,
        mask,
        bytes: payload.into_boxed_slice(),
        text,
    })
}

fn version_from_size(size: usize) -> Result<Version, QrError> {
    if size < 21 || !(size - 17).is_multiple_of(4) {
        return Err(QrError::InvalidMatrixSize(size));
    }
    Version::new(((size - 17) / 4) as u8)
}

fn decode_format_information(
    modules: &BitGrid,
) -> Result<(ErrorCorrectionLevel, MaskPattern), QrError> {
    let size = modules.size();
    let first = [
        (8, 0),
        (8, 1),
        (8, 2),
        (8, 3),
        (8, 4),
        (8, 5),
        (8, 7),
        (8, 8),
        (7, 8),
        (5, 8),
        (4, 8),
        (3, 8),
        (2, 8),
        (1, 8),
        (0, 8),
    ];

    let mut first_bits = 0_u16;
    for (index, &(x, y)) in first.iter().enumerate() {
        first_bits |= u16::from(modules.get(x, y)) << index;
    }

    let mut second_bits = 0_u16;
    for index in 0..8 {
        second_bits |= u16::from(modules.get(size - 1 - index, 8)) << index;
    }
    for index in 8..15 {
        second_bits |= u16::from(modules.get(8, size - 15 + index)) << index;
    }

    let mut best = None;
    let mut best_distance = usize::MAX;
    for ecc in [
        ErrorCorrectionLevel::Low,
        ErrorCorrectionLevel::Medium,
        ErrorCorrectionLevel::Quartile,
        ErrorCorrectionLevel::High,
    ] {
        for mask_value in 0..8 {
            let mask = MaskPattern::new(mask_value).expect("valid mask");
            let expected = format_information_bits(ecc, mask);
            let distance =
                hamming_distance(first_bits, expected).min(hamming_distance(second_bits, expected));
            if distance < best_distance {
                best_distance = distance;
                best = Some((ecc, mask));
            }
        }
    }

    if best_distance <= 3 {
        best.ok_or(QrError::InvalidFormatInformation)
    } else {
        Err(QrError::InvalidFormatInformation)
    }
}

fn build_function_modules(version: Version) -> BitGrid {
    let size = version.size();
    let mut functions = BitGrid::new(size);

    mark_finder(&mut functions, 3, 3);
    mark_finder(&mut functions, size - 4, 3);
    mark_finder(&mut functions, 3, size - 4);

    for index in 8..size - 8 {
        functions.set(6, index, true);
        functions.set(index, 6, true);
    }

    let alignments = alignment_pattern_positions(version);
    for &x in &alignments {
        for &y in &alignments {
            let x = usize::from(x);
            let y = usize::from(y);
            let is_corner = (x == 6 && (y == 6 || y == size - 7)) || (x == size - 7 && y == 6);
            if !is_corner {
                mark_alignment(&mut functions, x, y);
            }
        }
    }

    for index in 0..=8 {
        if index != 6 {
            functions.set(8, index, true);
            functions.set(index, 8, true);
        }
    }
    for index in 0..8 {
        functions.set(size - 1 - index, 8, true);
    }
    for index in 0..7 {
        functions.set(8, size - 1 - index, true);
    }
    functions.set(8, size - 8, true);

    if version.value() >= 7 {
        for bit in 0..18 {
            let x = size - 11 + (bit % 3);
            let y = bit / 3;
            functions.set(x, y, true);
            functions.set(y, x, true);
        }
    }

    functions
}

fn read_codewords(modules: &BitGrid, functions: &BitGrid, mask: MaskPattern) -> Vec<u8> {
    let size = modules.size();
    let total_bits = num_raw_data_modules(version_from_size(size).expect("valid size")) & !7;
    let mut codewords = vec![0_u8; total_bits / 8];
    let mut bit_index = 0_usize;
    let mut right = size as isize - 1;
    while right >= 1 {
        if right == 6 {
            right = 5;
        }
        for vertical in 0..size {
            for delta in 0..2 {
                let x = (right - delta) as usize;
                let upward = ((right + 1) & 2) == 0;
                let y = if upward {
                    size - 1 - vertical
                } else {
                    vertical
                };
                if functions.get(x, y) {
                    continue;
                }
                if bit_index < total_bits {
                    let mut dark = modules.get(x, y);
                    if mask_bit(mask, x, y) {
                        dark = !dark;
                    }
                    if dark {
                        codewords[bit_index >> 3] |= 1 << (7 - (bit_index & 7));
                    }
                    bit_index += 1;
                }
            }
        }
        right -= 2;
    }
    codewords
}

fn deinterleave_and_verify(
    version: Version,
    error_correction: ErrorCorrectionLevel,
    codewords: &[u8],
) -> Result<Vec<u8>, QrError> {
    let num_blocks = usize::from(
        NUM_ERROR_CORRECTION_BLOCKS[error_correction.ordinal()][usize::from(version.value())],
    );
    let ecc_len = usize::from(
        ECC_CODEWORDS_PER_BLOCK[error_correction.ordinal()][usize::from(version.value())],
    );
    let raw_codewords = num_raw_data_modules(version) / 8;
    let short_block_count = num_blocks - raw_codewords % num_blocks;
    let short_block_len = raw_codewords / num_blocks;
    let pad_column = short_block_len - ecc_len;
    let padded_block_len = short_block_len + 1;

    let mut padded_blocks = vec![0_u8; num_blocks * padded_block_len];

    let mut offset = 0_usize;
    for column in 0..padded_block_len {
        for index in 0..num_blocks {
            if column != pad_column || index >= short_block_count {
                padded_blocks[index * padded_block_len + column] = codewords[offset];
                offset += 1;
            }
        }
    }

    let mut data = Vec::with_capacity(raw_codewords - num_blocks * ecc_len);
    for index in 0..num_blocks {
        let data_len = short_block_len - ecc_len + usize::from(index >= short_block_count);
        let block_offset = index * padded_block_len;
        let block = &padded_blocks[block_offset..block_offset + padded_block_len];
        let ecc_slice = if index < short_block_count {
            &block[data_len + 1..data_len + 1 + ecc_len]
        } else {
            &block[data_len..data_len + ecc_len]
        };
        if !reed_solomon::check_segments(&block[..data_len], ecc_slice, ecc_len) {
            return Err(QrError::Checksum);
        }
        data.extend_from_slice(&block[..data_len]);
    }

    Ok(data)
}

fn decode_payload(
    data: &[u8],
    version: Version,
) -> Result<(Vec<u8>, Option<CompactString>), QrError> {
    let mut reader = BitReader::new(data);
    let mut bytes = Vec::<u8>::with_capacity(data.len());
    let mut utf8_candidate = true;

    while reader.remaining() >= 4 {
        let mode = reader.read(4)?;
        if mode == 0 {
            break;
        }
        match mode {
            0x1 => {
                let count = reader.read_numeric_count(version)? as usize;
                decode_numeric(&mut reader, count, &mut bytes)?;
            }
            0x2 => {
                let count = reader.read_alphanumeric_count(version)? as usize;
                decode_alphanumeric(&mut reader, count, &mut bytes)?;
            }
            0x4 => {
                let count = reader.read_byte_count(version)? as usize;
                for _ in 0..count {
                    bytes.push(reader.read(8)? as u8);
                }
            }
            0x7 => {
                let assignment = read_eci(&mut reader)?;
                if assignment != 26 {
                    utf8_candidate = false;
                }
            }
            _ => return Err(QrError::UnsupportedMode(mode as u8)),
        }
    }

    if utf8_candidate {
        let text = core::str::from_utf8(&bytes).ok().map(CompactString::from);
        Ok((bytes, text))
    } else {
        Ok((bytes, None))
    }
}

fn decode_numeric(
    reader: &mut BitReader<'_>,
    count: usize,
    out: &mut Vec<u8>,
) -> Result<(), QrError> {
    let groups = count / 3;
    let rem = count % 3;
    for _ in 0..groups {
        let value = reader.read(10)? as usize;
        if value >= 1000 {
            return Err(QrError::Checksum);
        }
        out.push((value / 100) as u8 + b'0');
        out.push(((value / 10) % 10) as u8 + b'0');
        out.push((value % 10) as u8 + b'0');
    }
    if rem == 1 {
        let value = reader.read(4)? as u8;
        if value >= 10 {
            return Err(QrError::Checksum);
        }
        out.push(value + b'0');
    } else if rem == 2 {
        let value = reader.read(7)? as usize;
        if value >= 100 {
            return Err(QrError::Checksum);
        }
        out.push((value / 10) as u8 + b'0');
        out.push((value % 10) as u8 + b'0');
    }
    Ok(())
}

fn decode_alphanumeric(
    reader: &mut BitReader<'_>,
    count: usize,
    out: &mut Vec<u8>,
) -> Result<(), QrError> {
    let pairs = count / 2;
    let rem = count % 2;
    for _ in 0..pairs {
        let value = reader.read(11)? as usize;
        let first = (value / 45) as u8;
        let second = (value % 45) as u8;
        if first >= 45 || second >= 45 {
            return Err(QrError::Checksum);
        }
        out.push(alphanumeric_char(first));
        out.push(alphanumeric_char(second));
    }
    if rem == 1 {
        let value = reader.read(6)? as u8;
        if value >= 45 {
            return Err(QrError::Checksum);
        }
        out.push(alphanumeric_char(value));
    }
    Ok(())
}

fn read_eci(reader: &mut BitReader<'_>) -> Result<u32, QrError> {
    let first = reader.read(8)?;
    if (first & 0x80) == 0 {
        Ok(first)
    } else if (first & 0xC0) == 0x80 {
        Ok(((first & 0x3F) << 8) | reader.read(8)?)
    } else if (first & 0xE0) == 0xC0 {
        Ok(((first & 0x1F) << 16) | reader.read(16)?)
    } else {
        Err(QrError::Checksum)
    }
}

fn hamming_distance(left: u16, right: u16) -> usize {
    (left ^ right).count_ones() as usize
}

fn mark_finder(grid: &mut BitGrid, center_x: usize, center_y: usize) {
    for dy in -4..=4 {
        for dx in -4..=4 {
            let x = center_x as isize + dx;
            let y = center_y as isize + dy;
            if x < 0 || y < 0 || x >= grid.size() as isize || y >= grid.size() as isize {
                continue;
            }
            grid.set(x as usize, y as usize, true);
        }
    }
}

fn mark_alignment(grid: &mut BitGrid, center_x: usize, center_y: usize) {
    for dy in -2..=2 {
        for dx in -2..=2 {
            grid.set(
                (center_x as isize + dx) as usize,
                (center_y as isize + dy) as usize,
                true,
            );
        }
    }
}

struct BitReader<'a> {
    bytes: &'a [u8],
    bit_len: usize,
    bit_offset: usize,
}

impl<'a> BitReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            bit_len: bytes.len() * 8,
            bit_offset: 0,
        }
    }

    fn remaining(&self) -> usize {
        self.bit_len - self.bit_offset
    }

    fn read(&mut self, width: u8) -> Result<u32, QrError> {
        let width = usize::from(width);
        if self.bit_offset + width > self.bit_len {
            return Err(QrError::MissingTerminator);
        }
        let mut value = 0_u32;
        for _ in 0..width {
            let byte = self.bytes[self.bit_offset >> 3];
            let bit = (byte >> (7 - (self.bit_offset & 7))) & 1;
            value = (value << 1) | u32::from(bit);
            self.bit_offset += 1;
        }
        Ok(value)
    }

    fn read_numeric_count(&mut self, version: Version) -> Result<u32, QrError> {
        self.read(match version.value() {
            1..=9 => 10,
            10..=26 => 12,
            _ => 14,
        })
    }

    fn read_alphanumeric_count(&mut self, version: Version) -> Result<u32, QrError> {
        self.read(match version.value() {
            1..=9 => 9,
            10..=26 => 11,
            _ => 13,
        })
    }

    fn read_byte_count(&mut self, version: Version) -> Result<u32, QrError> {
        self.read(match version.value() {
            1..=9 => 8,
            _ => 16,
        })
    }
}
