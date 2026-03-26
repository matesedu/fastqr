use crate::gf256;

pub fn generator(degree: usize) -> Vec<u8> {
    let mut result = vec![0_u8; degree];
    result[degree - 1] = 1;

    let mut root = 1_u8;
    for _ in 0..degree {
        for index in 0..degree {
            result[index] = gf256::mul(result[index], root);
            if index + 1 < degree {
                result[index] ^= result[index + 1];
            }
        }
        root = gf256::mul(root, 0x02);
    }
    result
}

pub fn remainder_into(data: &[u8], generator: &[u8], result: &mut [u8]) {
    debug_assert_eq!(result.len(), generator.len());
    result.fill(0);
    for &value in data {
        let factor = value ^ result[0];
        result.rotate_left(1);
        if let Some(last) = result.last_mut() {
            *last = 0;
        }
        for (index, coefficient) in generator.iter().enumerate() {
            result[index] ^= gf256::mul(*coefficient, factor);
        }
    }
}

pub fn check_segments(first: &[u8], second: &[u8], ecc_len: usize) -> bool {
    if ecc_len == 0 {
        return true;
    }

    let mut root = 1_u8;
    for _ in 0..ecc_len {
        let mut acc = 0_u8;
        for &value in first {
            acc = gf256::mul(acc, root) ^ value;
        }
        for &value in second {
            acc = gf256::mul(acc, root) ^ value;
        }
        if acc != 0 {
            return false;
        }
        root = gf256::mul(root, 0x02);
    }
    true
}
