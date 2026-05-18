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

    syndromes(first.iter().chain(second), ecc_len)
        .into_iter()
        .all(|syndrome| syndrome == 0)
}

pub fn correct(codewords: &mut [u8], ecc_len: usize) -> bool {
    if ecc_len == 0 {
        return true;
    }

    let syndrome_values = syndromes(codewords.iter(), ecc_len);
    if syndrome_values.iter().all(|&syndrome| syndrome == 0) {
        return true;
    }

    let locator = error_locator(&syndrome_values);
    let error_count = locator.len().saturating_sub(1);
    if error_count == 0 || error_count > ecc_len / 2 {
        return false;
    }

    let Some(error_locations) = find_error_locations(&locator, codewords.len()) else {
        return false;
    };
    if error_locations.len() != error_count {
        return false;
    }

    let Some(magnitudes) = error_magnitudes(&syndrome_values, &error_locations, codewords.len())
    else {
        return false;
    };

    for (position, magnitude) in error_locations.into_iter().zip(magnitudes) {
        codewords[position] ^= magnitude;
    }

    check_segments(codewords, &[], ecc_len)
}

fn syndromes<'a>(codewords: impl Iterator<Item = &'a u8> + Clone, ecc_len: usize) -> Vec<u8> {
    let mut result = Vec::with_capacity(ecc_len);
    let mut root = 1_u8;
    for _ in 0..ecc_len {
        let mut acc = 0_u8;
        for &value in codewords.clone() {
            acc = gf256::mul(acc, root) ^ value;
        }
        result.push(acc);
        root = gf256::mul(root, 0x02);
    }
    result
}

fn error_locator(syndromes: &[u8]) -> Vec<u8> {
    let mut locator = vec![1_u8];
    let mut previous = vec![1_u8];
    let mut previous_discrepancy = 1_u8;
    let mut gap = 1_usize;
    let mut error_count = 0_usize;

    for index in 0..syndromes.len() {
        let mut discrepancy = syndromes[index];
        for coefficient in 1..=error_count {
            discrepancy ^= gf256::mul(locator[coefficient], syndromes[index - coefficient]);
        }

        if discrepancy == 0 {
            gap += 1;
            continue;
        }

        let old_locator = locator.clone();
        let scale = gf256::div(discrepancy, previous_discrepancy);
        if locator.len() < previous.len() + gap {
            locator.resize(previous.len() + gap, 0);
        }
        for (coefficient, &value) in previous.iter().enumerate() {
            locator[coefficient + gap] ^= gf256::mul(scale, value);
        }

        if error_count * 2 <= index {
            error_count = index + 1 - error_count;
            previous = old_locator;
            previous_discrepancy = discrepancy;
            gap = 1;
        } else {
            gap += 1;
        }
    }

    locator.truncate(error_count + 1);
    locator
}

fn find_error_locations(locator: &[u8], codeword_len: usize) -> Option<Vec<usize>> {
    let mut result = Vec::with_capacity(locator.len().saturating_sub(1));
    for position in 0..codeword_len {
        let degree = codeword_len - 1 - position;
        let root = gf256::inverse(gf256::exp(degree));
        if evaluate(locator, root) == 0 {
            result.push(position);
        }
    }
    (result.len() == locator.len().saturating_sub(1)).then_some(result)
}

fn error_magnitudes(syndromes: &[u8], locations: &[usize], codeword_len: usize) -> Option<Vec<u8>> {
    let error_count = locations.len();
    let mut matrix = vec![0_u8; error_count * (error_count + 1)];

    for row in 0..error_count {
        for (column, &position) in locations.iter().enumerate() {
            let degree = codeword_len - 1 - position;
            matrix[row * (error_count + 1) + column] = gf256_power(gf256::exp(degree), row);
        }
        matrix[row * (error_count + 1) + error_count] = syndromes[row];
    }

    for pivot in 0..error_count {
        let pivot_row =
            (pivot..error_count).find(|&row| matrix[row * (error_count + 1) + pivot] != 0)?;
        if pivot_row != pivot {
            for column in pivot..=error_count {
                matrix.swap(
                    pivot * (error_count + 1) + column,
                    pivot_row * (error_count + 1) + column,
                );
            }
        }

        let inverse = gf256::inverse(matrix[pivot * (error_count + 1) + pivot]);
        for column in pivot..=error_count {
            let index = pivot * (error_count + 1) + column;
            matrix[index] = gf256::mul(matrix[index], inverse);
        }

        for row in 0..error_count {
            if row == pivot {
                continue;
            }
            let factor = matrix[row * (error_count + 1) + pivot];
            if factor == 0 {
                continue;
            }
            for column in pivot..=error_count {
                let target = row * (error_count + 1) + column;
                let source = pivot * (error_count + 1) + column;
                matrix[target] ^= gf256::mul(factor, matrix[source]);
            }
        }
    }

    let mut result = Vec::with_capacity(error_count);
    for row in 0..error_count {
        result.push(matrix[row * (error_count + 1) + error_count]);
    }
    Some(result)
}

fn evaluate(coefficients: &[u8], value: u8) -> u8 {
    let mut result = 0_u8;
    let mut power = 1_u8;
    for &coefficient in coefficients {
        result ^= gf256::mul(coefficient, power);
        power = gf256::mul(power, value);
    }
    result
}

fn gf256_power(value: u8, exponent: usize) -> u8 {
    let mut result = 1_u8;
    for _ in 0..exponent {
        result = gf256::mul(result, value);
    }
    result
}
