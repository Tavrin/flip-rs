use crate::{reserved, FlipError};

/// How a percentile is pooled from the sorted errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Weighting {
    /// Weight each error by its value, as for [`Statistics`]' quartiles.
    Weighted,
    /// Give each pixel equal weight, using the reference's zero-based index.
    Unweighted,
}

/// A sorted copy of an error map for repeated percentile queries.
///
/// Construct this with [`ErrorMap::percentiles`](crate::ErrorMap::percentiles).
/// Sorting and the row-major `f32` total are computed once. Queries do not
/// allocate or sort again; weighted queries scan the sorted errors and
/// unweighted queries index them directly.
#[derive(Debug)]
pub struct Percentiles {
    sorted: Vec<f32>,
    sum: f32,
}

impl Percentiles {
    pub(crate) fn new(data: &[f32]) -> Result<Self, FlipError> {
        if data.is_empty() {
            return Err(FlipError::InvalidDimensions);
        }
        let mut sorted = reserved(data.len())?;
        sorted.extend_from_slice(data);
        let sum = data.iter().fold(0.0_f32, |sum, &v| sum + v);
        sorted.sort_unstable_by(f32::total_cmp);
        Ok(Self { sorted, sum })
    }

    /// Returns the percentile at the fraction `p` in `0..=1`.
    ///
    /// This follows reference `pooling<float>::getPercentile(p, weighted)`:
    /// [`Weighting::Weighted`] returns the first sorted error whose running
    /// `f32` sum is strictly greater than `p` times the row-major `f32` total,
    /// or zero when none exceeds it. All-zero maps therefore return zero.
    /// Overflow of the total retains the reference arithmetic and fallback.
    ///
    /// [`Weighting::Unweighted`] uses the zero-based index
    /// `ceil((count as f32) * p)`, without interpolation or subtracting one.
    /// This index can be out of bounds even below `p = 1`, and is usually
    /// out of bounds at `p = 1`. Counts too large to be represented exactly
    /// in `f32` retain the reference's count rounding.
    ///
    /// # Errors
    ///
    /// [`FlipError::InvalidParameter`] if `p` is NaN, infinite, outside
    /// `0..=1`, or the unweighted index is out of bounds. The reference has
    /// no defined result for those queries, so the index is not clamped.
    pub fn percentile(&self, p: f32, weighting: Weighting) -> Result<f32, FlipError> {
        validate_percentile(p)?;
        match weighting {
            Weighting::Weighted => {
                let mut running = 0.0_f32;
                for &v in &self.sorted {
                    running += v;
                    if running > p * self.sum {
                        return Ok(v);
                    }
                }
                Ok(0.0)
            }
            Weighting::Unweighted => {
                let index = ((self.sorted.len() as f32) * p).ceil() as usize;
                self.sorted
                    .get(index)
                    .copied()
                    .ok_or(FlipError::InvalidParameter(
                        "unweighted percentile index is out of bounds",
                    ))
            }
        }
    }
}

pub(crate) fn validate_percentile(p: f32) -> Result<(), FlipError> {
    if !(0.0..=1.0).contains(&p) {
        return Err(FlipError::InvalidParameter(
            "percentile must be finite and in 0..=1",
        ));
    }
    Ok(())
}

/// A 100-bin histogram of errors over 0..=1, as in the reference tool.
///
/// An error of exactly 1 falls in the last bin.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Histogram {
    /// Number of errors in each bin.
    pub counts: [usize; 100],
    /// Number of errors outside 0..=1, which only a map built with
    /// [`ErrorMap::new`](crate::ErrorMap::new) can contain.
    pub out_of_range: usize,
}
impl Histogram {
    /// Each bin's count multiplied by its center value, as in the reference
    /// tool's histogram plots.
    pub fn weighted_counts(&self) -> [f64; 100] {
        // histogram::toPython
        std::array::from_fn(|i| {
            self.counts[i] as f64 * ((i as f64 + 0.5) * f64::from(1.0_f32 / 100.0))
        })
    }
}

/// Pooled statistics of an error map, as computed by reference
/// `FLIPPooling::pooling<float>`.
///
/// The quartiles are error-weighted: the median is the smallest sorted error
/// at which the running sum of errors exceeds half the total.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct Statistics {
    /// Whether the `f32` total and reported statistics are finite. When false,
    /// mean and weighted quartiles are unreliable; histogram and extrema
    /// remain usable. This retains reference arithmetic without hiding overflow.
    pub finite: bool,
    /// Mean error, summed sequentially in `f32` in row-major order.
    pub mean: f32,
    /// Error-weighted median.
    pub weighted_median: f32,
    /// Error-weighted first quartile.
    pub first_quartile: f32,
    /// Error-weighted third quartile.
    pub third_quartile: f32,
    /// Smallest error.
    pub min: f32,
    /// Largest error. For an all-zero map this is `f32::MIN_POSITIVE`, as in
    /// the reference.
    pub max: f32,
    /// Histogram of the errors.
    pub histogram: Histogram,
}

pub(crate) fn statistics(data: &[f32]) -> Statistics {
    // pooling::update / getMean / getPercentile(percent, true)
    let mut sum = 0.0_f32;
    let mut min = f32::MAX;
    let mut max = f32::MIN_POSITIVE;
    let mut histogram = Histogram {
        counts: [0; 100],
        out_of_range: 0,
    };
    for &v in data {
        sum += v;
        min = min.min(v);
        max = max.max(v);
        if (0.0..=1.0).contains(&v) {
            let idx = ((f64::from(v) / f64::from(1.0_f32 / 100.0)) as usize).min(99);
            histogram.counts[idx] += 1;
        } else {
            histogram.out_of_range += 1;
        }
    }
    let mut sorted = data.to_vec();
    sorted.sort_unstable_by(f32::total_cmp);
    let percentile = |p: f32| {
        let mut running = 0.0_f32;
        for &v in &sorted {
            running += v;
            if running > p * sum {
                return v;
            }
        }
        0.0
    };
    Statistics {
        finite: sum.is_finite(),
        mean: sum / data.len() as f32,
        weighted_median: percentile(0.5),
        first_quartile: percentile(0.25),
        third_quartile: percentile(0.75),
        min,
        max,
        histogram,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_percentile_pool_is_rejected() {
        assert!(matches!(
            Percentiles::new(&[]),
            Err(FlipError::InvalidDimensions)
        ));
    }
}
