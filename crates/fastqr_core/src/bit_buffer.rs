#[derive(Clone, Debug, Default)]
pub struct BitBuffer {
    bytes: Vec<u8>,
    bit_len: usize,
}

impl BitBuffer {
    pub fn with_capacity_bits(bit_len: usize) -> Self {
        let bytes = Vec::with_capacity((bit_len + 7) >> 3);
        Self { bytes, bit_len: 0 }
    }

    #[inline]
    pub fn bit_len(&self) -> usize {
        self.bit_len
    }

    pub fn append_bits(&mut self, value: u32, bit_count: u8) {
        debug_assert!(bit_count <= 31 || (value >> bit_count) == 0);
        if bit_count == 0 {
            return;
        }

        let new_bit_len = self.bit_len + usize::from(bit_count);
        let needed = (new_bit_len + 7) >> 3;
        if needed > self.bytes.len() {
            self.bytes.resize(needed, 0);
        }

        let mut remaining = bit_count;
        while remaining > 0 {
            let byte_index = self.bit_len >> 3;
            let bit_index = self.bit_len & 7;
            let space = 8 - bit_index;
            let take = space.min(usize::from(remaining));
            let shift = usize::from(remaining) - take;
            let mask = ((value >> shift) & ((1 << take) - 1)) as u8;
            self.bytes[byte_index] |= mask << (space - take);
            self.bit_len += take;
            remaining -= take as u8;
        }
    }

    pub fn append_byte(&mut self, value: u8) {
        self.append_bits(u32::from(value), 8);
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}
