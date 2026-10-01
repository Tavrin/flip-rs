//! PNG and OpenEXR loading and saving, through the `image` crate.
//!
//! Requires the `image` feature.
use crate::{dimensions, reserved, ErrorMap, FlipError, RgbImage};
use std::path::Path;

// Check the detected format rather than trusting the file extension. The
// decoder keeps image crate's default limits; checked layouts and fallible
// reservations protect the conversions performed by this crate.
fn load(path: &Path, format: image::ImageFormat) -> Result<image::DynamicImage, FlipError> {
    let reader = image::ImageReader::open(path)
        .map_err(image::ImageError::IoError)?
        .with_guessed_format()
        .map_err(image::ImageError::IoError)?;
    if reader.format() != Some(format) {
        return Err(FlipError::InvalidParameter(
            "load_srgb requires PNG; load_linear requires OpenEXR",
        ));
    }
    Ok(reader.decode()?)
}

/// Loads PNG channels as sRGB floats for [`crate::ldr_flip`].
///
/// Assumes the source channels are sRGB; embedded profiles are not transformed.
/// Channels are quantized to 8 bits, as the reference tool does. Alpha is
/// discarded. Float formats are rejected rather than relabeled as sRGB.
pub fn load_srgb(path: impl AsRef<Path>) -> Result<RgbImage<f32>, FlipError> {
    let image = load(path.as_ref(), image::ImageFormat::Png)?;
    let (w, h) = (image.width() as usize, image.height() as usize);
    let mut pixels = reserved(dimensions(w, h, 3)?)?;
    pixels.extend(
        image
            .into_rgb8()
            .into_raw()
            .into_iter()
            .map(|v| f32::from(v) / 255.0),
    );
    RgbImage::new(w, h, pixels)
}

/// Loads OpenEXR as linear RGB floats for [`crate::hdr_flip`].
///
/// Only OpenEXR (the enabled float decoder) is accepted. Its channels are
/// assumed linear and read without color conversion. Alpha is discarded.
/// PNG and other integer formats return an error; use [`load_srgb`] for PNG.
pub fn load_linear(path: impl AsRef<Path>) -> Result<RgbImage<f32>, FlipError> {
    let image = load(path.as_ref(), image::ImageFormat::OpenExr)?;
    dimensions(image.width() as usize, image.height() as usize, 3)?;
    let image = image.into_rgb32f();
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
        let mut pixels = reserved(dimensions(image.width(), image.height(), 3)?)?;
        pixels.extend_from_slice(image.pixels());
        let data = image::Rgb32FImage::from_raw(w, h, pixels).ok_or(FlipError::SizeMismatch)?;
        image::DynamicImage::ImageRgb32F(data).save(path)?;
    } else {
        let mut data = reserved(dimensions(image.width(), image.height(), 3)?)?;
        data.extend(
            image
                .pixels()
                .iter()
                .map(|v| (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8),
        );
        image::save_buffer(path, &data, w, h, image::ColorType::Rgb8)?;
    }
    Ok(())
}

/// Saves an error map colored with [`crate::MAGMA`]; see [`save_rgb`].
pub fn save_heatmap(map: &ErrorMap, path: impl AsRef<Path>) -> Result<(), FlipError> {
    save_rgb(&map.colorize()?, path)
}

/// Saves raw errors as gray RGB; see [`save_rgb`].
pub fn save_error_map(map: &ErrorMap, path: impl AsRef<Path>) -> Result<(), FlipError> {
    let mut pixels = reserved(dimensions(map.width(), map.height(), 3)?)?;
    for v in map.pixels() {
        pixels.extend_from_slice(&[*v; 3]);
    }
    let rgb = RgbImage::new(map.width(), map.height(), pixels)?;
    save_rgb(&rgb, path)
}
