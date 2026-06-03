use std::{
    fs,
    path::{Path, PathBuf},
};

use gpui::{Image, ImageFormat};

#[derive(Debug)]
pub enum LoadImageError {
    UnsupportedFormat {
        path: PathBuf,
    },
    ReadFailed {
        path: PathBuf,
        error: std::io::Error,
    },
}

impl std::fmt::Display for LoadImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadImageError::UnsupportedFormat { path } => {
                write!(f, "unsupported image format for `{}`", path.display())
            }
            LoadImageError::ReadFailed { path, error } => {
                write!(f, "failed to read `{}`: {error}", path.display())
            }
        }
    }
}

impl std::error::Error for LoadImageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LoadImageError::UnsupportedFormat { .. } => None,
            LoadImageError::ReadFailed { error, .. } => Some(error),
        }
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

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

pub fn load_image_from_path(path: impl AsRef<Path>) -> Result<Image, LoadImageError> {
    let image_path = path.as_ref();

    let Some(image_format) = image_format_for_path(image_path) else {
        return Err(LoadImageError::UnsupportedFormat {
            path: image_path.to_path_buf(),
        });
    };

    match fs::read(image_path) {
        Ok(bytes) => Ok(Image::from_bytes(image_format, bytes)),
        Err(error) => {
            return Err(LoadImageError::ReadFailed {
                path: image_path.to_path_buf(),
                error,
            });
        }
    }
}
