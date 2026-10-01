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
        mean: sum / data.len() as f32,
        weighted_median: percentile(0.5),
        first_quartile: percentile(0.25),
        third_quartile: percentile(0.75),
        min,
        max,
        histogram,
    }
}
