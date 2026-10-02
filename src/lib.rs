//! A Rust port of NVIDIA FLIP v1.7, a perceptual metric for the difference a
//! viewer sees when flipping between a reference and a test image.
//!
//! The output is a per-pixel error map, normally in 0..1, together with pooled
//! statistics and a Magma heatmap. Results match the C++ reference at revision
//! `b475eb4` to within 6e-8 per pixel on the measured corpus; see the
//! repository's `parity/` directory for corpus and randomized sweep results.
//!
//! ![Reference, test and FLIP heatmap of a procedural scene](https://raw.githubusercontent.com/Tavrin/flip-rs/main/docs/img/hero.webp)
//!
//! A [browser demo](https://tavrin.github.io/flip-rs/) runs [`ldr_flip`]
//! through the `wasm` feature on images you choose.
//!
//! # Algorithm
//!
//! [`ldr_flip`] converts both sRGB images to linear RGB and then to the YCxCz
//! opponent space. A color pipeline filters each channel with a
//! contrast-sensitivity function sized by the viewing distance in pixels per
//! degree, converts back to CIELab with the Hunt adjustment, and measures the
//! HyAB distance. A feature pipeline filters luminance with first and second
//! Gaussian derivatives to detect edge and point differences. The feature
//! error raises the color error to a power, which gives the final value.
//!
//! [`hdr_flip`] takes linear RGB, tone-maps it at a range of exposures chosen
//! from the reference image's luminance, and keeps the largest LDR-FLIP error
//! per pixel.
//!
//! # Data layout
//!
//! Buffers are row-major, top to bottom, with no padding. RGB images are
//! interleaved `R, G, B, R, G, B, ...`; maps hold one `f32` per pixel.
//! Invalid dimensions, nonfinite input and parameters that make the reference
//! algorithm undefined return a [`FlipError`].
//!
//! # Example
//!
//! ```
//! use flip_rs::{ldr_flip, RgbImage, DEFAULT_PPD};
//!
//! let reference = RgbImage::new(2, 1, vec![128_u8; 6])?;
//! let test = RgbImage::new(2, 1, vec![130_u8; 6])?;
//! let error = ldr_flip(&reference, &test, DEFAULT_PPD)?;
//! assert!(error.statistics().mean > 0.0);
//! # Ok::<(), flip_rs::FlipError>(())
//! ```
//!
//! # Features
//!
//! - `parallel` (default): process independent image rows with Rayon.
//!   Pooling is always sequential, so results do not depend on this feature.
//! - `image`: the `io` module, which loads and saves PNG and OpenEXR files.
//! - `wasm`: a wasm-bindgen export, `ldrFlip`, for RGBA bytes from `ImageData`.
//!
//! # References
//!
//! - Pontus Andersson, Jim Nilsson, Tomas Akenine-Möller, Magnus Oskarsson,
//!   Kalle Åström and Mark D. Fairchild,
//!   [FLIP: A Difference Evaluator for Alternating Images](https://research.nvidia.com/publication/2020-07_FLIP),
//!   Proceedings of the ACM on Computer Graphics and Interactive Techniques
//!   3(2), 2020 (HPG 2020). DOI 10.1145/3406183.
//! - Pontus Andersson, Jim Nilsson, Peter Shirley and Tomas Akenine-Möller,
//!   [Visualizing Errors in Rendered High Dynamic Range Images](https://research.nvidia.com/publication/2021-05_HDR-FLIP),
//!   Eurographics 2021 Short Papers. DOI 10.2312/egs.20211015.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod color;
mod filters;
mod hdr;
#[cfg(feature = "image")]
pub mod io;
mod magma;
mod pooling;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
mod wasm;
#[cfg(all(feature = "wasm", target_arch = "wasm32"))]
pub use wasm::ldr_flip_rgba;

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

pub use hdr::{
    hdr_flip, ExposureMap, HdrOptions, HdrParameters, HdrResult, Tonemapper, MAX_EXPOSURES,
};
pub use magma::MAGMA;
pub use pooling::{Histogram, Statistics};

/// The reference tool's default pixels per degree, about 67.02: a 0.7 m wide
/// monitor with 3840 horizontal pixels, viewed from 0.7 m.
pub const DEFAULT_PPD: f32 = 0.7 * (3840.0 / 0.7) * (std::f32::consts::PI / 180.0);

/// Computes pixels per degree from the viewing distance, the horizontal
/// resolution and the monitor width. Distance and width use the same unit.
///
/// Corresponds to `FLIP::calculatePPD`.
///
/// # Errors
///
/// [`FlipError::InvalidParameter`] unless every argument and the result are
/// finite and positive.
pub fn pixels_per_degree(
    distance: f32,
    resolution_x: f32,
    monitor_width: f32,
) -> Result<f32, FlipError> {
    if [distance, resolution_x, monitor_width]
        .iter()
        .any(|v| !v.is_finite() || *v <= 0.0)
    {
        return Err(FlipError::InvalidParameter(
            "viewing geometry must be finite and positive",
        ));
    }
    let ppd = distance * (resolution_x / monitor_width) * (std::f32::consts::PI / 180.0);
    if !ppd.is_finite() || ppd <= 0.0 {
        return Err(FlipError::InvalidParameter(
            "viewing geometry overflows or underflows",
        ));
    }
    Ok(ppd)
}

/// Errors returned by this crate.
#[derive(Debug)]
#[non_exhaustive]
pub enum FlipError {
    /// Dimensions are zero or their buffer size cannot be represented.
    InvalidDimensions,
    /// A buffer length or the two images' dimensions do not match.
    SizeMismatch,
    /// A float input contains NaN or infinity.
    NonFiniteInput,
    /// A parameter would make the reference algorithm undefined.
    InvalidParameter(&'static str),
    /// A fallible buffer reservation failed.
    Allocation(std::collections::TryReserveError),
    /// Loading or saving an image file failed.
    #[cfg(feature = "image")]
    Image(image::ImageError),
}

impl std::fmt::Display for FlipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDimensions => {
                f.write_str("image dimensions must be nonzero and fit in memory")
            }
            Self::SizeMismatch => f.write_str("image dimensions or buffer lengths do not match"),
            Self::NonFiniteInput => f.write_str("input contains a nonfinite value"),
            Self::InvalidParameter(msg) => f.write_str(msg),
            Self::Allocation(err) => write!(f, "buffer allocation failed: {err}"),
            #[cfg(feature = "image")]
            Self::Image(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for FlipError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Allocation(err) => Some(err),
            #[cfg(feature = "image")]
            Self::Image(err) => Some(err),
            _ => None,
        }
    }
}

#[cfg(feature = "image")]
impl From<image::ImageError> for FlipError {
    fn from(err: image::ImageError) -> Self {
        Self::Image(err)
    }
}

impl From<std::collections::TryReserveError> for FlipError {
    fn from(err: std::collections::TryReserveError) -> Self {
        Self::Allocation(err)
    }
}

mod sealed {
    pub trait Sealed: Copy + Send + Sync {
        fn to_f32(self) -> f32;
    }
    impl Sealed for u8 {
        fn to_f32(self) -> f32 {
            f32::from(self) / 255.0
        }
    }
    impl Sealed for f32 {
        fn to_f32(self) -> f32 {
            self
        }
    }
}

/// Channel types accepted by [`ldr_flip`]: `u8` in 0..=255 or `f32` in 0..=1.
///
/// Bytes are divided by 255. Floats outside 0..=1 are clamped, as in the
/// reference. This trait is sealed.
pub trait Channel: sealed::Sealed {}
impl Channel for u8 {}
impl Channel for f32 {}

/// An owned RGB image with interleaved channels and nonzero dimensions.
///
/// LDR evaluation reads it as sRGB, HDR evaluation as linear RGB.
#[derive(Clone, Debug)]
pub struct RgbImage<T> {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) pixels: Vec<T>,
}
impl<T> RgbImage<T> {
    /// Creates an image from exactly `width * height * 3` channels.
    ///
    /// # Errors
    ///
    /// [`FlipError::InvalidDimensions`] for a zero or overflowing size and
    /// [`FlipError::SizeMismatch`] if `pixels` has the wrong length.
    pub fn new(width: usize, height: usize, pixels: Vec<T>) -> Result<Self, FlipError> {
        if dimensions(width, height, 3)? != pixels.len() {
            return Err(FlipError::SizeMismatch);
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }
    /// Width in pixels.
    pub fn width(&self) -> usize {
        self.width
    }
    /// Height in pixels.
    pub fn height(&self) -> usize {
        self.height
    }
    /// Interleaved RGB channels in row-major order.
    pub fn pixels(&self) -> &[T] {
        &self.pixels
    }
    /// Consumes this image and returns the interleaved channels.
    pub fn into_pixels(self) -> Vec<T> {
        self.pixels
    }
}

/// A FLIP error map: one value per pixel, normally in 0..=1.
#[derive(Clone, Debug)]
pub struct ErrorMap {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) pixels: Vec<f32>,
}
impl ErrorMap {
    /// Creates a map from `width * height` errors, for example to pool or
    /// colorize errors computed elsewhere.
    ///
    /// # Errors
    ///
    /// [`FlipError::InvalidDimensions`] or [`FlipError::SizeMismatch`] for a
    /// bad layout, [`FlipError::NonFiniteInput`] for NaN or infinity, and
    /// [`FlipError::InvalidParameter`] for negative values.
    pub fn new(width: usize, height: usize, pixels: Vec<f32>) -> Result<Self, FlipError> {
        if dimensions(width, height, 1)? != pixels.len() {
            return Err(FlipError::SizeMismatch);
        }
        finite(&pixels)?;
        if pixels.iter().any(|v| *v < 0.0) {
            return Err(FlipError::InvalidParameter("errors must be nonnegative"));
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }
    /// Width in pixels.
    pub fn width(&self) -> usize {
        self.width
    }
    /// Height in pixels.
    pub fn height(&self) -> usize {
        self.height
    }
    /// Scalar errors, one `f32` per pixel.
    pub fn pixels(&self) -> &[f32] {
        &self.pixels
    }
    /// Consumes the map and returns its scalar errors.
    pub fn into_pixels(self) -> Vec<f32> {
        self.pixels
    }
    /// Pools the map as the reference tool does: mean, error-weighted
    /// quartiles, extrema and a 100-bin histogram.
    ///
    /// Accumulation stays in `f32` for reference parity. Very large externally
    /// supplied values can overflow the sum; check [`Statistics::finite`]
    /// before using the mean or quartiles.
    pub fn statistics(&self) -> Statistics {
        pooling::statistics(&self.pixels)
    }
    /// Maps each error to the [`MAGMA`] colormap, returning sRGB floats.
    ///
    /// Values outside 0..=1 are clamped.
    /// # Errors
    ///
    /// [`FlipError::InvalidDimensions`] if the RGB expansion cannot be
    /// represented (including on wasm32), or [`FlipError::Allocation`].
    pub fn colorize(&self) -> Result<RgbImage<f32>, FlipError> {
        // FLIP::tensor::colorMap
        let mut pixels = reserved(dimensions(self.width, self.height, 3)?)?;
        for v in &self.pixels {
            pixels.extend_from_slice(&MAGMA[(v.clamp(0.0, 1.0) * 255.0 + 0.5) as usize]);
        }
        Ok(RgbImage {
            width: self.width,
            height: self.height,
            pixels,
        })
    }
}

/// Evaluates LDR-FLIP between two **sRGB** images of equal size.
///
/// Corresponds to `FLIP::evaluate` with `useHDR = false`, after the reference
/// tool's clamp and sRGB-to-linear conversion. The two images may use
/// different channel types. `ppd` is the viewing distance in pixels per degree;
/// see [`DEFAULT_PPD`] and [`pixels_per_degree`].
///
/// # Errors
///
/// [`FlipError::SizeMismatch`] if the dimensions differ,
/// [`FlipError::NonFiniteInput`] for NaN or infinite channels, and
/// [`FlipError::InvalidParameter`] if `ppd` is not finite and positive or
/// yields degenerate filters or exceeds the filter limits: at most 8,193
/// taps per kernel and 2^34 weighted channel additions per evaluation.
/// Full reference kernels are retained even for narrow images; rejecting
/// excessive work avoids changing replicated-border summation order.
/// [`FlipError::Allocation`] if a buffer reservation fails.
pub fn ldr_flip<R: Channel, T: Channel>(
    reference: &RgbImage<R>,
    test: &RgbImage<T>,
    ppd: f32,
) -> Result<ErrorMap, FlipError> {
    if reference.width != test.width || reference.height != test.height {
        return Err(FlipError::SizeMismatch);
    }
    if reference
        .pixels
        .iter()
        .any(|v| !sealed::Sealed::to_f32(*v).is_finite())
        || test
            .pixels
            .iter()
            .any(|v| !sealed::Sealed::to_f32(*v).is_finite())
    {
        return Err(FlipError::NonFiniteInput);
    }
    let filters = filters::Filters::new(ppd, reference.width, reference.height)?;
    let mut workspace = filters.workspace(reference.width, reference.height)?;
    let convert = |p: [f32; 3]| {
        // color3::sRGBToLinearRGB -> LinearRGBToXYZ -> XYZToYCxCz
        color::linear_to_opponent(p.map(|v| color::srgb_to_linear(color::clamp(v))))
    };
    workspace.reference.convert(&reference.pixels, convert);
    workspace.test.convert(&test.pixels, convert);
    filters.evaluate(&mut workspace);
    Ok(ErrorMap {
        width: reference.width,
        height: reference.height,
        pixels: workspace.pixels,
    })
}

pub(crate) fn dimensions(w: usize, h: usize, channels: usize) -> Result<usize, FlipError> {
    dimensions_with_limit(w, h, channels, isize::MAX as usize)
}

fn dimensions_with_limit(
    w: usize,
    h: usize,
    channels: usize,
    byte_limit: usize,
) -> Result<usize, FlipError> {
    let n = w.checked_mul(h).and_then(|n| n.checked_mul(channels));
    match n {
        Some(n) if w > 0 && h > 0 && n <= byte_limit / std::mem::size_of::<f32>() => Ok(n),
        _ => Err(FlipError::InvalidDimensions),
    }
}

pub(crate) fn reserved<T>(len: usize) -> Result<Vec<T>, FlipError> {
    let mut data = Vec::new();
    data.try_reserve_exact(len)?;
    Ok(data)
}

pub(crate) fn zeros(len: usize) -> Result<Vec<f32>, FlipError> {
    let mut data = reserved(len)?;
    data.resize(len, 0.0);
    Ok(data)
}

pub(crate) fn finite(data: &[f32]) -> Result<(), FlipError> {
    if data.iter().any(|v| !v.is_finite()) {
        Err(FlipError::NonFiniteInput)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;

    #[test]
    fn rgb_expansion_checks_wasm32_byte_limit_before_reserving() {
        let limit = i32::MAX as usize;
        assert!(dimensions_with_limit(178_956_971, 1, 1, limit).is_ok());
        assert!(matches!(
            dimensions_with_limit(178_956_971, 1, 3, limit),
            Err(FlipError::InvalidDimensions)
        ));
        assert!(dimensions(usize::MAX, 2, 3).is_err());
        assert!(matches!(
            reserved::<f32>(usize::MAX),
            Err(FlipError::Allocation(_))
        ));
    }

    #[test]
    fn filter_work_counts_all_vertical_feature_accumulators() {
        // At PPD 200, 11M pixels exceeded the work limit only when all eight
        // vertical feature accumulators were included. No image allocation.
        assert!(matches!(
            filters::Filters::new(200.0, 11_000_000, 1),
            Err(FlipError::InvalidParameter(
                "image and PPD exceed the 2^34 filter-work limit"
            ))
        ));
    }
}
