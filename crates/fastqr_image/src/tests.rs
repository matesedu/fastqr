use fastqr_core::{BitGrid, EncodeOptions, QrError, Version, decode_modules, encode_text};

use crate::{
    DecodeOptions, RasterError, RasterFormat, RenderOptions,
    binary::binarize,
    decode::{decode_bytes_with_format, decode_luma, decode_rgba},
    detect::{Point, estimate_bottom_right, locate, sample_pure_qr},
    render::{render_to_rgba, write_to_bytes},
};

#[test]
fn roundtrip_png() {
    let code = encode_text("fastqr image roundtrip", EncodeOptions::default()).expect("encodes");
    let bytes =
        write_to_bytes(&code, RasterFormat::Png, RenderOptions::default()).expect("renders");
    let decoded = decode_bytes_with_format(&bytes, RasterFormat::Png, DecodeOptions::default())
        .expect("decodes");
    assert_eq!(decoded.text.as_deref(), Some("fastqr image roundtrip"));
}

#[test]
fn rgba_roundtrip() {
    let code = encode_text("HELLO CAMERA", EncodeOptions::default()).expect("encodes");
    let render = RenderOptions {
        scale: 12,
        border: 4,
        ..RenderOptions::default()
    };
    let rgba = render_to_rgba(&code, render).expect("renders");
    let size = (code.size() as u32 + render.border * 2) * render.scale;
    let decoded = decode_rgba(
        size as usize,
        size as usize,
        &rgba,
        DecodeOptions::default(),
    )
    .expect("decodes");
    assert_eq!(decoded.text.as_deref(), Some("HELLO CAMERA"));
}

#[test]
fn rgba_roundtrip_larger_version() {
    let code = encode_text("fastqr image roundtrip", EncodeOptions::default()).expect("encodes");
    let render = RenderOptions {
        scale: 8,
        border: 4,
        ..RenderOptions::default()
    };
    let rgba = render_to_rgba(&code, render).expect("renders");
    let size = (code.size() as u32 + render.border * 2) * render.scale;
    let decoded = decode_rgba(
        size as usize,
        size as usize,
        &rgba,
        DecodeOptions::default(),
    )
    .expect("decodes");
    assert_eq!(decoded.text.as_deref(), Some("fastqr image roundtrip"));
}

#[test]
fn pure_sampler_matches_rendered_grid() {
    let code = encode_text("fastqr image roundtrip", EncodeOptions::default()).expect("encodes");
    let render = RenderOptions {
        scale: 8,
        border: 4,
        ..RenderOptions::default()
    };
    let rgba = render_to_rgba(&code, render).expect("renders");
    let size = (code.size() as u32 + render.border * 2) * render.scale;
    let mut luma = vec![0_u8; size as usize * size as usize];
    for (pixel, rgba) in luma.iter_mut().zip(rgba.chunks_exact(4)) {
        *pixel = rgba[0];
    }
    let binary = binarize(size as usize, size as usize, &luma);
    let sampled = sample_pure_qr(&binary).expect("samples");
    assert_eq!(sampled.size(), code.size());
    for y in 0..code.size() {
        for x in 0..code.size() {
            assert_eq!(sampled.get(x, y), code.module(x, y), "mismatch at {x},{y}");
        }
    }
}

#[test]
fn rejects_decode_dimensions_that_overflow() {
    let error = decode_rgba(usize::MAX, 2, &[], DecodeOptions::default()).expect_err("rejects");
    assert!(matches!(error, crate::RasterError::InvalidDimensions));
}

#[test]
fn rejects_decode_buffers_above_configured_pixel_limit() {
    let luma = vec![255_u8; 9];
    let error = decode_luma(
        3,
        3,
        &luma,
        DecodeOptions {
            try_invert: false,
            max_pixels: Some(8),
        },
    )
    .expect_err("rejects");
    assert!(matches!(error, RasterError::InvalidDimensions));

    let error = decode_luma(
        3,
        3,
        &luma,
        DecodeOptions {
            try_invert: false,
            max_pixels: None,
        },
    )
    .expect_err("continues to detector");
    assert!(matches!(error, RasterError::Detector(_)));
}

#[test]
fn preserves_checksum_error_after_sampled_candidate() {
    let grid = checksum_error_grid_with_damaged_finders();
    let (rgba, size) = render_grid_to_rgba(&grid, 8, 4);
    let error = decode_rgba(
        size,
        size,
        &rgba,
        DecodeOptions {
            try_invert: false,
            ..DecodeOptions::default()
        },
    )
    .expect_err("propagates checksum");
    assert!(matches!(error, RasterError::Qr(QrError::Checksum)));
}

#[test]
fn preserves_unsupported_mode_error_after_sampled_candidate() {
    let mut grid = unsupported_mode_grid();
    damage_finders(&mut grid);
    assert_eq!(decode_modules(&grid), Err(QrError::UnsupportedMode(8)));

    let (rgba, size) = render_grid_to_rgba(&grid, 8, 4);
    let error = decode_rgba(
        size,
        size,
        &rgba,
        DecodeOptions {
            try_invert: false,
            ..DecodeOptions::default()
        },
    )
    .expect_err("propagates unsupported mode");
    assert!(matches!(
        error,
        RasterError::Qr(QrError::UnsupportedMode(8))
    ));
}

#[test]
fn perspective_fixture_refines_bottom_right_from_alignment_pattern() {
    let code = encode_text(
        "fastqr perspective alignment fixture",
        EncodeOptions {
            min_version: Version::new(2).expect("valid version"),
            ..EncodeOptions::default()
        },
    )
    .expect("encodes");
    let width = 360;
    let height = 320;
    let scale = 10;
    let border = 4;
    let corners = [(44.0, 34.0), (304.0, 54.0), (32.0, 286.0), (282.0, 272.0)];
    let (luma, source_to_dest) =
        render_perspective_luma(&code, scale, border, width, height, corners);
    let binary = binarize(width, height, &luma);
    let (top_left, top_right, bottom_left, dimension) = locate(&binary).expect("locates");
    assert_eq!(dimension, code.size());

    let affine = Point {
        x: top_right.x + bottom_left.x - top_left.x,
        y: top_right.y + bottom_left.y - top_left.y,
    };
    let refined = estimate_bottom_right(&binary, top_left, top_right, bottom_left, dimension);
    let actual = projected_module_point(&source_to_dest, scale, border, dimension as f64 - 3.5);
    let actual_alignment =
        projected_module_point(&source_to_dest, scale, border, dimension as f64 - 6.5);
    let finder_span = dimension as f32 - 7.0;
    let x_axis = Point {
        x: (top_right.x - top_left.x) / finder_span,
        y: (top_right.y - top_left.y) / finder_span,
    };
    let y_axis = Point {
        x: (bottom_left.x - top_left.x) / finder_span,
        y: (bottom_left.y - top_left.y) / finder_span,
    };
    let expected_alignment = Point {
        x: top_left.x + (x_axis.x + y_axis.x) * (dimension as f32 - 10.0),
        y: top_left.y + (x_axis.y + y_axis.y) * (dimension as f32 - 10.0),
    };
    let detected_alignment = Point {
        x: refined.x - (affine.x - expected_alignment.x),
        y: refined.y - (affine.y - expected_alignment.y),
    };
    let affine_error = point_distance(affine, actual);
    let refined_error = point_distance(refined, actual);
    assert!(
        refined_error < affine_error * 0.75,
        "refined corner should improve affine estimate: refined={refined_error}, affine={affine_error}, refined=({},{}) affine=({},{}) actual=({},{}) alignment=({},{}) detected_alignment=({},{}) expected_alignment=({},{})",
        refined.x,
        refined.y,
        affine.x,
        affine.y,
        actual.x,
        actual.y,
        actual_alignment.x,
        actual_alignment.y,
        detected_alignment.x,
        detected_alignment.y,
        expected_alignment.x,
        expected_alignment.y
    );

    let decoded = decode_luma(
        width,
        height,
        &luma,
        DecodeOptions {
            try_invert: false,
            ..DecodeOptions::default()
        },
    )
    .expect("decodes perspective fixture");
    assert_eq!(
        decoded.text.as_deref(),
        Some("fastqr perspective alignment fixture")
    );
}

#[test]
fn rejects_zero_render_scale() {
    let code = encode_text("fastqr image dimensions", EncodeOptions::default()).expect("encodes");
    let error = render_to_rgba(
        &code,
        RenderOptions {
            scale: 0,
            ..RenderOptions::default()
        },
    )
    .expect_err("rejects");
    assert!(matches!(error, crate::RasterError::InvalidDimensions));
}

fn checksum_error_grid_with_damaged_finders() -> BitGrid {
    let code =
        encode_text("fastqr checksum propagation", EncodeOptions::default()).expect("encodes");
    let mut candidate = code.modules().clone();
    let size = candidate.size();
    for y in (0..size).rev() {
        for x in (0..size).rev() {
            if is_likely_function_module(size, x, y) {
                continue;
            }
            candidate.invert(x, y);
            if decode_modules(&candidate) == Err(QrError::Checksum) {
                damage_finders(&mut candidate);
                assert_eq!(decode_modules(&candidate), Err(QrError::Checksum));
                return candidate;
            }
        }
    }
    panic!("could not create checksum fixture");
}

fn is_likely_function_module(size: usize, x: usize, y: usize) -> bool {
    let top_left = x <= 8 && y <= 8;
    let top_right = x >= size - 8 && y <= 8;
    let bottom_left = x <= 8 && y >= size - 8;
    top_left || top_right || bottom_left || x == 6 || y == 6 || x == 8 || y == 8
}

fn damage_finders(grid: &mut BitGrid) {
    let size = grid.size();
    for (center_x, center_y) in [(3_usize, 3_usize), (size - 4, 3), (3, size - 4)] {
        for dy in -1_i32..=1 {
            for dx in -1_i32..=1 {
                grid.set(
                    (center_x as i32 + dx) as usize,
                    (center_y as i32 + dy) as usize,
                    false,
                );
            }
        }
    }
}

fn render_grid_to_rgba(grid: &BitGrid, scale: usize, border: usize) -> (Vec<u8>, usize) {
    let size = (grid.size() + border * 2) * scale;
    let mut rgba = vec![255_u8; size * size * 4];
    for module_y in 0..grid.size() {
        for module_x in 0..grid.size() {
            if !grid.get(module_x, module_y) {
                continue;
            }
            let start_x = (module_x + border) * scale;
            let start_y = (module_y + border) * scale;
            for y in start_y..start_y + scale {
                let row = y * size * 4;
                for x in start_x..start_x + scale {
                    let offset = row + x * 4;
                    rgba[offset..offset + 3].fill(0);
                }
            }
        }
    }
    (rgba, size)
}

fn unsupported_mode_grid() -> BitGrid {
    const SIZE: usize = 21;
    const DATA_CODEWORDS: usize = 19;
    const ECC_CODEWORDS: usize = 7;

    let mut data = Vec::with_capacity(DATA_CODEWORDS);
    data.push(0x80);
    let mut pad = 0xEC_u8;
    while data.len() < DATA_CODEWORDS {
        data.push(pad);
        pad ^= 0xFD;
    }

    let generator = rs_generator(ECC_CODEWORDS);
    let mut ecc = vec![0_u8; ECC_CODEWORDS];
    rs_remainder_into(&data, &generator, &mut ecc);
    let mut codewords = data;
    codewords.extend_from_slice(&ecc);

    let mut modules = BitGrid::new(SIZE);
    let mut functions = BitGrid::new(SIZE);
    draw_version1_function_patterns(&mut modules, &mut functions);
    draw_codewords(&mut modules, &functions, &codewords);
    apply_mask0(&mut modules, &functions);
    draw_format_bits_low_mask0(&mut modules, &mut functions);

    assert_eq!(decode_modules(&modules), Err(QrError::UnsupportedMode(8)));
    modules
}

fn draw_version1_function_patterns(modules: &mut BitGrid, functions: &mut BitGrid) {
    let size = modules.size();
    draw_finder(modules, functions, 3, 3);
    draw_finder(modules, functions, size - 4, 3);
    draw_finder(modules, functions, 3, size - 4);

    for index in 8..size - 8 {
        let dark = (index & 1) == 0;
        set_function(modules, functions, 6, index, dark);
        set_function(modules, functions, index, 6, dark);
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
    set_function(modules, functions, 8, size - 8, true);
}

fn draw_finder(modules: &mut BitGrid, functions: &mut BitGrid, center_x: usize, center_y: usize) {
    let size = modules.size();
    for dy in -4_i32..=4 {
        for dx in -4_i32..=4 {
            let x = center_x as i32 + dx;
            let y = center_y as i32 + dy;
            if x < 0 || y < 0 || x >= size as i32 || y >= size as i32 {
                continue;
            }
            let dist = dx.unsigned_abs().max(dy.unsigned_abs());
            set_function(
                modules,
                functions,
                x as usize,
                y as usize,
                dist != 2 && dist != 4,
            );
        }
    }
}

fn set_function(modules: &mut BitGrid, functions: &mut BitGrid, x: usize, y: usize, dark: bool) {
    modules.set(x, y, dark);
    functions.set(x, y, true);
}

fn draw_codewords(modules: &mut BitGrid, functions: &BitGrid, codewords: &[u8]) {
    let size = modules.size();
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
                if bit_index < codewords.len() * 8 {
                    let bit = ((codewords[bit_index >> 3] >> (7 - (bit_index & 7))) & 1) != 0;
                    modules.set(x, y, bit);
                    bit_index += 1;
                }
            }
        }
        right -= 2;
    }
}

fn apply_mask0(modules: &mut BitGrid, functions: &BitGrid) {
    for y in 0..modules.size() {
        for x in 0..modules.size() {
            if !functions.get(x, y) && ((x + y) & 1) == 0 {
                modules.invert(x, y);
            }
        }
    }
}

fn draw_format_bits_low_mask0(modules: &mut BitGrid, functions: &mut BitGrid) {
    let size = modules.size();
    let bits = format_information_bits_low_mask0();
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
        set_function(modules, functions, x, y, ((bits >> index) & 1) != 0);
    }

    for index in 0..8 {
        set_function(
            modules,
            functions,
            size - 1 - index,
            8,
            ((bits >> index) & 1) != 0,
        );
    }
    for index in 8..15 {
        set_function(
            modules,
            functions,
            8,
            size - 15 + index,
            ((bits >> index) & 1) != 0,
        );
    }
}

fn format_information_bits_low_mask0() -> u16 {
    let data = 1_u16 << 3;
    let mut rem = data;
    for _ in 0..10 {
        rem = (rem << 1) ^ (((rem >> 9) & 1) * 0x537);
    }
    ((data << 10) | rem) ^ 0x5412
}

fn rs_generator(degree: usize) -> Vec<u8> {
    let mut result = vec![0_u8; degree];
    result[degree - 1] = 1;
    let mut root = 1_u8;
    for _ in 0..degree {
        for index in 0..degree {
            result[index] = gf_mul(result[index], root);
            if index + 1 < degree {
                result[index] ^= result[index + 1];
            }
        }
        root = gf_mul(root, 0x02);
    }
    result
}

fn rs_remainder_into(data: &[u8], generator: &[u8], result: &mut [u8]) {
    result.fill(0);
    for &value in data {
        let factor = value ^ result[0];
        result.rotate_left(1);
        if let Some(last) = result.last_mut() {
            *last = 0;
        }
        for (index, coefficient) in generator.iter().enumerate() {
            result[index] ^= gf_mul(*coefficient, factor);
        }
    }
}

fn gf_mul(left: u8, right: u8) -> u8 {
    let mut left = left;
    let mut right = right;
    let mut result = 0_u8;
    while right != 0 {
        if (right & 1) != 0 {
            result ^= left;
        }
        let carry = (left & 0x80) != 0;
        left <<= 1;
        if carry {
            left ^= 0x1D;
        }
        right >>= 1;
    }
    result
}

fn render_perspective_luma(
    code: &fastqr_core::QrCode,
    scale: usize,
    border: usize,
    width: usize,
    height: usize,
    corners: [(f64, f64); 4],
) -> (Vec<u8>, [f64; 8]) {
    let source_side = ((code.size() + border * 2) * scale) as f64;
    let dest_to_source = solve_homography([
        (corners[0].0, corners[0].1, 0.0, 0.0),
        (corners[1].0, corners[1].1, source_side, 0.0),
        (corners[2].0, corners[2].1, 0.0, source_side),
        (corners[3].0, corners[3].1, source_side, source_side),
    ]);
    let source_to_dest = solve_homography([
        (0.0, 0.0, corners[0].0, corners[0].1),
        (source_side, 0.0, corners[1].0, corners[1].1),
        (0.0, source_side, corners[2].0, corners[2].1),
        (source_side, source_side, corners[3].0, corners[3].1),
    ]);

    let mut luma = vec![255_u8; width * height];
    for y in 0..height {
        for x in 0..width {
            let (source_x, source_y) = project(&dest_to_source, x as f64 + 0.5, y as f64 + 0.5);
            if source_x < 0.0
                || source_y < 0.0
                || source_x >= source_side
                || source_y >= source_side
            {
                continue;
            }
            let module_x = source_x / scale as f64 - border as f64;
            let module_y = source_y / scale as f64 - border as f64;
            if module_x < 0.0
                || module_y < 0.0
                || module_x >= code.size() as f64
                || module_y >= code.size() as f64
            {
                continue;
            }
            if code.module(module_x.floor() as usize, module_y.floor() as usize) {
                luma[y * width + x] = 0;
            }
        }
    }

    (luma, source_to_dest)
}

fn projected_module_point(
    source_to_dest: &[f64; 8],
    scale: usize,
    border: usize,
    module_coord: f64,
) -> Point {
    let source = (border as f64 + module_coord) * scale as f64;
    let (x, y) = project(source_to_dest, source, source);
    Point {
        x: x as f32,
        y: y as f32,
    }
}

fn point_distance(a: Point, b: Point) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    (dx * dx + dy * dy).sqrt()
}

#[allow(clippy::needless_range_loop)]
fn solve_homography(correspondences: [(f64, f64, f64, f64); 4]) -> [f64; 8] {
    let mut matrix = [[0_f64; 9]; 8];
    for (index, &(x, y, u, v)) in correspondences.iter().enumerate() {
        let row = index * 2;
        matrix[row] = [x, y, 1.0, 0.0, 0.0, 0.0, -u * x, -u * y, u];
        matrix[row + 1] = [0.0, 0.0, 0.0, x, y, 1.0, -v * x, -v * y, v];
    }

    for pivot in 0..8 {
        let mut best_row = pivot;
        let mut best_value = matrix[pivot][pivot].abs();
        for (row, values) in matrix.iter().enumerate().skip(pivot + 1) {
            let value = values[pivot].abs();
            if value > best_value {
                best_value = value;
                best_row = row;
            }
        }
        assert!(best_value >= 1e-9, "homography is solvable");
        if best_row != pivot {
            matrix.swap(best_row, pivot);
        }

        let scale = matrix[pivot][pivot];
        for column in pivot..9 {
            matrix[pivot][column] /= scale;
        }
        for row in 0..8 {
            if row == pivot {
                continue;
            }
            let factor = matrix[row][pivot];
            if factor == 0.0 {
                continue;
            }
            for column in pivot..9 {
                matrix[row][column] -= factor * matrix[pivot][column];
            }
        }
    }

    let mut result = [0_f64; 8];
    for index in 0..8 {
        result[index] = matrix[index][8];
    }
    result
}

fn project(transform: &[f64; 8], x: f64, y: f64) -> (f64, f64) {
    let denominator = transform[6] * x + transform[7] * y + 1.0;
    (
        (transform[0] * x + transform[1] * y + transform[2]) / denominator,
        (transform[3] * x + transform[4] * y + transform[5]) / denominator,
    )
}

#[test]
fn rejects_render_dimensions_above_safe_limit() {
    let code = encode_text("fastqr image dimensions", EncodeOptions::default()).expect("encodes");
    let error = render_to_rgba(
        &code,
        RenderOptions {
            border: 20_000,
            ..RenderOptions::default()
        },
    )
    .expect_err("rejects");
    assert!(matches!(error, crate::RasterError::InvalidDimensions));
}
