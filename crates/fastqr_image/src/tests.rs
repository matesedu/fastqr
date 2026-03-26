use fastqr_core::{EncodeOptions, encode_text};

use crate::{
    DecodeOptions, RasterFormat, RenderOptions,
    binary::binarize,
    decode::{decode_bytes_with_format, decode_rgba},
    detect::sample_pure_qr,
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
    let rgba = render_to_rgba(&code, render);
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
    let rgba = render_to_rgba(&code, render);
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
    let rgba = render_to_rgba(&code, render);
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
