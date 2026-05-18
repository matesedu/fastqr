use std::{fs, io::Cursor, path::Path};

use fastqr_core::{QrCode, encode_bytes, encode_text};
use image::{DynamicImage, ImageBuffer, Rgba};

use crate::{
    RasterError, RasterFormat, RenderOptions,
    format::{infer_format, raster_format_to_image_format},
};

const MAX_PIXELS_PER_SIDE: usize = 16_384;

pub fn encode_text_to_image(
    text: &str,
    render: RenderOptions,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, RasterError> {
    let code = encode_text(text, Default::default())?;
    render_to_image(&code, render)
}

pub fn encode_bytes_to_image(
    data: &[u8],
    render: RenderOptions,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, RasterError> {
    let code = encode_bytes(data, Default::default())?;
    render_to_image(&code, render)
}

pub fn render_to_image(
    code: &QrCode,
    render: RenderOptions,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, RasterError> {
    let dimensions = render_dimensions(code, render)?;
    let rgba = render_to_rgba(code, render)?;
    let size =
        u32::try_from(dimensions.pixels_per_side).map_err(|_| RasterError::InvalidDimensions)?;
    ImageBuffer::from_vec(size, size, rgba).ok_or(RasterError::InvalidBuffer)
}

pub fn render_to_rgba(code: &QrCode, render: RenderOptions) -> Result<Vec<u8>, RasterError> {
    let dimensions = render_dimensions(code, render)?;
    let scale = dimensions.scale;
    let border = dimensions.border;
    let stride = dimensions.stride;
    let mut rgba = vec![0_u8; dimensions.buffer_len];
    fill_rgba(&mut rgba, render.light);

    for y in 0..code.size() {
        let row_start = (y + border) * scale * stride;
        let mut x = 0;
        while x < code.size() {
            if !code.module(x, y) {
                x += 1;
                continue;
            }
            let run_start = x;
            while x < code.size() && code.module(x, y) {
                x += 1;
            }
            let module_start = row_start + (run_start + border) * scale * 4;
            let module_end = row_start + (x + border) * scale * 4;
            fill_rgba(&mut rgba[module_start..module_end], render.dark);
        }

        let row_end = row_start + stride;
        for copy_row in 1..scale {
            rgba.copy_within(row_start..row_end, row_start + copy_row * stride);
        }
    }
    Ok(rgba)
}

pub fn write_to_bytes(
    code: &QrCode,
    format: RasterFormat,
    render: RenderOptions,
) -> Result<Vec<u8>, RasterError> {
    let image = DynamicImage::ImageRgba8(render_to_image(code, render)?);
    let mut cursor = Cursor::new(Vec::new());
    image.write_to(&mut cursor, raster_format_to_image_format(format))?;
    Ok(cursor.into_inner())
}

pub fn write_to_path<P: AsRef<Path>>(
    code: &QrCode,
    path: P,
    render: RenderOptions,
) -> Result<(), RasterError> {
    let path = path.as_ref();
    let format = infer_format(path)?;
    let bytes = write_to_bytes(code, format, render)?;
    fs::write(path, bytes).map_err(image::ImageError::IoError)?;
    Ok(())
}

fn fill_rgba(bytes: &mut [u8], rgba: [u8; 4]) {
    if bytes.is_empty() {
        return;
    }

    bytes[..4].copy_from_slice(&rgba);
    let mut filled = 4;
    while filled < bytes.len() {
        let copy_len = filled.min(bytes.len() - filled);
        let (head, tail) = bytes.split_at_mut(filled);
        tail[..copy_len].copy_from_slice(&head[..copy_len]);
        filled += copy_len;
    }
}

struct RenderDimensions {
    scale: usize,
    border: usize,
    pixels_per_side: usize,
    stride: usize,
    buffer_len: usize,
}

fn render_dimensions(
    code: &QrCode,
    render: RenderOptions,
) -> Result<RenderDimensions, RasterError> {
    let scale = usize::try_from(render.scale).map_err(|_| RasterError::InvalidDimensions)?;
    if scale == 0 {
        return Err(RasterError::InvalidDimensions);
    }

    let border = usize::try_from(render.border).map_err(|_| RasterError::InvalidDimensions)?;
    let modules_per_side = code
        .size()
        .checked_add(
            border
                .checked_mul(2)
                .ok_or(RasterError::InvalidDimensions)?,
        )
        .ok_or(RasterError::InvalidDimensions)?;
    let pixels_per_side = modules_per_side
        .checked_mul(scale)
        .ok_or(RasterError::InvalidDimensions)?;
    if pixels_per_side > MAX_PIXELS_PER_SIDE {
        return Err(RasterError::InvalidDimensions);
    }

    let stride = pixels_per_side
        .checked_mul(4)
        .ok_or(RasterError::InvalidDimensions)?;
    let buffer_len = pixels_per_side
        .checked_mul(stride)
        .ok_or(RasterError::InvalidDimensions)?;

    Ok(RenderDimensions {
        scale,
        border,
        pixels_per_side,
        stride,
        buffer_len,
    })
}
