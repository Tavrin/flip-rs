use crate::{color, filters::Filters, finite, ErrorMap, FlipError, RgbImage};

/// Tone curves applied to each exposure, as in reference `image::toneMap`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Tonemapper {
    /// The reference's ACES filmic approximation (the default).
    #[default]
    Aces,
    /// Hable's filmic curve.
    Hable,
    /// Reinhard, scaling each pixel by `1 / (1 + luminance)`.
    Reinhard,
}
impl Tonemapper {
    fn coefficients(self) -> [f32; 6] {
        // FLIP::ToneMappingCoefficients
        match self {
            Self::Reinhard => [0.0, 1.0, 0.0, 0.0, 1.0, 1.0],
            Self::Aces => [
                0.6 * 0.6 * 2.51,
                0.6 * 0.03,
                0.0,
                0.6 * 0.6 * 2.43,
                0.6 * 0.59,
                0.14,
            ],
            Self::Hable => [0.231683, 0.013791, 0.0, 0.18, 0.3, 0.018],
        }
    }
}

/// Options for [`hdr_flip`].
///
/// Each exposure field left as `None` is computed from the reference image,
/// as the reference tool does. Start from [`HdrOptions::default`] and set the
/// fields you need:
///
/// ```
/// let mut options = flip_rs::HdrOptions::default();
/// options.tonemapper = flip_rs::Tonemapper::Hable;
/// options.start_exposure = Some(-2.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct HdrOptions {
    /// Pixels per degree. Defaults to [`crate::DEFAULT_PPD`].
    pub ppd: f32,
    /// First exposure in stops; exposure `e` scales linear RGB by `2^e`.
    pub start_exposure: Option<f32>,
    /// Last exposure in stops, inclusive.
    pub stop_exposure: Option<f32>,
    /// Number of equally spaced exposures, at least two.
    pub num_exposures: Option<usize>,
    /// Tone curve. Defaults to ACES.
    pub tonemapper: Tonemapper,
    /// Whether to return an [`ExposureMap`]. Defaults to `true`.
    pub return_exposure_map: bool,
}
impl Default for HdrOptions {
    fn default() -> Self {
        Self {
            ppd: crate::DEFAULT_PPD,
            start_exposure: None,
            stop_exposure: None,
            num_exposures: None,
            tonemapper: Tonemapper::Aces,
            return_exposure_map: true,
        }
    }
}

/// The parameters an HDR evaluation used, with automatic values resolved.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct HdrParameters {
    /// Pixels per degree used by every LDR evaluation.
    pub ppd: f32,
    /// First exposure in stops.
    pub start_exposure: f32,
    /// Last exposure in stops.
    pub stop_exposure: f32,
    /// Number of exposures evaluated.
    pub num_exposures: usize,
    /// Tone curve used.
    pub tonemapper: Tonemapper,
}

/// For each pixel, the exposure that gave the largest error, as the
/// normalized index `i / (num_exposures - 1)`.
///
/// Ties keep the earliest exposure, so pixels with zero error have index zero.
#[derive(Clone, Debug)]
pub struct ExposureMap {
    width: usize,
    height: usize,
    pixels: Vec<f32>,
}
impl ExposureMap {
    /// Width in pixels.
    pub fn width(&self) -> usize {
        self.width
    }
    /// Height in pixels.
    pub fn height(&self) -> usize {
        self.height
    }
    /// Normalized exposure indices in row-major order.
    pub fn pixels(&self) -> &[f32] {
        &self.pixels
    }
    /// Consumes the map and returns the normalized indices.
    pub fn into_pixels(self) -> Vec<f32> {
        self.pixels
    }
}

/// The output of [`hdr_flip`].
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct HdrResult {
    /// Maximum LDR error across all evaluated exposures.
    pub error_map: ErrorMap,
    /// Exposure indices, if [`HdrOptions::return_exposure_map`] was set.
    pub exposure_map: Option<ExposureMap>,
    /// The parameters used, including automatically chosen exposures.
    pub parameters: HdrParameters,
}

/// Evaluates HDR-FLIP between two **linear RGB** images of equal size.
///
/// Corresponds to `FLIP::evaluate` with `useHDR = true`. Automatic exposure
/// endpoints come from the reference image only: the start from its maximum
/// luminance, the stop from its median luminance. The automatic count is
/// `max(2, ceil(stop - start))`. Both endpoints are evaluated, and each pixel
/// keeps its largest error.
///
/// # Errors
///
/// [`FlipError::SizeMismatch`] if the dimensions differ,
/// [`FlipError::NonFiniteInput`] for NaN or infinite values, and
/// [`FlipError::InvalidParameter`] for a bad `ppd`, a reversed or nonfinite
/// exposure range, or fewer than two exposures. An all-black reference has no
/// defined automatic start exposure (the reference tool exits); pass explicit
/// endpoints to compare black images.
pub fn hdr_flip(
    reference: &RgbImage<f32>,
    test: &RgbImage<f32>,
    options: HdrOptions,
) -> Result<HdrResult, FlipError> {
    if reference.width != test.width || reference.height != test.height {
        return Err(FlipError::SizeMismatch);
    }
    let (width, height) = (reference.width, reference.height);
    let (reference, test) = (&reference.pixels[..], &test.pixels[..]);
    finite(reference)?;
    finite(test)?;
    let filters = Filters::new(options.ppd)?;
    let parameters = resolve(reference, options)?;
    let mut error_map = ErrorMap {
        width,
        height,
        pixels: vec![0.0; width * height],
    };
    let mut exposure_map = options.return_exposure_map.then(|| ExposureMap {
        width,
        height,
        pixels: vec![0.0; width * height],
    });
    let step = (parameters.stop_exposure - parameters.start_exposure)
        / (parameters.num_exposures - 1) as f32;
    let mut workspace = filters.workspace(width, height)?;
    for i in 0..parameters.num_exposures {
        let exposure = parameters.start_exposure + i as f32 * step;
        let multiplier = 2.0_f32.powf(exposure); // image::expose
        let convert = |v: [f32; 3]| {
            // image::expose -> toneMap -> LinearRGBToYCxCz
            let exposed = v.map(|c| c * multiplier);
            color::linear_to_opponent(tone_map(exposed, options.tonemapper).map(color::clamp))
        };
        workspace.reference.convert(reference, convert);
        workspace.test.convert(test, convert);
        filters.evaluate(&mut workspace);
        // image::setMaxExposure: a strict comparison preserves the first maximum.
        for (j, (dst, &src)) in error_map
            .pixels
            .iter_mut()
            .zip(&workspace.pixels)
            .enumerate()
        {
            if src > *dst {
                *dst = src;
                if let Some(map) = &mut exposure_map {
                    map.pixels[j] = i as f32 / (parameters.num_exposures - 1) as f32;
                }
            }
        }
    }
    Ok(HdrResult {
        error_map,
        exposure_map,
        parameters,
    })
}

fn resolve(reference: &[f32], options: HdrOptions) -> Result<HdrParameters, FlipError> {
    // image::computeExposures / evaluate exposure parameter resolution
    let (start, stop) = match (options.start_exposure, options.stop_exposure) {
        (Some(start), Some(stop)) => (start, stop),
        (start, stop) => {
            let tc = options.tonemapper.coefficients();
            let a = tc[0] - 0.85 * tc[3];
            let b = tc[1] - 0.85 * tc[4];
            let c = tc[2] - 0.85 * tc[5];
            let xmax = if a == 0.0 {
                -c / b
            } else {
                // FLIP::solveSecondDegree
                let d1 = -0.5 * (b / a);
                let d2 = (d1 * d1 - c / a).sqrt();
                d1 + d2
            };
            let mut luminances: Vec<f32> = reference
                .as_chunks::<3>()
                .0
                .iter()
                .map(|p| color::luminance(*p))
                .collect();
            let ymax = luminances.iter().copied().fold(-1e30_f32, f32::max);
            let median_index = luminances.len() / 2;
            let (_, median, _) = luminances.select_nth_unstable_by(median_index, f32::total_cmp);
            let median = median.max(f32::EPSILON);
            (
                start.unwrap_or_else(|| f64::from(xmax / ymax).log2() as f32),
                stop.unwrap_or_else(|| f64::from(xmax / median).log2() as f32),
            )
        }
    };
    let range = stop - start;
    if !start.is_finite() || !stop.is_finite() || !range.is_finite() || range < 0.0 {
        return Err(FlipError::InvalidParameter(
            "exposure endpoints must be finite and ordered (an all-black reference needs explicit endpoints)",
        ));
    }
    let count = match options.num_exposures {
        Some(n) => n,
        None => range.ceil().max(2.0) as usize,
    };
    if count < 2 || count > i32::MAX as usize {
        return Err(FlipError::InvalidParameter(
            "exposure count must be between 2 and i32::MAX",
        ));
    }
    Ok(HdrParameters {
        ppd: options.ppd,
        start_exposure: start,
        stop_exposure: stop,
        num_exposures: count,
        tonemapper: options.tonemapper,
    })
}

#[inline]
fn tone_map(v: [f32; 3], tm: Tonemapper) -> [f32; 3] {
    // image::toneMap
    if tm == Tonemapper::Reinhard {
        let factor = 1.0 / (1.0 + color::luminance(v));
        return v.map(|c| c * factor);
    }
    let tc = tm.coefficients();
    v.map(|c| ((c * c) * tc[0] + c * tc[1] + tc[2]) / ((c * c) * tc[3] + c * tc[4] + tc[5]))
}
