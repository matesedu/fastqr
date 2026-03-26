use crate::{
    bit_buffer::BitBuffer,
    bit_grid::BitGrid,
    error::QrError,
    reed_solomon,
    tables::{
        ECC_CODEWORDS_PER_BLOCK, NUM_ERROR_CORRECTION_BLOCKS, alignment_pattern_positions,
        alphanumeric_value, num_data_codewords, num_raw_data_modules, total_bit_len,
    },
    types::{DataMode, EncodeOptions, ErrorCorrectionLevel, MaskPattern, QrCode, Version},
};

struct MatrixBuilder {
    version: Version,
    size: usize,
    modules: BitGrid,
    function_modules: BitGrid,
}

impl MatrixBuilder {
    fn new(version: Version) -> Self {
        let size = version.size();
        Self {
            version,
            size,
            modules: BitGrid::new(size),
            function_modules: BitGrid::new(size),
        }
    }

    #[inline]
    fn set_function(&mut self, x: usize, y: usize, dark: bool) {
        self.modules.set(x, y, dark);
        self.function_modules.set(x, y, true);
    }

    fn draw_function_patterns(&mut self) {
        self.draw_finder(3, 3);
        self.draw_finder(self.size - 4, 3);
        self.draw_finder(3, self.size - 4);

        for index in 8..self.size - 8 {
            let dark = (index & 1) == 0;
            self.set_function(6, index, dark);
            self.set_function(index, 6, dark);
        }

        let alignments = alignment_pattern_positions(self.version);
        for &x in &alignments {
            for &y in &alignments {
                let x = usize::from(x);
                let y = usize::from(y);
                let is_corner =
                    (x == 6 && (y == 6 || y == self.size - 7)) || (x == self.size - 7 && y == 6);
                if !is_corner {
                    self.draw_alignment(x, y);
                }
            }
        }

        for index in 0..=8 {
            if index != 6 {
                self.function_modules.set(8, index, true);
                self.function_modules.set(index, 8, true);
            }
        }
        for index in 0..8 {
            self.function_modules.set(self.size - 1 - index, 8, true);
        }
        for index in 0..7 {
            self.function_modules.set(8, self.size - 1 - index, true);
        }
        self.function_modules.set(8, self.size - 8, true);
        self.set_function(8, self.size - 8, true);

        if self.version.value() >= 7 {
            let bits = version_information_bits(self.version);
            for bit in 0..18 {
                let dark = ((bits >> bit) & 1) != 0;
                let x = self.size - 11 + (bit % 3);
                let y = bit / 3;
                self.set_function(x, y, dark);
                self.set_function(y, x, dark);
            }
        }
    }

    fn draw_finder(&mut self, center_x: usize, center_y: usize) {
        for dy in -4..=4 {
            for dx in -4..=4 {
                let x = center_x as isize + dx;
                let y = center_y as isize + dy;
                if x < 0 || y < 0 || x >= self.size as isize || y >= self.size as isize {
                    continue;
                }
                let dist = dx.unsigned_abs().max(dy.unsigned_abs());
                self.set_function(x as usize, y as usize, dist != 2 && dist != 4);
            }
        }
    }

    fn draw_alignment(&mut self, center_x: usize, center_y: usize) {
        for dy in -2_i32..=2 {
            for dx in -2_i32..=2 {
                let dist = dx.unsigned_abs().max(dy.unsigned_abs());
                self.set_function(
                    (center_x as isize + dx as isize) as usize,
                    (center_y as isize + dy as isize) as usize,
                    dist != 1,
                );
            }
        }
    }

    fn draw_codewords(&mut self, codewords: &[u8]) {
        let mut bit_index = 0_usize;
        let mut right = self.size as isize - 1;
        while right >= 1 {
            if right == 6 {
                right = 5;
            }
            for vertical in 0..self.size {
                for delta in 0..2 {
                    let x = (right - delta) as usize;
                    let upward = ((right + 1) & 2) == 0;
                    let y = if upward {
                        self.size - 1 - vertical
                    } else {
                        vertical
                    };
                    if self.function_modules.get(x, y) {
                        continue;
                    }
                    if bit_index < codewords.len() * 8 {
                        let bit = ((codewords[bit_index >> 3] >> (7 - (bit_index & 7))) & 1) != 0;
                        self.modules.set(x, y, bit);
                        bit_index += 1;
                    }
                }
            }
            right -= 2;
        }
    }

    fn apply_mask(&mut self, mask: MaskPattern) {
        for y in 0..self.size {
            for x in 0..self.size {
                if !self.function_modules.get(x, y) && mask_bit(mask, x, y) {
                    self.modules.invert(x, y);
                }
            }
        }
    }

    fn draw_format_bits(&mut self, ecc: ErrorCorrectionLevel, mask: MaskPattern) {
        let bits = format_information_bits(ecc, mask);
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
        for (index, &(x, y)) in first.iter().enumerate() {
            self.set_function(x, y, ((bits >> index) & 1) != 0);
        }

        for index in 0..8 {
            self.set_function(self.size - 1 - index, 8, ((bits >> index) & 1) != 0);
        }
        for index in 8..15 {
            self.set_function(8, self.size - 15 + index, ((bits >> index) & 1) != 0);
        }
    }

    fn penalty_score(&self) -> usize {
        const PATTERN_1: u16 = 0b10111010000;
        const PATTERN_2: u16 = 0b00001011101;
        let mut score = 0_usize;

        for y in 0..self.size {
            let mut run_color = self.modules.get(0, y);
            let mut run_len = 1_usize;
            for x in 1..self.size {
                let dark = self.modules.get(x, y);
                if dark == run_color {
                    run_len += 1;
                } else {
                    if run_len >= 5 {
                        score += run_len - 2;
                    }
                    run_color = dark;
                    run_len = 1;
                }
            }
            if run_len >= 5 {
                score += run_len - 2;
            }
            if self.size >= 11 {
                for x in 0..=self.size - 11 {
                    let pattern = row_pattern(&self.modules, x, y);
                    if pattern == PATTERN_1 || pattern == PATTERN_2 {
                        score += 40;
                    }
                }
            }
        }

        for x in 0..self.size {
            let mut run_color = self.modules.get(x, 0);
            let mut run_len = 1_usize;
            for y in 1..self.size {
                let dark = self.modules.get(x, y);
                if dark == run_color {
                    run_len += 1;
                } else {
                    if run_len >= 5 {
                        score += run_len - 2;
                    }
                    run_color = dark;
                    run_len = 1;
                }
            }
            if run_len >= 5 {
                score += run_len - 2;
            }
            if self.size >= 11 {
                for y in 0..=self.size - 11 {
                    let pattern = column_pattern(&self.modules, x, y);
                    if pattern == PATTERN_1 || pattern == PATTERN_2 {
                        score += 40;
                    }
                }
            }
        }

        for y in 0..self.size - 1 {
            for x in 0..self.size - 1 {
                let dark = self.modules.get(x, y);
                if dark == self.modules.get(x + 1, y)
                    && dark == self.modules.get(x, y + 1)
                    && dark == self.modules.get(x + 1, y + 1)
                {
                    score += 3;
                }
            }
        }

        let total = self.size * self.size;
        let dark = self.modules.count_dark();
        let imbalance = (dark * 20).abs_diff(total * 10);
        score + imbalance.div_ceil(total).saturating_sub(1) * 10
    }
}

pub fn encode_text(text: &str, options: EncodeOptions) -> Result<QrCode, QrError> {
    let mode = classify_text(text.as_bytes());
    encode_payload(mode, text.as_bytes(), options)
}

pub fn encode_bytes(data: &[u8], options: EncodeOptions) -> Result<QrCode, QrError> {
    encode_payload(DataMode::Byte, data, options)
}

fn encode_payload(mode: DataMode, data: &[u8], options: EncodeOptions) -> Result<QrCode, QrError> {
    if options.min_version.value() > options.max_version.value() {
        return Err(QrError::InvalidVersion(options.min_version.value()));
    }

    let version = choose_version(mode, data.len(), options)?;
    let total_bits = total_bit_len(version, mode, data.len());
    let ecc = choose_error_correction(version, total_bits, options);
    let data_codewords = build_data_codewords(version, ecc, mode, data)?;
    let codewords = add_error_correction_and_interleave(version, ecc, &data_codewords);
    let mut matrix = MatrixBuilder::new(version);
    matrix.draw_function_patterns();
    matrix.draw_codewords(&codewords);

    let mask = if let Some(mask) = options.mask {
        matrix.apply_mask(mask);
        matrix.draw_format_bits(ecc, mask);
        mask
    } else {
        let mut best_mask = MaskPattern::new(0).expect("valid mask");
        let mut best_score = usize::MAX;
        for value in 0..8 {
            let mask = MaskPattern::new(value).expect("valid mask");
            matrix.apply_mask(mask);
            matrix.draw_format_bits(ecc, mask);
            let score = matrix.penalty_score();
            if score < best_score {
                best_score = score;
                best_mask = mask;
            }
            matrix.apply_mask(mask);
        }
        matrix.apply_mask(best_mask);
        matrix.draw_format_bits(ecc, best_mask);
        best_mask
    };

    Ok(QrCode::new(version, ecc, mask, matrix.modules))
}

pub(crate) fn build_data_codewords(
    version: Version,
    ecc: ErrorCorrectionLevel,
    mode: DataMode,
    data: &[u8],
) -> Result<Vec<u8>, QrError> {
    let data_codewords = num_data_codewords(version, ecc);
    let capacity_bits = data_codewords * 8;

    let char_count_limit = 1_usize << mode.char_count_bits(version);
    if data.len() >= char_count_limit {
        return Err(QrError::DataTooLong);
    }

    let mut bits = BitBuffer::with_capacity_bits(capacity_bits);
    bits.append_bits(u32::from(mode.mode_bits()), 4);
    bits.append_bits(data.len() as u32, mode.char_count_bits(version));
    append_payload(&mut bits, mode, data)?;

    let terminator = (capacity_bits - bits.bit_len()).min(4);
    bits.append_bits(0, terminator as u8);
    while (bits.bit_len() & 7) != 0 {
        bits.append_bits(0, 1);
    }

    let mut pad = 0xEC_u8;
    while bits.bit_len() < capacity_bits {
        bits.append_byte(pad);
        pad ^= 0xFD;
    }

    Ok(bits.into_bytes())
}

fn classify_text(data: &[u8]) -> DataMode {
    if data.iter().all(u8::is_ascii_digit) {
        DataMode::Numeric
    } else if data.iter().all(|byte| alphanumeric_value(*byte).is_some()) {
        DataMode::Alphanumeric
    } else {
        DataMode::Byte
    }
}

fn choose_version(
    mode: DataMode,
    data_len: usize,
    options: EncodeOptions,
) -> Result<Version, QrError> {
    for value in options.min_version.value()..=options.max_version.value() {
        let version = Version::new(value)?;
        let total_bits = total_bit_len(version, mode, data_len);
        let capacity = num_data_codewords(version, options.min_error_correction) * 8;
        if total_bits <= capacity {
            return Ok(version);
        }
    }
    Err(QrError::DataTooLong)
}

fn choose_error_correction(
    version: Version,
    total_bits: usize,
    options: EncodeOptions,
) -> ErrorCorrectionLevel {
    let mut ecc = options.min_error_correction;
    if !options.boost_error_correction {
        return ecc;
    }
    while let Some(next) = ecc.higher() {
        if total_bits <= num_data_codewords(version, next) * 8 {
            ecc = next;
        } else {
            break;
        }
    }
    ecc
}

fn append_payload(bits: &mut BitBuffer, mode: DataMode, data: &[u8]) -> Result<(), QrError> {
    match mode {
        DataMode::Numeric => {
            for chunk in data.chunks(3) {
                let mut value = 0_u32;
                for &byte in chunk {
                    if !byte.is_ascii_digit() {
                        return Err(QrError::InvalidCharacter);
                    }
                    value = value * 10 + u32::from(byte - b'0');
                }
                let width = match chunk.len() {
                    1 => 4,
                    2 => 7,
                    _ => 10,
                };
                bits.append_bits(value, width);
            }
        }
        DataMode::Alphanumeric => {
            for chunk in data.chunks(2) {
                let first =
                    u32::from(alphanumeric_value(chunk[0]).ok_or(QrError::InvalidCharacter)?);
                if chunk.len() == 2 {
                    let second =
                        u32::from(alphanumeric_value(chunk[1]).ok_or(QrError::InvalidCharacter)?);
                    bits.append_bits(first * 45 + second, 11);
                } else {
                    bits.append_bits(first, 6);
                }
            }
        }
        DataMode::Byte => {
            for &byte in data {
                bits.append_byte(byte);
            }
        }
    }
    Ok(())
}

pub(crate) fn add_error_correction_and_interleave(
    version: Version,
    ecc: ErrorCorrectionLevel,
    data: &[u8],
) -> Vec<u8> {
    let num_blocks =
        usize::from(NUM_ERROR_CORRECTION_BLOCKS[ecc.ordinal()][usize::from(version.value())]);
    let ecc_len = usize::from(ECC_CODEWORDS_PER_BLOCK[ecc.ordinal()][usize::from(version.value())]);
    let raw_codewords = num_raw_data_modules(version) / 8;
    let short_block_count = num_blocks - raw_codewords % num_blocks;
    let short_block_len = raw_codewords / num_blocks;
    let padded_block_len = short_block_len + 1;

    let generator = reed_solomon::generator(ecc_len);
    let mut blocks = vec![0_u8; num_blocks * padded_block_len];
    let mut ecc_bytes = vec![0_u8; ecc_len];
    let mut data_offset = 0_usize;
    for index in 0..num_blocks {
        let data_len = short_block_len - ecc_len + usize::from(index >= short_block_count);
        let block_offset = index * padded_block_len;
        let block = &mut blocks[block_offset..block_offset + padded_block_len];
        block[..data_len].copy_from_slice(&data[data_offset..data_offset + data_len]);
        data_offset += data_len;
        reed_solomon::remainder_into(&block[..data_len], &generator, &mut ecc_bytes);
        let ecc_offset = data_len + usize::from(index < short_block_count);
        block[ecc_offset..ecc_offset + ecc_len].copy_from_slice(&ecc_bytes);
    }

    let mut result = Vec::with_capacity(raw_codewords);
    let pad_column = short_block_len - ecc_len;
    for column in 0..padded_block_len {
        for index in 0..num_blocks {
            if column != pad_column || index >= short_block_count {
                result.push(blocks[index * padded_block_len + column]);
            }
        }
    }
    result
}

pub(crate) fn format_information_bits(ecc: ErrorCorrectionLevel, mask: MaskPattern) -> u16 {
    let data = (u16::from(ecc.format_bits()) << 3) | u16::from(mask.value());
    let mut rem = data;
    for _ in 0..10 {
        rem = (rem << 1) ^ (((rem >> 9) & 1) * 0x537);
    }
    ((data << 10) | rem) ^ 0x5412
}

fn version_information_bits(version: Version) -> u32 {
    let data = u32::from(version.value());
    let mut rem = data;
    for _ in 0..12 {
        rem = (rem << 1) ^ (((rem >> 11) & 1) * 0x1F25);
    }
    (data << 12) | rem
}

pub(crate) fn mask_bit(mask: MaskPattern, x: usize, y: usize) -> bool {
    match mask.value() {
        0 => ((x + y) & 1) == 0,
        1 => (y & 1) == 0,
        2 => x.is_multiple_of(3),
        3 => (x + y).is_multiple_of(3),
        4 => ((y >> 1) + (x / 3)) & 1 == 0,
        5 => {
            let product = x * y;
            product % 2 + product % 3 == 0
        }
        6 => {
            let product = x * y;
            ((product % 2 + product % 3) & 1) == 0
        }
        7 => {
            let product = x * y;
            (((x + y) & 1) + (product % 3)) & 1 == 0
        }
        _ => false,
    }
}

fn row_pattern(grid: &BitGrid, x: usize, y: usize) -> u16 {
    let mut bits = 0_u16;
    for offset in 0..11 {
        bits = (bits << 1) | u16::from(grid.get(x + offset, y));
    }
    bits
}

fn column_pattern(grid: &BitGrid, x: usize, y: usize) -> u16 {
    let mut bits = 0_u16;
    for offset in 0..11 {
        bits = (bits << 1) | u16::from(grid.get(x, y + offset));
    }
    bits
}

#[cfg(test)]
mod tests {
    use crate::{EncodeOptions, decode_modules};

    use super::{encode_bytes, encode_text};

    #[test]
    fn roundtrip_numeric() {
        let code = encode_text("01234567", EncodeOptions::default()).expect("encodes");
        let decoded = decode_modules(code.modules()).expect("decodes");
        assert_eq!(&decoded.bytes[..], b"01234567");
        assert_eq!(decoded.text.as_deref(), Some("01234567"));
    }

    #[test]
    fn roundtrip_alphanumeric() {
        let code = encode_text("HELLO WORLD", EncodeOptions::default()).expect("encodes");
        let decoded = decode_modules(code.modules()).expect("decodes");
        assert_eq!(decoded.text.as_deref(), Some("HELLO WORLD"));
    }

    #[test]
    fn roundtrip_bytes() {
        let payload = "fastqr:こんにちは".as_bytes();
        let code = encode_bytes(payload, EncodeOptions::default()).expect("encodes");
        let decoded = decode_modules(code.modules()).expect("decodes");
        assert_eq!(&decoded.bytes[..], payload);
    }

    #[test]
    fn roundtrip_larger_version() {
        let code =
            encode_text("fastqr image roundtrip", EncodeOptions::default()).expect("encodes");
        let decoded = decode_modules(code.modules()).expect("decodes");
        assert_eq!(decoded.text.as_deref(), Some("fastqr image roundtrip"));
    }

    #[test]
    fn finder_edges_are_not_overwritten_by_timing_patterns() {
        let code = encode_text("FASTQR", EncodeOptions::default()).expect("encodes");

        for coordinate in 0..=6 {
            assert!(code.module(coordinate, 6), "top-left finder bottom edge at ({coordinate}, 6)");
            assert!(code.module(6, coordinate), "top-left finder right edge at (6, {coordinate})");
        }
        assert!(!code.module(7, 6), "separator next to top-left finder must stay light");
        assert!(!code.module(6, 7), "separator below top-left finder must stay light");

        let size = code.size();
        for coordinate in size - 7..size {
            assert!(
                code.module(coordinate, 6),
                "top-right finder bottom edge at ({coordinate}, 6)"
            );
            assert!(
                code.module(6, coordinate),
                "bottom-left finder right edge at (6, {coordinate})"
            );
        }
    }
}
