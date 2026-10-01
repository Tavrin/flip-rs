// NVIDIA FLIP color3 functions. Keep expression order and f32 constants intact.
#![allow(clippy::excessive_precision)]

#[inline]
pub(crate) fn clamp(v: f32) -> f32 {
    // color3::clamp / Min(Max(v, 0), 1): NaN from HDR arithmetic maps to 0.
    if v > 0.0 {
        if v > 1.0 {
            1.0
        } else {
            v
        }
    } else {
        0.0
    }
}

#[inline]
pub(crate) fn srgb_to_linear(v: f32) -> f32 {
    // color3::sRGBToLinearRGB
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

#[inline]
pub(crate) fn luminance(v: [f32; 3]) -> f32 {
    // color3::linearRGBToLuminance
    0.2126 * v[0] + 0.7152 * v[1] + 0.0722 * v[2]
}

#[inline]
fn linear_to_xyz(v: [f32; 3]) -> [f32; 3] {
    // color3::LinearRGBToXYZ
    [
        (10135552.0 / 24577794.0) * v[0]
            + (8788810.0 / 24577794.0) * v[1]
            + (4435075.0 / 24577794.0) * v[2],
        (2613072.0 / 12288897.0) * v[0]
            + (8788810.0 / 12288897.0) * v[1]
            + (887015.0 / 12288897.0) * v[2],
        (1425312.0 / 73733382.0) * v[0]
            + (8788810.0 / 73733382.0) * v[1]
            + (70074185.0 / 73733382.0) * v[2],
    ]
}

#[inline]
pub(crate) fn linear_to_opponent(v: [f32; 3]) -> [f32; 3] {
    // color3::XYZToYCxCz(color3::LinearRGBToXYZ(v))
    let [x, y, z] = linear_to_xyz(v);
    let x = x * 1.052156925;
    let z = z * 0.918357670;
    [116.0 * y - 16.0, 500.0 * (x - y), 200.0 * (y - z)]
}

#[inline]
fn lab(v: [f32; 3]) -> [f32; 3] {
    // color3::XYZToCIELab, followed by color3::Hunt
    let [x, y, z] = linear_to_xyz(v);
    let delta = 6.0_f32 / 29.0;
    let delta_square = delta * delta;
    let delta_cube = delta * delta_square;
    let factor = 1.0 / (3.0 * delta_square);
    let term = 4.0 / 29.0;
    let transform = |v: f32| {
        if v > delta_cube {
            v.powf(1.0 / 3.0)
        } else {
            factor * v + term
        }
    };
    let x = transform(x * 1.052156925);
    let y = transform(y);
    let z = transform(z * 0.918357670);
    let l = 116.0 * y - 16.0;
    [
        l,
        0.01 * l * (500.0 * (x - y)),
        0.01 * l * (200.0 * (y - z)),
    ]
}

#[inline]
pub(crate) fn hyab(r: [f32; 3], t: [f32; 3]) -> f32 {
    // color3::HyAB
    let a = r[1] - t[1];
    let b = r[2] - t[2];
    (r[0] - t[0]).abs() + (a * a + b * b).sqrt()
}

pub(crate) fn max_distance() -> f32 {
    // color3::computeMaxDistance
    hyab(lab([0.0, 1.0, 0.0]), lab([0.0, 0.0, 1.0])).powf(0.7)
}

// Converts the first `len` lanes of filtered Y, Cx and Cz planes to
// Hunt-adjusted CIELab in place. The Cz filter has two Gaussian terms, which
// arrive in planes 2 and 3 and are summed here.
#[inline(always)]
pub(crate) fn filtered_lab_planes<const N: usize>(v: &mut [[f32; N]; 4], len: usize) {
    let [yp, cx, cz, cz2] = v;
    // color3::YCxCzToXYZ -> XYZToLinearRGB -> clamp, in independent pixel lanes.
    for (((yp, cx), cz), &cz2) in yp[..len]
        .iter_mut()
        .zip(&mut cx[..len])
        .zip(&mut cz[..len])
        .zip(&cz2[..len])
    {
        let y = (*yp + 16.0) / 116.0;
        let x = (y + *cx / 500.0) * 0.950428545;
        let z = (y - (*cz + cz2) / 200.0) * 1.088900371;
        *yp = clamp(3.241003275 * x + -1.537398934 * y + -0.498615861 * z);
        *cx = clamp(-0.969224334 * x + 1.875930071 * y + 0.041554224 * z);
        *cz = clamp(0.055639423 * x + -0.204011202 * y + 1.057148933 * z);
    }
    // color3::LinearRGBToXYZ -> XYZToCIELab (reference D65 normalization).
    for ((x, y), z) in yp[..len].iter_mut().zip(&mut cx[..len]).zip(&mut cz[..len]) {
        let xyz = linear_to_xyz([*x, *y, *z]);
        *x = xyz[0] * 1.052156925;
        *y = xyz[1];
        *z = xyz[2] * 0.918357670;
    }
    let delta = 6.0_f32 / 29.0;
    let delta_square = delta * delta;
    let delta_cube = delta * delta_square;
    let factor = 1.0 / (3.0 * delta_square);
    let term = 4.0 / 29.0;
    // Retain scalar libm powf; approximating the cube root changes parity.
    for plane in [&mut *yp, &mut *cx, &mut *cz] {
        for v in &mut plane[..len] {
            *v = if *v > delta_cube {
                v.powf(1.0 / 3.0)
            } else {
                factor * *v + term
            };
        }
    }
    // color3::XYZToCIELab -> Hunt
    for ((x, y), z) in yp[..len].iter_mut().zip(&mut cx[..len]).zip(&mut cz[..len]) {
        let l = 116.0 * *y - 16.0;
        let a = 0.01 * l * (500.0 * (*x - *y));
        let b = 0.01 * l * (200.0 * (*y - *z));
        *x = l;
        *y = a;
        *z = b;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Scalar form of `filtered_lab_planes`, one pixel at a time:
    // color3::YCxCzToXYZ -> XYZToLinearRGB -> clamp -> LinearRGBToXYZ -> XYZToCIELab -> Hunt
    fn filtered_lab(v: [f32; 4]) -> [f32; 3] {
        let y = (v[0] + 16.0) / 116.0;
        let x = (y + v[1] / 500.0) * 0.950428545;
        let z = (y - (v[2] + v[3]) / 200.0) * 1.088900371;
        lab([
            clamp(3.241003275 * x + -1.537398934 * y + -0.498615861 * z),
            clamp(-0.969224334 * x + 1.875930071 * y + 0.041554224 * z),
            clamp(0.055639423 * x + -0.204011202 * y + 1.057148933 * z),
        ])
    }

    #[test]
    fn lab_planes_preserve_scalar_rounding() {
        let input: [[f32; 129]; 4] = std::array::from_fn(|c| {
            std::array::from_fn(|i| match c {
                0 => (i as f32 - 20.0) * 1.25,
                1 => (i as f32 - 64.0) * 7.0,
                2 => (32.0 - i as f32) * 4.0,
                _ => (i as f32 - 96.0) * 2.0,
            })
        });
        for len in [1, 3, 4, 7, 8, 16, 127, 128, 129] {
            let expected: Vec<_> = (0..len)
                .map(|i| filtered_lab([input[0][i], input[1][i], input[2][i], input[3][i]]))
                .collect();
            let mut actual = input;
            filtered_lab_planes(&mut actual, len);
            for (i, expected) in expected.iter().enumerate() {
                for c in 0..3 {
                    assert_eq!(actual[c][i].to_bits(), expected[c].to_bits());
                }
            }
            // Partial tiles must not overwrite the unused lanes.
            for c in 0..4 {
                assert_eq!(&actual[c][len..], &input[c][len..]);
            }
        }
    }
}
