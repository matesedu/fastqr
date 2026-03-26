#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BitGrid {
    size: usize,
    words: Box<[u64]>,
}

impl BitGrid {
    pub fn new(size: usize) -> Self {
        let word_len = (size * size + 63) >> 6;
        Self {
            size,
            words: vec![0; word_len].into_boxed_slice(),
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> bool {
        debug_assert!(x < self.size && y < self.size);
        let bit_index = y * self.size + x;
        ((self.words[bit_index >> 6] >> (bit_index & 63)) & 1) != 0
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, dark: bool) {
        debug_assert!(x < self.size && y < self.size);
        let bit_index = y * self.size + x;
        let word_index = bit_index >> 6;
        let mask = 1_u64 << (bit_index & 63);
        if dark {
            self.words[word_index] |= mask;
        } else {
            self.words[word_index] &= !mask;
        }
    }

    #[inline]
    pub fn invert(&mut self, x: usize, y: usize) {
        debug_assert!(x < self.size && y < self.size);
        let bit_index = y * self.size + x;
        self.words[bit_index >> 6] ^= 1_u64 << (bit_index & 63);
    }

    pub fn count_dark(&self) -> usize {
        self.words
            .iter()
            .map(|word| word.count_ones() as usize)
            .sum()
    }

    pub fn fill_row_major_bytes(&self, bytes: &mut [u8]) {
        assert_eq!(bytes.len(), self.size * self.size);
        for (index, byte) in bytes.iter_mut().enumerate() {
            *byte = ((self.words[index >> 6] >> (index & 63)) & 1) as u8;
        }
    }

    pub fn to_row_major_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0_u8; self.size * self.size];
        self.fill_row_major_bytes(&mut bytes);
        bytes
    }
}
