use std::{fs, io::Cursor, path::Path};

use fastqr_core::{DecodedQr, decode_modules};
use image::{DynamicImage, ImageReader};

use crate::{
    DecodeOptions, RasterError, RasterFormat,
    binary::{BinaryImage, binarize},
    detect::{locate, sample_detected_qr_grid, sample_pure_qr},
    format::{infer_format, raster_format_to_image_format},
};

pub fn decode_file<P: AsRef<Path>>(path: P) -> Result<DecodedQr, RasterError> {
    decode_file_with_options(path, DecodeOptions::default())
}

pub fn decode_file_with_options<P: AsRef<Path>>(
    path: P,
    options: DecodeOptions,
) -> Result<DecodedQr, RasterError> {
    let path = path.as_ref();
    let bytes = fs::read(path).map_err(image::ImageError::IoError)?;
    let format = infer_format(path)?;
    decode_bytes_with_format(&bytes, format, options)
}

pub fn decode_bytes(bytes: &[u8], options: DecodeOptions) -> Result<DecodedQr, RasterError> {
    let image = load_image_from_memory(bytes, None, options)?;
    decode_dynamic_image(&image, options)
}

pub fn decode_bytes_with_format(
    bytes: &[u8],
    format: RasterFormat,
    options: DecodeOptions,
) -> Result<DecodedQr, RasterError> {
    let image =
        load_image_from_memory(bytes, Some(raster_format_to_image_format(format)), options)?;
    decode_dynamic_image(&image, options)
}

pub fn decode_dynamic_image(
    image: &DynamicImage,
    options: DecodeOptions,
) -> Result<DecodedQr, RasterError> {
    checked_decode_pixel_len(image.width() as usize, image.height() as usize, options)?;
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
    let pixel_len = checked_decode_pixel_len(width, height, options)?;
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
    let pixel_len = checked_decode_pixel_len(width, height, options)?;
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
    let pixel_len = checked_decode_pixel_len(width, height, options)?;
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
    let expected_len = checked_decode_pixel_len(width, height, options)?;
    if luma.len() != expected_len {
        return Err(RasterError::InvalidBuffer);
    }
    let binary = binarize(width, height, luma);
    let mut sampled_error = None;
    match decode_binary(&binary) {
        Ok(decoded) => return Ok(decoded),
        Err(RasterError::Qr(error)) => sampled_error = Some(error),
        Err(_) => {}
    }
    if options.try_invert {
        let inverted = binary.into_inverted();
        match decode_binary(&inverted) {
            Ok(decoded) => return Ok(decoded),
            Err(RasterError::Qr(error)) => {
                if sampled_error.is_none() {
                    sampled_error = Some(error);
                }
            }
            Err(_) => {}
        }
    }
    if let Some(error) = sampled_error {
        return Err(RasterError::Qr(error));
    }
    Err(RasterError::Detector(
        "unable to locate a QR code in the image",
    ))
}

fn load_image_from_memory(
    bytes: &[u8],
    format: Option<image::ImageFormat>,
    options: DecodeOptions,
) -> Result<DynamicImage, RasterError> {
    let cursor = Cursor::new(bytes);
    let mut reader = if let Some(format) = format {
        ImageReader::with_format(cursor, format)
    } else {
        ImageReader::new(cursor)
            .with_guessed_format()
            .map_err(image::ImageError::IoError)?
    };
    reader.limits(image_decode_limits(options)?);

    let image = reader.decode()?;
    checked_decode_pixel_len(image.width() as usize, image.height() as usize, options)?;
    Ok(image)
}

fn image_decode_limits(options: DecodeOptions) -> Result<image::Limits, RasterError> {
    let Some(max_pixels) = options.max_pixels else {
        return Ok(image::Limits::no_limits());
    };

    let max_alloc = max_pixels
        .checked_mul(4)
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(RasterError::InvalidDimensions)?;
    let max_side = max_pixels.min(u32::MAX as usize) as u32;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(max_side);
    limits.max_image_height = Some(max_side);
    limits.max_alloc = Some(max_alloc);
    Ok(limits)
}

fn checked_decode_pixel_len(
    width: usize,
    height: usize,
    options: DecodeOptions,
) -> Result<usize, RasterError> {
    let pixels = checked_raster_len(width, height, 1)?;
    if let Some(max_pixels) = options.max_pixels
        && pixels > max_pixels
    {
        return Err(RasterError::InvalidDimensions);
    }
    Ok(pixels)
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
    let mut sampled_error = None;
    if let Ok(sampled) = sample_pure_qr(binary) {
        match decode_modules(&sampled) {
            Ok(decoded) => return Ok(decoded),
            Err(error) => sampled_error = Some(error),
        }
    }
    let (top_left, top_right, bottom_left, dimension) = match locate(binary) {
        Ok(located) => located,
        Err(error) => {
            return if let Some(error) = sampled_error {
                Err(RasterError::Qr(error))
            } else {
                Err(error)
            };
        }
    };
    match sample_detected_qr_grid(binary, top_left, top_right, bottom_left, dimension) {
        Ok(sampled) => match decode_modules(&sampled) {
            Ok(decoded) => Ok(decoded),
            Err(error) => Err(RasterError::Qr(error)),
        },
        Err(error) => {
            if let Some(error) = sampled_error {
                Err(RasterError::Qr(error))
            } else {
                Err(error)
            }
        }
    }
}
