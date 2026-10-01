//! PNG and OpenEXR loading and saving, through the `image` crate.
//!
//! Requires the `image` feature.
use crate::{ErrorMap, FlipError, RgbImage};
use std::path::Path;

/// Loads an image as sRGB floats in 0..=1, as input for [`crate::ldr_flip`].
///
/// Channels are quantized to 8 bits, as the reference tool does for PNG.
/// Alpha is discarded.
pub fn load_srgb(path: impl AsRef<Path>) -> Result<RgbImage<f32>, FlipError> {
    let image = image::open(path)?.to_rgb8();
    RgbImage::new(
        image.width() as usize,
        image.height() as usize,
        image
            .into_raw()
            .into_iter()
            .map(|v| f32::from(v) / 255.0)
            .collect(),
    )
}

/// Loads an image as linear RGB floats, as input for [`crate::hdr_flip`].
///
/// Float formats such as OpenEXR are read without conversion. Alpha is
/// discarded.
pub fn load_linear(path: impl AsRef<Path>) -> Result<RgbImage<f32>, FlipError> {
    let image = image::open(path)?.to_rgb32f();
    RgbImage::new(
        image.width() as usize,
        image.height() as usize,
        image.into_raw(),
    )
}

/// Saves RGB floats without color conversion.
///
/// A `.exr` path stores the floats unchanged. A `.png` path stores 8-bit RGB,
/// with values clamped to 0..=1 and rounded. Other formats are not enabled.
pub fn save_rgb(image: &RgbImage<f32>, path: impl AsRef<Path>) -> Result<(), FlipError> {
    crate::finite(image.pixels())?;
    let w = u32::try_from(image.width()).map_err(|_| FlipError::InvalidDimensions)?;
    let h = u32::try_from(image.height()).map_err(|_| FlipError::InvalidDimensions)?;
    let path = path.as_ref();
    if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("exr"))
    {
        let data = image::Rgb32FImage::from_raw(w, h, image.pixels().to_vec())
            .ok_or(FlipError::SizeMismatch)?;
        image::DynamicImage::ImageRgb32F(data).save(path)?;
    } else {
        let data: Vec<u8> = image
            .pixels()
            .iter()
            .map(|v| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8)
            .collect();
        image::save_buffer(path, &data, w, h, image::ColorType::Rgb8)?;
    }
    Ok(())
}

/// Saves an error map colored with [`crate::MAGMA`]; see [`save_rgb`].
pub fn save_heatmap(map: &ErrorMap, path: impl AsRef<Path>) -> Result<(), FlipError> {
    save_rgb(&map.colorize(), path)
}

/// Saves raw errors as gray RGB; see [`save_rgb`].
pub fn save_error_map(map: &ErrorMap, path: impl AsRef<Path>) -> Result<(), FlipError> {
    let rgb = RgbImage::new(
        map.width(),
        map.height(),
        map.pixels().iter().flat_map(|v| [*v; 3]).collect(),
    )?;
    save_rgb(&rgb, path)
}
