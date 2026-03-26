use std::path::Path;

use image::ImageFormat;

use crate::{RasterError, RasterFormat};

pub(crate) fn raster_format_to_image_format(format: RasterFormat) -> ImageFormat {
    match format {
        RasterFormat::Png => ImageFormat::Png,
        RasterFormat::Jpeg => ImageFormat::Jpeg,
        RasterFormat::WebP => ImageFormat::WebP,
    }
}

pub(crate) fn infer_format(path: &Path) -> Result<RasterFormat, RasterError> {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return Err(RasterError::MissingExtension(path.to_path_buf()));
    };
    match extension.to_ascii_lowercase().as_str() {
        "png" => Ok(RasterFormat::Png),
        "jpg" | "jpeg" => Ok(RasterFormat::Jpeg),
        "webp" | "wep" => Ok(RasterFormat::WebP),
        _ => Err(RasterError::MissingExtension(path.to_path_buf())),
    }
}
