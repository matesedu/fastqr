use std::{fs, path::Path};

use fastqr_core::{DecodedQr, decode_modules};
use image::DynamicImage;

use crate::{
    DecodeOptions, RasterError, RasterFormat,
    binary::{BinaryImage, binarize},
    detect::{Point, locate, sample_pure_qr, sample_qr_grid},
    format::{infer_format, raster_format_to_image_format},
};

pub fn decode_file<P: AsRef<Path>>(path: P) -> Result<DecodedQr, RasterError> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(image::ImageError::IoError)?;
    let format = infer_format(path)?;
    decode_bytes_with_format(&bytes, format, DecodeOptions::default())
}

pub fn decode_bytes(bytes: &[u8], options: DecodeOptions) -> Result<DecodedQr, RasterError> {
    let image = image::load_from_memory(bytes)?;
    decode_dynamic_image(&image, options)
}

pub fn decode_bytes_with_format(
    bytes: &[u8],
    format: RasterFormat,
    options: DecodeOptions,
) -> Result<DecodedQr, RasterError> {
    let image = image::load_from_memory_with_format(bytes, raster_format_to_image_format(format))?;
    decode_dynamic_image(&image, options)
}

pub fn decode_dynamic_image(
    image: &DynamicImage,
    options: DecodeOptions,
) -> Result<DecodedQr, RasterError> {
    match image {
        DynamicImage::ImageLuma8(image) => decode_luma(
            image.width() as usize,
            image.height() as usize,
            image.as_raw(),
            options,
        ),
        DynamicImage::ImageLumaA8(image) => decode_luma_alpha(
            image.width() as usize,
            image.height() as usize,
            image.as_raw(),
            options,
        ),
        DynamicImage::ImageRgb8(image) => decode_rgb(
            image.width() as usize,
            image.height() as usize,
            image.as_raw(),
            options,
        ),
        DynamicImage::ImageRgba8(image) => decode_rgba(
            image.width() as usize,
            image.height() as usize,
            image.as_raw(),
            options,
        ),
        _ => {
            let rgba = image.to_rgba8();
            decode_rgba(
                rgba.width() as usize,
                rgba.height() as usize,
                rgba.as_raw(),
                options,
            )
        }
    }
}

pub fn decode_rgba(
    width: usize,
    height: usize,
    rgba: &[u8],
    options: DecodeOptions,
) -> Result<DecodedQr, RasterError> {
    let pixel_len = checked_raster_len(width, height, 1)?;
    let expected_len = checked_raster_len(width, height, 4)?;
    if rgba.len() != expected_len {
        return Err(RasterError::InvalidBuffer);
    }
    let mut luma = vec![0_u8; pixel_len];
    for (pixel, chunk) in luma.iter_mut().zip(rgba.chunks_exact(4)) {
        let alpha = u16::from(chunk[3]);
        let red = (u16::from(chunk[0]) * alpha + 255 * (255 - alpha)) / 255;
        let green = (u16::from(chunk[1]) * alpha + 255 * (255 - alpha)) / 255;
        let blue = (u16::from(chunk[2]) * alpha + 255 * (255 - alpha)) / 255;
        *pixel = ((red * 77 + green * 150 + blue * 29) >> 8) as u8;
    }
    decode_luma(width, height, &luma, options)
}

fn decode_rgb(
    width: usize,
    height: usize,
    rgb: &[u8],
    options: DecodeOptions,
) -> Result<DecodedQr, RasterError> {
    let pixel_len = checked_raster_len(width, height, 1)?;
    let expected_len = checked_raster_len(width, height, 3)?;
    if rgb.len() != expected_len {
        return Err(RasterError::InvalidBuffer);
    }
    let mut luma = vec![0_u8; pixel_len];
    for (pixel, chunk) in luma.iter_mut().zip(rgb.chunks_exact(3)) {
        *pixel = ((u16::from(chunk[0]) * 77 + u16::from(chunk[1]) * 150 + u16::from(chunk[2]) * 29)
            >> 8) as u8;
    }
    decode_luma(width, height, &luma, options)
}

fn decode_luma_alpha(
    width: usize,
    height: usize,
    luma_alpha: &[u8],
    options: DecodeOptions,
) -> Result<DecodedQr, RasterError> {
    let pixel_len = checked_raster_len(width, height, 1)?;
    let expected_len = checked_raster_len(width, height, 2)?;
    if luma_alpha.len() != expected_len {
        return Err(RasterError::InvalidBuffer);
    }
    let mut luma = vec![0_u8; pixel_len];
    for (pixel, chunk) in luma.iter_mut().zip(luma_alpha.chunks_exact(2)) {
        let alpha = u16::from(chunk[1]);
        *pixel = ((u16::from(chunk[0]) * alpha + 255 * (255 - alpha)) / 255) as u8;
    }
    decode_luma(width, height, &luma, options)
}

pub fn decode_luma(
    width: usize,
    height: usize,
    luma: &[u8],
    options: DecodeOptions,
) -> Result<DecodedQr, RasterError> {
    let expected_len = checked_raster_len(width, height, 1)?;
    if luma.len() != expected_len {
        return Err(RasterError::InvalidBuffer);
    }
    let binary = binarize(width, height, luma);
    if let Ok(decoded) = decode_binary(&binary) {
        return Ok(decoded);
    }
    if options.try_invert {
        let inverted = binary.into_inverted();
        if let Ok(decoded) = decode_binary(&inverted) {
            return Ok(decoded);
        }
    }
    Err(RasterError::Detector(
        "unable to locate a QR code in the image",
    ))
}

fn checked_raster_len(width: usize, height: usize, channels: usize) -> Result<usize, RasterError> {
    if width == 0 || height == 0 {
        return Err(RasterError::InvalidDimensions);
    }
    width
        .checked_mul(height)
        .and_then(|pixels| pixels.checked_mul(channels))
        .ok_or(RasterError::InvalidDimensions)
}

fn decode_binary(binary: &BinaryImage) -> Result<DecodedQr, RasterError> {
    if let Ok(sampled) = sample_pure_qr(binary)
        && let Ok(decoded) = decode_modules(&sampled)
    {
        return Ok(decoded);
    }
    let (top_left, top_right, bottom_left, dimension) = locate(binary)?;
    let bottom_right = Point {
        x: top_right.x + bottom_left.x - top_left.x,
        y: top_right.y + bottom_left.y - top_left.y,
    };
    let sampled = sample_qr_grid(
        binary,
        top_left,
        top_right,
        bottom_left,
        bottom_right,
        dimension,
    )?;
    Ok(decode_modules(&sampled)?)
}
