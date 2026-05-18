use std::{hint::black_box, time::Instant};

use fastqr_core::{EncodeOptions, Version, encode_text};
use fastqr_image::{
    DecodeOptions, RasterFormat, RenderOptions, decode_bytes_with_format, decode_rgba,
    render_to_rgba, write_to_bytes,
};

fn main() {
    let png_code = encode_text(
        "fastqr image decode benchmark png",
        EncodeOptions {
            min_version: Version::new(2).expect("valid version"),
            ..EncodeOptions::default()
        },
    )
    .expect("encodes");
    let png_bytes = write_to_bytes(
        &png_code,
        RasterFormat::Png,
        RenderOptions {
            scale: 8,
            border: 4,
            ..RenderOptions::default()
        },
    )
    .expect("renders png");

    let rgba_code = encode_text(
        "fastqr image decode benchmark camera frame with a larger symbol",
        EncodeOptions {
            min_version: Version::new(5).expect("valid version"),
            ..EncodeOptions::default()
        },
    )
    .expect("encodes");
    let rgba_render = RenderOptions {
        scale: 6,
        border: 4,
        ..RenderOptions::default()
    };
    let rgba = render_to_rgba(&rgba_code, rgba_render).expect("renders rgba");
    let rgba_size =
        (rgba_code.size() + rgba_render.border as usize * 2) * rgba_render.scale as usize;

    bench("png_bytes_v2", 250, || {
        let decoded = decode_bytes_with_format(
            black_box(&png_bytes),
            RasterFormat::Png,
            DecodeOptions::default(),
        )
        .expect("decodes png");
        black_box(decoded);
    });
    bench("rgba_camera_v5", 250, || {
        let decoded = decode_rgba(
            black_box(rgba_size),
            black_box(rgba_size),
            black_box(&rgba),
            DecodeOptions::default(),
        )
        .expect("decodes rgba");
        black_box(decoded);
    });
}

fn bench(name: &str, iterations: u32, mut run: impl FnMut()) {
    for _ in 0..10 {
        run();
    }

    let start = Instant::now();
    for _ in 0..iterations {
        run();
    }
    let elapsed = start.elapsed();
    let per_iter =
        std::time::Duration::from_secs_f64(elapsed.as_secs_f64() / f64::from(iterations));
    println!("{name}: {per_iter:?}/iter over {iterations} iterations ({elapsed:?} total)");
}
