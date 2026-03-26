#[derive(Clone, Debug)]
pub(crate) struct BinaryImage {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) pixels: Vec<u8>,
    inverted: bool,
}

impl BinaryImage {
    #[inline]
    pub(crate) fn get(&self, x: usize, y: usize) -> bool {
        self.get_index(y * self.width + x)
    }

    #[inline]
    pub(crate) fn get_index(&self, index: usize) -> bool {
        (self.pixels[index] != 0) ^ self.inverted
    }

    pub(crate) fn into_inverted(mut self) -> Self {
        self.inverted = !self.inverted;
        self
    }
}

pub(crate) fn binarize(width: usize, height: usize, luma: &[u8]) -> BinaryImage {
    const BLOCK_SIZE: usize = 8;
    const MIN_DYNAMIC_RANGE: u32 = 24;

    let block_cols = width.div_ceil(BLOCK_SIZE);
    let block_rows = height.div_ceil(BLOCK_SIZE);
    let mut black_points = vec![0_u8; block_cols * block_rows];

    for block_y in 0..block_rows {
        let start_y = block_y * BLOCK_SIZE;
        let end_y = (start_y + BLOCK_SIZE).min(height);
        for block_x in 0..block_cols {
            let start_x = block_x * BLOCK_SIZE;
            let end_x = (start_x + BLOCK_SIZE).min(width);
            let mut sum = 0_u32;
            let mut min = 255_u8;
            let mut max = 0_u8;
            for y in start_y..end_y {
                let row = y * width;
                for x in start_x..end_x {
                    let value = luma[row + x];
                    sum += u32::from(value);
                    min = min.min(value);
                    max = max.max(value);
                }
            }
            let pixel_count = ((end_x - start_x) * (end_y - start_y)) as u32;
            let average = if u32::from(max) - u32::from(min) <= MIN_DYNAMIC_RANGE {
                (u32::from(min) / 2) as u8
            } else {
                (sum / pixel_count) as u8
            };
            black_points[block_y * block_cols + block_x] = average;
        }
    }

    let mut pixels = vec![0_u8; width * height];
    for block_y in 0..block_rows {
        let top = block_y.saturating_sub(2);
        let bottom = (block_y + 2).min(block_rows - 1);
        for block_x in 0..block_cols {
            let left = block_x.saturating_sub(2);
            let right = (block_x + 2).min(block_cols - 1);
            let mut threshold_sum = 0_u32;
            let mut samples = 0_u32;
            for sample_y in top..=bottom {
                for sample_x in left..=right {
                    threshold_sum += u32::from(black_points[sample_y * block_cols + sample_x]);
                    samples += 1;
                }
            }
            let threshold = (threshold_sum / samples) as u8;
            let start_y = block_y * BLOCK_SIZE;
            let end_y = (start_y + BLOCK_SIZE).min(height);
            let start_x = block_x * BLOCK_SIZE;
            let end_x = (start_x + BLOCK_SIZE).min(width);
            for y in start_y..end_y {
                let row = y * width;
                for x in start_x..end_x {
                    pixels[row + x] = u8::from(luma[row + x] <= threshold);
                }
            }
        }
    }

    BinaryImage {
        width,
        height,
        pixels,
        inverted: false,
    }
}
