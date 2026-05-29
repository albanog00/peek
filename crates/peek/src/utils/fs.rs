use std::{fs, path::Path};

use gpui::{Image, ImageFormat};

pub fn image_format_for_path(path: impl AsRef<std::path::Path>) -> Option<ImageFormat> {
    match path.as_ref().extension()?.to_str()?.to_lowercase().as_str() {
        "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
        "png" => Some(ImageFormat::Png),
        "webp" => Some(ImageFormat::Webp),
        "gif" => Some(ImageFormat::Gif),
        "bmp" => Some(ImageFormat::Bmp),
        "tif" | "tiff" => Some(ImageFormat::Tiff),
        "ico" => Some(ImageFormat::Ico),
        _ => None,
    }
}

pub fn load_image_from_path(path: impl AsRef<Path>) -> Option<Image> {
    let image_path = path.as_ref();

    let Some(image_format) = image_format_for_path(image_path) else {
        tracing::error!("unsupported image format for `{}`", image_path.display());
        return None;
    };

    match fs::read(image_path) {
        Ok(bytes) => Some(Image::from_bytes(image_format, bytes)),
        Err(error) => {
            tracing::error!("failed to read `{}`: {error}", image_path.display());
            None
        }
    }
}
