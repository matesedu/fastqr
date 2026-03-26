const fn build_tables() -> ([u8; 512], [u8; 256]) {
    let mut exp = [0_u8; 512];
    let mut log = [0_u8; 256];
    let mut x = 1_u16;
    let mut i = 0_usize;
    while i < 255 {
        exp[i] = x as u8;
        log[x as usize] = i as u8;
        x <<= 1;
        if x & 0x100 != 0 {
            x ^= 0x11D;
        }
        i += 1;
    }
    while i < 512 {
        exp[i] = exp[i - 255];
        i += 1;
    }
    (exp, log)
}

const TABLES: ([u8; 512], [u8; 256]) = build_tables();

#[inline]
pub fn mul(left: u8, right: u8) -> u8 {
    if left == 0 || right == 0 {
        return 0;
    }
    TABLES.0[usize::from(TABLES.1[left as usize]) + usize::from(TABLES.1[right as usize])]
}
