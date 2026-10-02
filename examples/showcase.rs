//! Generates the README images and the browser demo's sample pair.
//!
//! ```sh
//! cargo run --release --features image --example showcase -- docs/img examples/web
//! ```
//!
//! The scenes are procedural, so no third-party images are needed. The
//! reference is rendered with 16 samples per pixel. The test image uses one
//! sample per pixel, adds per-pixel noise, blurs the sign and shifts the sky
//! color, which imitates the differences between two renderers.
//!
//! Outputs, all PNG:
//!
//! - `IMG_DIR/hero.png`: reference | test | LDR-FLIP heatmap.
//! - `IMG_DIR/hdr-exposures.png`: the HDR reference tone-mapped at several
//!   exposures (top), the LDR-FLIP heatmap at each exposure (bottom), and the
//!   HDR-FLIP heatmap, which keeps the largest error per pixel (last column).
//! - `WEB_DIR/sample-reference.png` and `WEB_DIR/sample-test.png`.

use flip_rs::{hdr_flip, io, ldr_flip, HdrOptions, RgbImage, DEFAULT_PPD};
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const USAGE: &str = "usage: showcase IMG_DIR WEB_DIR";

/// Space between panels in the composite images, in pixels.
const GAP: usize = 6;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [img_dir, web_dir] = &args[..] else {
        return Err(USAGE.into());
    };
    let (img_dir, web_dir) = (Path::new(img_dir), Path::new(web_dir));

    // Hero triplet.
    let (w, h) = (300, 200);
    let reference = srgb(&render(w, h, Variant::Reference)?, 0.0)?;
    let test = srgb(&render(w, h, Variant::Test)?, 0.0)?;
    let map = ldr_flip(&reference, &test, DEFAULT_PPD)?;
    let s = map.statistics();
    println!(
        "hero {w}x{h}: mean {:.4}, weighted median {:.4}, max {:.4}",
        s.mean, s.weighted_median, s.max
    );
    let heatmap = map.colorize()?;
    let hero = compose(&[&[&reference, &test, &heatmap]])?;
    io::save_rgb(&hero, img_dir.join("hero.png"))?;

    // Sample pair for the browser demo.
    let (w, h) = (480, 320);
    io::save_rgb(
        &srgb(&render(w, h, Variant::Reference)?, 0.0)?,
        web_dir.join("sample-reference.png"),
    )?;
    io::save_rgb(
        &srgb(&render(w, h, Variant::Test)?, 0.0)?,
        web_dir.join("sample-test.png"),
    )?;

    // HDR exposure strip.
    let (w, h) = (200, 136);
    let reference = render(w, h, Variant::Reference)?;
    let test = render(w, h, Variant::Test)?;
    let full = hdr_flip(&reference, &test, HdrOptions::default())?;
    let p = full.parameters;
    let s = full.error_map.statistics();
    println!(
        "hdr {w}x{h}: exposures {:.3}..{:.3} ({}), mean {:.4}, max {:.4}",
        p.start_exposure, p.stop_exposure, p.num_exposures, s.mean, s.max
    );
    let columns = 4;
    let mut top = Vec::new();
    let mut bottom = Vec::new();
    for i in 0..columns {
        let t = i as f32 / (columns - 1) as f32;
        let exposure = p.start_exposure + t * (p.stop_exposure - p.start_exposure);
        // Equal endpoints evaluate a single exposure.
        let mut options = HdrOptions::default();
        options.start_exposure = Some(exposure);
        options.stop_exposure = Some(exposure);
        options.num_exposures = Some(2);
        options.return_exposure_map = false;
        let single = hdr_flip(&reference, &test, options)?;
        println!(
            "  exposure {exposure:+.3}: mean {:.4}",
            single.error_map.statistics().mean
        );
        top.push(srgb(&reference, exposure)?);
        bottom.push(single.error_map.colorize()?);
    }
    // The last column shows the test image at the middle exposure above the
    // HDR-FLIP result.
    let middle = 0.5 * (p.start_exposure + p.stop_exposure);
    top.push(srgb(&test, middle)?);
    bottom.push(full.error_map.colorize()?);
    let top: Vec<&RgbImage<f32>> = top.iter().collect();
    let bottom: Vec<&RgbImage<f32>> = bottom.iter().collect();
    let strip = compose(&[&top, &bottom])?;
    io::save_rgb(&strip, img_dir.join("hdr-exposures.png"))?;
    Ok(())
}

#[derive(Clone, Copy, PartialEq)]
enum Variant {
    Reference,
    Test,
}

/// Renders the scene as linear RGB radiance. The sun is far brighter than 1.
fn render(w: usize, h: usize, variant: Variant) -> Result<RgbImage<f32>> {
    let samples = match variant {
        Variant::Reference => 4,
        Variant::Test => 1,
    };
    let mut pixels = Vec::with_capacity(w * h * 3);
    for y in 0..h {
        for x in 0..w {
            let mut sum = [0.0_f32; 3];
            for sy in 0..samples {
                for sx in 0..samples {
                    let u = (x as f32 + (sx as f32 + 0.5) / samples as f32) / w as f32;
                    let v = (y as f32 + (sy as f32 + 0.5) / samples as f32) / h as f32;
                    let c = shade(u, v * h as f32 / w as f32, variant);
                    for k in 0..3 {
                        sum[k] += c[k];
                    }
                }
            }
            let n = (samples * samples) as f32;
            let mut c = sum.map(|s| s / n);
            if variant == Variant::Test {
                // Per-pixel noise, as from a low sample count.
                let noise = 1.0 + 0.35 * (hash(x as u32, y as u32, 7) - 0.5);
                c = c.map(|v| v * noise);
            }
            pixels.extend_from_slice(&c);
        }
    }
    let mut image = RgbImage::new(w, h, pixels)?;
    if variant == Variant::Test {
        let (x0, y0, x1, y1) = sign_rect(w, h);
        image = box_blur(&image, x0, y0, x1, y1, 2)?;
    }
    Ok(image)
}

/// Pixel bounds of the sign, which carries the text-like detail.
fn sign_rect(w: usize, h: usize) -> (usize, usize, usize, usize) {
    let aspect = h as f32 / w as f32;
    let to_y = |v: f32| (v / aspect * h as f32) as usize;
    (
        (0.06 * w as f32) as usize,
        to_y(0.10),
        (0.40 * w as f32) as usize,
        to_y(0.32),
    )
}

/// Radiance at `(u, v)`, where `u` is 0..1 across and `v` runs down with the
/// same scale.
fn shade(u: f32, v: f32, variant: Variant) -> [f32; 3] {
    let horizon = 0.40;
    // Sign with lines of glyph-like blocks.
    if (0.06..0.40).contains(&u) && (0.10..0.32).contains(&v) {
        let border = u < 0.07 || u > 0.39 || v < 0.11 || v > 0.31;
        if border {
            return [0.05, 0.05, 0.06];
        }
        let line = ((v - 0.13) / 0.035).floor();
        let in_line = ((v - 0.13) / 0.035).fract() < 0.6 && (0.0..5.0).contains(&line);
        let column = ((u - 0.09) / 0.012).floor();
        let glyph = hash(column as u32, line as u32, 3);
        let gx = ((u - 0.09) / 0.012).fract();
        let gy = ((v - 0.13) / 0.035).fract() / 0.6;
        let stroke = gx < 0.7 && (glyph > 0.25) && (gy < 0.25 || gx < 0.22 || glyph > 0.7);
        let word_gap = hash(column as u32 / 5, line as u32, 4) < 0.2;
        if in_line && u < 0.37 && stroke && !word_gap {
            return [0.04, 0.05, 0.08];
        }
        return [0.85, 0.82, 0.70];
    }
    // Two thin cables.
    for (a, b) in [(0.06, 0.18), (0.09, 0.24)] {
        let cable = a + (b - a) * u - 0.05 * (u * std::f32::consts::PI).sin();
        if (v - cable).abs() < 0.0012 && u > 0.40 {
            return [0.02, 0.02, 0.02];
        }
    }
    if v < horizon {
        // Sky gradient, a sun disc and a glow.
        let t = v / horizon;
        let mut sky = [0.15 + 0.55 * t, 0.30 + 0.50 * t, 0.75 + 0.20 * t];
        if variant == Variant::Test {
            sky = [sky[0] * 0.92, sky[1] * 1.0, sky[2] * 1.08];
        }
        let d = ((u - 0.75).powi(2) + (v - 0.17).powi(2)).sqrt();
        if d < 0.035 {
            return [60.0, 52.0, 40.0];
        }
        let glow = 0.6 * (-d * 18.0).exp();
        return [
            sky[0] + glow * 3.0,
            sky[1] + glow * 2.4,
            sky[2] + glow * 1.6,
        ];
    }
    // Ground: a checkerboard in perspective, textured with value noise and
    // fading into haze at the horizon.
    let depth = 0.12 / (v - horizon);
    let gx = (u - 0.5) * depth * 12.0;
    let gz = depth * 12.0;
    let check = (gx.floor() + gz.floor()) as i64 & 1 == 0;
    let grain = 0.75 + 0.5 * value_noise(gx * 4.0, gz * 4.0);
    let base = if check {
        [0.55, 0.32, 0.20]
    } else {
        [0.18, 0.16, 0.14]
    };
    let haze = (-depth * 0.12).exp();
    let fog = [0.65, 0.72, 0.85];
    std::array::from_fn(|k| base[k] * grain * haze + fog[k] * (1.0 - haze))
}

/// Deterministic hash of three integers to 0..1.
fn hash(x: u32, y: u32, seed: u32) -> f32 {
    let mut h = x
        .wrapping_mul(0x8da6_b343)
        .wrapping_add(y.wrapping_mul(0xd816_3841))
        .wrapping_add(seed.wrapping_mul(0xcb1a_b31f));
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297a_2d39);
    h ^= h >> 15;
    (h >> 8) as f32 / (1 << 24) as f32
}

/// Bilinearly interpolated value noise in 0..1.
fn value_noise(x: f32, y: f32) -> f32 {
    let (xi, yi) = (x.floor(), y.floor());
    let (fx, fy) = (x - xi, y - yi);
    let at = |dx: f32, dy: f32| hash((xi + dx) as i32 as u32, (yi + dy) as i32 as u32, 11);
    let top = at(0.0, 0.0) * (1.0 - fx) + at(1.0, 0.0) * fx;
    let bottom = at(0.0, 1.0) * (1.0 - fx) + at(1.0, 1.0) * fx;
    top * (1.0 - fy) + bottom * fy
}

/// Box-blurs the rectangle `x0..x1`, `y0..y1` with the given radius.
fn box_blur(
    image: &RgbImage<f32>,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    radius: usize,
) -> Result<RgbImage<f32>> {
    let (w, h) = (image.width(), image.height());
    let src = image.pixels();
    let mut out = src.to_vec();
    for y in y0..y1.min(h) {
        for x in x0..x1.min(w) {
            let mut sum = [0.0_f32; 3];
            let mut n = 0.0;
            for sy in y.saturating_sub(radius)..(y + radius + 1).min(h) {
                for sx in x.saturating_sub(radius)..(x + radius + 1).min(w) {
                    for k in 0..3 {
                        sum[k] += src[(sy * w + sx) * 3 + k];
                    }
                    n += 1.0;
                }
            }
            for k in 0..3 {
                out[(y * w + x) * 3 + k] = sum[k] / n;
            }
        }
    }
    Ok(RgbImage::new(w, h, out)?)
}

/// Tone-maps linear radiance at `exposure` stops with FLIP's ACES curve and
/// encodes it as sRGB in 0..=1.
fn srgb(image: &RgbImage<f32>, exposure: f32) -> Result<RgbImage<f32>> {
    let scale = exposure.exp2();
    let (a, b, c, d, e) = (
        0.6 * 0.6 * 2.51,
        0.6 * 0.03,
        0.6 * 0.6 * 2.43,
        0.6 * 0.59,
        0.14,
    );
    let pixels = image
        .pixels()
        .iter()
        .map(|&v| {
            let x = v * scale;
            let t = ((x * x * a + x * b) / (x * x * c + x * d + e)).clamp(0.0, 1.0);
            if t <= 0.0031308 {
                12.92 * t
            } else {
                1.055 * t.powf(1.0 / 2.4) - 0.055
            }
        })
        .collect();
    Ok(RgbImage::new(image.width(), image.height(), pixels)?)
}

/// Lays out equally sized images in rows, separated by [`GAP`] pixels of a
/// neutral gray.
fn compose(rows: &[&[&RgbImage<f32>]]) -> Result<RgbImage<f32>> {
    let (pw, ph) = (rows[0][0].width(), rows[0][0].height());
    let columns = rows[0].len();
    let w = columns * pw + (columns - 1) * GAP;
    let h = rows.len() * ph + (rows.len() - 1) * GAP;
    let mut pixels = vec![0.5_f32; w * h * 3];
    for (r, row) in rows.iter().enumerate() {
        for (c, image) in row.iter().enumerate() {
            let (ox, oy) = (c * (pw + GAP), r * (ph + GAP));
            for y in 0..ph {
                let src = &image.pixels()[y * pw * 3..(y + 1) * pw * 3];
                let start = ((oy + y) * w + ox) * 3;
                pixels[start..start + pw * 3].copy_from_slice(src);
            }
        }
    }
    Ok(RgbImage::new(w, h, pixels)?)
}
