use crate::{color, FlipError};
use std::f32::consts::PI;

pub(crate) struct Filters {
    spatial: Vec<[f32; 4]>,
    feature: Vec<[f32; 3]>,
}

impl Filters {
    pub(crate) fn new(ppd: f32, w: usize, h: usize) -> Result<Self, FlipError> {
        // calculateSpatialFilterRadius / setSpatialFilters / setFeatureFilter
        if !ppd.is_finite() || ppd <= 0.0 {
            return Err(FlipError::InvalidParameter(
                "PPD must be finite and positive",
            ));
        }
        let radius = (3.0 * (0.04 / (2.0 * (PI * PI))).sqrt() * ppd).ceil();
        let sigma = 0.5 * 0.082 * ppd;
        let feature_radius = (3.0 * sigma).ceil();
        // Retain full kernels for exact reference reduction order. Bound
        // storage and image-dependent work before any allocation. 2^34
        // permits default-PPD 4K comparisons with headroom.
        if radius.max(feature_radius) > 4096.0 {
            return Err(FlipError::InvalidParameter(
                "PPD exceeds the 8193-tap kernel limit",
            ));
        }
        let spatial_len = 2 * radius as usize + 1;
        let feature_len = 2 * feature_radius as usize + 1;
        let pixels = crate::dimensions(w, h, 1)?;
        // Color: 8 channels in each pass. Features: 6 horizontal and
        // 8 vertical accumulators. Count every weighted channel addition.
        let work = (pixels as u64).checked_mul((16 * spatial_len + 14 * feature_len) as u64);
        if work.is_none_or(|n| n > (1 << 34)) {
            return Err(FlipError::InvalidParameter(
                "image and PPD exceed the 2^34 filter-work limit",
            ));
        }
        let radius = radius as isize;
        let mut spatial = crate::reserved(spatial_len)?;
        let mut sum = [0.0; 4];
        let delta = 1.0 / ppd;
        for i in -radius..=radius {
            let x = i as f32 * delta;
            let x2 = x * x;
            let gaussian = |a: f32, b: f32| a * (PI / b).sqrt() * (-(PI * PI) * x2 / b).exp();
            let gaussian_sqrt =
                |a: f32, b: f32| (a * (PI / b).sqrt()).sqrt() * (-(PI * PI) * x2 / b).exp();
            let weights = [
                gaussian(1.0, 0.0047),
                gaussian(1.0, 0.0053),
                gaussian_sqrt(34.1, 0.04),
                gaussian_sqrt(13.5, 0.025),
            ];
            for c in 0..4 {
                sum[c] += weights[c];
            }
            spatial.push(weights);
        }
        let norm = [
            1.0 / sum[0],
            1.0 / sum[1],
            1.0 / (sum[2] * sum[2] + sum[3] * sum[3]).sqrt(),
        ];
        for weights in &mut spatial {
            weights[0] *= norm[0];
            weights[1] *= norm[1];
            weights[2] *= norm[2];
            weights[3] *= norm[2];
        }
        let mut feature = crate::reserved(feature_len)?;
        let (mut gsum, mut dgneg, mut dgpos, mut ddgneg, mut ddgpos) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for i in -(feature_radius as isize)..=feature_radius as isize {
            let x = i as f32;
            let g = (-(x * x) / (2.0 * sigma * sigma)).exp();
            let dg = -x * g;
            let ddg = (x * x / (sigma * sigma) - 1.0) * g;
            gsum += g;
            if dg > 0.0 {
                dgpos += dg;
            } else {
                dgneg -= dg;
            }
            if ddg > 0.0 {
                ddgpos += ddg;
            } else {
                ddgneg -= ddg;
            }
            feature.push([g, dg, ddg]);
        }
        for v in &mut feature {
            v[0] /= gsum;
            v[1] /= if v[1] > 0.0 { dgpos } else { dgneg };
            v[2] /= if v[2] > 0.0 { ddgpos } else { ddgneg };
        }
        if spatial
            .iter()
            .flatten()
            .chain(feature.iter().flatten())
            .any(|v| !v.is_finite())
        {
            return Err(FlipError::InvalidParameter(
                "PPD produces degenerate reference filters",
            ));
        }
        Ok(Self { spatial, feature })
    }

    pub(crate) fn workspace(&self, w: usize, h: usize) -> Result<Workspace, FlipError> {
        // image::LDR_FLIP: reuse its intermediate images across HDR exposures.
        let padding = (self.spatial.len() / 2).max(self.feature.len() / 2);
        let stride = w
            .checked_add(2 * padding)
            .ok_or(FlipError::InvalidDimensions)?;
        let input_size = crate::dimensions(stride, h, 3)?;
        let intermediate_size = crate::dimensions(w, h, 8)?;
        Ok(Workspace {
            reference: Opponent::new(w, stride, input_size)?,
            test: Opponent::new(w, stride, input_size)?,
            intermediate: crate::zeros(intermediate_size)?,
            pixels: crate::zeros(crate::dimensions(w, h, 1)?)?,
        })
    }

    pub(crate) fn evaluate(&self, workspace: &mut Workspace) {
        // image::LDR_FLIP
        self.color_difference(
            &workspace.reference,
            &workspace.test,
            &mut workspace.intermediate,
            &mut workspace.pixels,
        );
        workspace.reference.normalize_luminance();
        workspace.test.normalize_luminance();
        self.feature_difference_and_final_error(
            &workspace.reference,
            &workspace.test,
            &mut workspace.intermediate,
            &mut workspace.pixels,
        );
    }

    fn color_difference(
        &self,
        r: &Opponent,
        t: &Opponent,
        intermediate: &mut [f32],
        out: &mut [f32],
    ) {
        // image::computeColorDifference (two separable passes)
        let w = r.width;
        let h = out.len() / w;
        let radius = self.spatial.len() / 2;
        rows(intermediate, 8 * w, |y, row| {
            for (c, dst) in row.chunks_exact_mut(w).enumerate() {
                let input = if c < 4 { r } else { t };
                horizontal(
                    input.plane(y, (c % 4).min(2)),
                    dst,
                    &self.spatial,
                    c % 4,
                    input.padding() - radius,
                );
            }
        });
        let cmax = color::max_distance();
        let pccmax = 0.4 * cmax;
        rows(out, w, |y, row| {
            for (tile, dst) in row.chunks_mut(TILE).enumerate() {
                let x = tile * TILE;
                let len = dst.len();
                let mut sum = [[0.0; TILE]; 8];
                for (i, weights) in self.spatial.iter().enumerate() {
                    let yy = border(y, i, radius, h);
                    for (c, accum) in sum.iter_mut().enumerate() {
                        let offset = yy * 8 * w + c * w + x;
                        add_scaled(
                            &mut accum[..len],
                            &intermediate[offset..offset + len],
                            weights[c % 4],
                        );
                    }
                }
                for planes in sum.as_chunks_mut::<4>().0 {
                    color::filtered_lab_planes(planes, len);
                }
                let mut distance = [0.0; TILE];
                for (i, d) in distance[..len].iter_mut().enumerate() {
                    *d = color::hyab(
                        [sum[0][i], sum[1][i], sum[2][i]],
                        [sum[4][i], sum[5][i], sum[6][i]],
                    );
                }
                for d in &mut distance[..len] {
                    *d = d.powf(0.7);
                }
                for (dst, &d) in dst.iter_mut().zip(&distance[..len]) {
                    *dst = if d < pccmax {
                        d * (0.95 / pccmax)
                    } else {
                        0.95 + ((d - pccmax) / (cmax - pccmax)) * (1.0 - 0.95)
                    };
                }
            }
        });
    }

    fn feature_difference_and_final_error(
        &self,
        r: &Opponent,
        t: &Opponent,
        intermediate: &mut [f32],
        out: &mut [f32],
    ) {
        // image::computeFeatureDifferenceAndFinalError
        let w = r.width;
        let h = out.len() / w;
        let radius = self.feature.len() / 2;
        let intermediate = &mut intermediate[..6 * out.len()];
        rows(intermediate, 6 * w, |y, row| {
            for (c, dst) in row.chunks_exact_mut(w).enumerate() {
                let input = if c < 3 { r } else { t };
                horizontal(
                    input.plane(y, 0),
                    dst,
                    &self.feature,
                    (c + 1) % 3,
                    input.padding() - radius,
                );
            }
        });
        let norm = 1.0 / 2.0_f32.sqrt();
        // LLVM otherwise replaces powf(x, 0.5) with sqrt. The last-bit
        // difference can change the winning HDR exposure on near-ties.
        // Keep the reference powf operation, including its rounding.
        let feature_exponent = std::hint::black_box(0.5_f32);
        rows(out, w, |y, row| {
            for (tile, dst) in row.chunks_mut(TILE).enumerate() {
                let x = tile * TILE;
                let len = dst.len();
                let mut sum = [[0.0; TILE]; 8]; // dx, ddx, dy, ddy for each image
                for (i, weights) in self.feature.iter().enumerate() {
                    let yy = border(y, i, radius, h);
                    for (c, accum) in sum.iter_mut().enumerate() {
                        let channel = (c / 4) * 3 + (c % 4).min(2);
                        let weight = weights[(c % 4).saturating_sub(1)];
                        let offset = yy * 6 * w + channel * w + x;
                        add_scaled(
                            &mut accum[..len],
                            &intermediate[offset..offset + len],
                            weight,
                        );
                    }
                }
                let mut features = [0.0; TILE];
                for (i, feature) in features[..len].iter_mut().enumerate() {
                    let edge_r = (sum[0][i] * sum[0][i] + sum[2][i] * sum[2][i]).sqrt();
                    let edge_t = (sum[4][i] * sum[4][i] + sum[6][i] * sum[6][i]).sqrt();
                    let point_r = (sum[1][i] * sum[1][i] + sum[3][i] * sum[3][i]).sqrt();
                    let point_t = (sum[5][i] * sum[5][i] + sum[7][i] * sum[7][i]).sqrt();
                    *feature = (norm * (edge_r - edge_t).abs().max((point_r - point_t).abs()))
                        .powf(feature_exponent);
                }
                for (dst, &feature) in dst.iter_mut().zip(&features[..len]) {
                    *dst = dst.powf(1.0 - feature);
                }
            }
        });
    }
}

// Keep the accumulators in L1 while traversing each tap in reference order.
// Vectorization is across independent pixels, never across the tap reduction.
const TILE: usize = 128;

#[inline(always)]
fn add_scaled(out: &mut [f32], input: &[f32], weight: f32) {
    for (dst, &src) in out.iter_mut().zip(input) {
        *dst += weight * src;
    }
}

#[inline(always)]
fn horizontal<const N: usize>(
    input: &[f32],
    out: &mut [f32],
    weights: &[[f32; N]],
    channel: usize,
    margin: usize,
) {
    // image::computeColorDifference / computeFeatureDifferenceAndFinalError:
    // replicated padding removes border checks from the contiguous tap loops.
    for (tile, dst) in out.chunks_mut(TILE).enumerate() {
        dst.fill(0.0);
        for (i, weights) in weights.iter().enumerate() {
            let offset = tile * TILE + margin + i;
            add_scaled(dst, &input[offset..offset + dst.len()], weights[channel]);
        }
    }
}

#[inline]
fn border(p: usize, tap: usize, radius: usize, size: usize) -> usize {
    // Min(Max(0, coordinate + offset), size - 1)
    (p + tap).saturating_sub(radius).min(size - 1)
}

// Calls `f(y, row)` for each `width`-long row, in parallel for large outputs.
// Rows are independent, so each pixel's reduction order is unaffected.
fn rows<T: Send, F: Fn(usize, &mut [T]) + Sync + Send>(output: &mut [T], width: usize, f: F) {
    #[cfg(feature = "parallel")]
    if output.len() >= 16_384 {
        use rayon::prelude::*;
        output
            .par_chunks_mut(width)
            .enumerate()
            .for_each(|(y, row)| f(y, row));
        return;
    }
    for (y, row) in output.chunks_mut(width).enumerate() {
        f(y, row);
    }
}

pub(crate) struct Workspace {
    pub(crate) reference: Opponent,
    pub(crate) test: Opponent,
    intermediate: Vec<f32>,
    pub(crate) pixels: Vec<f32>,
}

// Three contiguous, horizontally padded Y/Cx/Cz planes per row. The Y plane
// becomes normalized luminance after color filtering, before feature filtering.
pub(crate) struct Opponent {
    width: usize,
    stride: usize,
    data: Vec<f32>,
}

impl Opponent {
    fn new(width: usize, stride: usize, size: usize) -> Result<Self, FlipError> {
        Ok(Self {
            width,
            stride,
            data: crate::zeros(size)?,
        })
    }

    fn padding(&self) -> usize {
        (self.stride - self.width) / 2
    }

    fn plane(&self, y: usize, channel: usize) -> &[f32] {
        let offset = (y * 3 + channel) * self.stride;
        &self.data[offset..offset + self.stride]
    }

    pub(crate) fn convert<T: crate::Channel, F>(&mut self, input: &[T], transform: F)
    where
        F: Fn([f32; 3]) -> [f32; 3] + Send + Sync,
    {
        // image::LinearRGBToYCxCz; callers fuse sRGB conversion or expose/toneMap.
        let w = self.width;
        let stride = self.stride;
        let padding = self.padding();
        rows(&mut self.data, 3 * stride, |y, row| {
            let (yp, chroma) = row.split_at_mut(stride);
            let (cx, cz) = chroma.split_at_mut(stride);
            let src = &input[y * w * 3..(y + 1) * w * 3];
            for (((y, cx), cz), src) in yp[padding..padding + w]
                .iter_mut()
                .zip(&mut cx[padding..padding + w])
                .zip(&mut cz[padding..padding + w])
                .zip(src.as_chunks::<3>().0)
            {
                let v = transform(src.map(crate::sealed::Sealed::to_f32));
                *y = v[0];
                *cx = v[1];
                *cz = v[2];
            }
            for plane in [yp, cx, cz] {
                let first = plane[padding];
                let last = plane[padding + w - 1];
                plane[..padding].fill(first);
                plane[padding + w..].fill(last);
            }
        });
    }

    fn normalize_luminance(&mut self) {
        // image::computeFeatureDifferenceAndFinalError: compute once per pixel,
        // in the same order as the reference's per-tap Y normalization.
        let stride = self.stride;
        rows(&mut self.data, 3 * stride, |_, row| {
            for v in &mut row[..stride] {
                *v = *v * (1.0 / 116.0) + (16.0 / 116.0);
            }
        });
    }
}
