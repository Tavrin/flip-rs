use flip_rs::{hdr_flip, ldr_flip, ErrorMap, HdrOptions, RgbImage, Tonemapper, DEFAULT_PPD};
use std::{
    error::Error,
    path::{Path, PathBuf},
    process::Command,
};
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// HDR options with the given viewing distance, tone curve and exposures.
pub fn hdr_options(
    ppd: f32,
    tonemapper: Tonemapper,
    start: Option<f32>,
    stop: Option<f32>,
    count: Option<usize>,
) -> HdrOptions {
    let mut options = HdrOptions::default();
    options.ppd = ppd;
    options.tonemapper = tonemapper;
    options.start_exposure = start;
    options.stop_exposure = stop;
    options.num_exposures = count;
    options
}

pub struct Case {
    pub name: String,
    pub group: &'static str,
    pub w: usize,
    pub h: usize,
    pub r: Vec<f32>,
    pub t: Vec<f32>,
    pub options: Option<HdrOptions>,
    pub ppd: f32,
}
pub struct Output {
    pub map: ErrorMap,
    pub exposure: Option<Vec<f32>>,
    pub used: Option<(f32, f32, usize)>,
    pub seconds: f64,
}
impl Case {
    pub fn rust(&self) -> Result<Output> {
        // Build the images outside the timed call; the C++ timing also excludes input loading.
        let r = RgbImage::new(self.w, self.h, self.r.clone())?;
        let t = RgbImage::new(self.w, self.h, self.t.clone())?;
        let start = std::time::Instant::now();
        if let Some(options) = self.options {
            let result = hdr_flip(&r, &t, options)?;
            let seconds = start.elapsed().as_secs_f64();
            Ok(Output {
                map: result.error_map,
                exposure: result.exposure_map.map(|m| m.into_pixels()),
                used: Some((
                    result.parameters.start_exposure,
                    result.parameters.stop_exposure,
                    result.parameters.num_exposures,
                )),
                seconds,
            })
        } else {
            let map = ldr_flip(&r, &t, self.ppd)?;
            Ok(Output {
                map,
                exposure: None,
                used: None,
                seconds: start.elapsed().as_secs_f64(),
            })
        }
    }
}

pub enum Probe {
    Output(Output),
    Rejected {
        code: Option<i32>,
        diagnostic: String,
    },
}

pub struct Oracle {
    bin: PathBuf,
    dir: PathBuf,
}
impl Oracle {
    pub fn new(bin: impl AsRef<Path>) -> Result<Self> {
        let dir = std::env::temp_dir().join(format!(
            "flip-parity-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        std::fs::create_dir(&dir)?;
        Ok(Self {
            bin: bin.as_ref().to_owned(),
            dir,
        })
    }
    pub fn cpp(&self, case: &Case, repeats: usize) -> Result<Output> {
        match self.probe(case, repeats)? {
            Probe::Output(output) => Ok(output),
            Probe::Rejected { code, diagnostic } => {
                Err(format!("C++ failed for {} (exit {code:?}): {diagnostic}", case.name).into())
            }
        }
    }
    pub fn probe(&self, case: &Case, repeats: usize) -> Result<Probe> {
        let r = self.dir.join("reference.f32");
        let t = self.dir.join("test.f32");
        let out = self.dir.join("output.bin");
        for (path, data) in [(&r, &case.r), (&t, &case.t)] {
            let bytes: Vec<u8> = data.iter().flat_map(|v| v.to_le_bytes()).collect();
            std::fs::write(path, bytes)?;
        }
        let options = case.options.unwrap_or_default();
        let endpoint = |e: Option<f32>| e.map_or("auto".into(), |v| v.to_string());
        let tm = match options.tonemapper {
            Tonemapper::Aces => "aces",
            Tonemapper::Hable => "hable",
            Tonemapper::Reinhard => "reinhard",
            other => return Err(format!("no C++ name for {other:?}").into()),
        };
        let status = Command::new(&self.bin)
            .args([
                if case.options.is_some() {
                    "hdr".into()
                } else {
                    "ldr".into()
                },
                case.w.to_string(),
                case.h.to_string(),
                case.ppd.to_string(),
                tm.into(),
                endpoint(options.start_exposure),
                endpoint(options.stop_exposure),
                options
                    .num_exposures
                    .map_or("auto".into(), |n| n.to_string()),
            ])
            .arg(&r)
            .arg(&t)
            .arg(&out)
            .arg(repeats.to_string())
            .output()?;
        if !status.status.success() {
            return Ok(Probe::Rejected {
                code: status.status.code(),
                diagnostic: format!(
                    "{} {}",
                    String::from_utf8_lossy(&status.stdout),
                    String::from_utf8_lossy(&status.stderr)
                )
                .trim()
                .into(),
            });
        }
        let bytes = std::fs::read(out)?;
        let n = case.w * case.h;
        let expected = 848 + 4 * n * if case.options.is_some() { 2 } else { 1 };
        if bytes.len() != expected {
            return Err(format!("oracle output size {}, expected {expected}", bytes.len()).into());
        }
        let mut cursor = 0;
        let mut take = |len| {
            let start = cursor;
            cursor += len;
            &bytes[start..cursor]
        };
        if u32::from_le_bytes(take(4).try_into()?) != 0x464c1737 {
            return Err("oracle magic mismatch".into());
        }
        let start = f32::from_le_bytes(take(4).try_into()?);
        let stop = f32::from_le_bytes(take(4).try_into()?);
        let count = u32::from_le_bytes(take(4).try_into()?) as usize;
        let seconds = f64::from_le_bytes(take(8).try_into()?);
        let stats = (0..6)
            .map(|_| Ok(f32::from_le_bytes(take(4).try_into()?)))
            .collect::<Result<Vec<_>>>()?;
        let counts = (0..100)
            .map(|_| Ok(u64::from_le_bytes(take(8).try_into()?) as usize))
            .collect::<Result<Vec<_>>>()?;
        let pixels = (0..n)
            .map(|_| Ok(f32::from_le_bytes(take(4).try_into()?)))
            .collect::<Result<Vec<_>>>()?;
        let map = ErrorMap::new(case.w, case.h, pixels)?;
        // Validate protocol stats against actual reference pixels before comparing Rust.
        let pooled = map.statistics();
        if stats != stats_array(&map) || counts != pooled.histogram.counts {
            return Err("oracle pooling or histogram disagrees with map".into());
        }
        let exposure = if case.options.is_some() {
            Some(
                (0..n)
                    .map(|_| Ok(f32::from_le_bytes(take(4).try_into()?)))
                    .collect::<Result<Vec<_>>>()?,
            )
        } else {
            None
        };
        Ok(Probe::Output(Output {
            map,
            exposure,
            used: case.options.map(|_| (start, stop, count)),
            seconds,
        }))
    }
}
impl Drop for Oracle {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

pub fn stats_array(map: &ErrorMap) -> Vec<f32> {
    let s = map.statistics();
    vec![
        s.mean,
        s.weighted_median,
        s.first_quartile,
        s.third_quartile,
        s.min,
        s.max,
    ]
}

pub fn compare(r: &Output, c: &Output) -> Result<(f32, f32, f32)> {
    let max_diff = |a: &[f32], b: &[f32]| -> Result<f32> {
        if a.len() != b.len() {
            return Err("length mismatch".into());
        }
        let mut max = 0.0_f32;
        for (&x, &y) in a.iter().zip(b) {
            if !x.is_finite() || !y.is_finite() {
                return Err("nonfinite parity value".into());
            }
            max = max.max((x - y).abs());
        }
        Ok(max)
    };
    let pixels = max_diff(r.map.pixels(), c.map.pixels())?;
    let stats = max_diff(&stats_array(&r.map), &stats_array(&c.map))?;
    let exposure = match (&r.exposure, &c.exposure) {
        (Some(r), Some(c)) => max_diff(r, c)?,
        (None, None) => 0.0,
        _ => return Err("exposure map mismatch".into()),
    };
    if let (Some(r), Some(c)) = (r.used, c.used) {
        if max_diff(&[r.0, r.1], &[c.0, c.1])? > 1e-6 || r.2 != c.2 {
            return Err(format!("HDR parameters mismatch: {r:?} vs {c:?}").into());
        }
    } else if r.used != c.used {
        return Err("HDR parameters missing".into());
    }
    if r.map.statistics().histogram != c.map.statistics().histogram {
        return Err("histogram count mismatch".into());
    }
    if pixels > 1e-5 || stats > 1e-6 || exposure > 1e-6 {
        return Err(format!(
            "parity failure: pixels={pixels:e}, stats={stats:e}, exposure={exposure:e}"
        )
        .into());
    }
    Ok((pixels, stats, exposure))
}

pub fn generated(kind: &str, w: usize, h: usize, hdr: bool) -> (Vec<f32>, Vec<f32>) {
    let mut state = 0x12345678_u32;
    let mut r = Vec::with_capacity(w * h * 3);
    let mut t = Vec::with_capacity(w * h * 3);
    for y in 0..h {
        for x in 0..w {
            for ch in 0..3 {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                let noise = (state >> 8) as f32 / 16777215.0;
                let base = match kind {
                    "zeros" => 0.0,
                    "flat" => 0.3 + 0.1 * ch as f32,
                    "gradient" => (x as f32 + y as f32 + ch as f32) / (w + h + 2) as f32,
                    "edges" => {
                        if x < w / 2 && y < h / 2 {
                            0.1
                        } else {
                            0.9
                        }
                    }
                    "extreme" => [0.0, 1e-20, 1e-6, 1.0, 1e3, 1e10, 1e20, 1e30][(x + y + ch) % 8],
                    "mostly-black" => {
                        if x == 0 && y == 0 {
                            1000.0
                        } else {
                            0.0
                        }
                    }
                    _ => noise,
                };
                let scale = if hdr && !matches!(kind, "extreme" | "mostly-black" | "zeros") {
                    100.0
                } else {
                    1.0
                };
                let a = base * scale;
                let b = if kind == "zeros" {
                    0.0
                } else if hdr {
                    a * (0.8 + 0.4 * noise)
                } else {
                    (a + 0.04 * (noise - 0.5)).clamp(0.0, 1.0)
                };
                r.push(a);
                t.push(b);
            }
        }
    }
    (r, t)
}

pub fn corpus(full: bool) -> Result<Vec<Case>> {
    let mut cases = Vec::new();
    let mut shapes = vec![
        ("zeros", 1, 1),
        ("flat", 1, 1),
        ("gradient", 2, 3),
        ("gradient", 31, 17),
        ("edges", 65, 33),
        ("noise", 127, 93),
    ];
    if full {
        shapes.push(("noise", 1024, 1024));
    }
    for &(kind, w, h) in &shapes {
        for ppd in [20.0, DEFAULT_PPD, 120.0] {
            let (r, t) = generated(kind, w, h, false);
            cases.push(Case {
                name: format!("ldr-{kind}-{w}x{h}-{ppd}"),
                group: "Generated LDR",
                w,
                h,
                r,
                t,
                options: None,
                ppd,
            });
        }
    }
    shapes.extend([("extreme", 17, 19), ("mostly-black", 7, 9)]);
    for &(kind, w, h) in &shapes {
        for ppd in [20.0, DEFAULT_PPD, 120.0] {
            for tonemapper in [Tonemapper::Aces, Tonemapper::Hable, Tonemapper::Reinhard] {
                for auto in [false, true] {
                    // The reference exits on automatic exposure for a black image.
                    if auto && kind == "zeros" {
                        continue;
                    }
                    let (r, t) = generated(kind, w, h, true);
                    let options = hdr_options(
                        ppd,
                        tonemapper,
                        (!auto).then_some(-4.0),
                        (!auto).then_some(4.0),
                        (!auto).then_some(3),
                    );
                    cases.push(Case {
                        name: format!("hdr-{kind}-{w}x{h}-{ppd}-{tonemapper:?}-auto{auto}"),
                        group: "Generated HDR",
                        w,
                        h,
                        r,
                        t,
                        options: Some(options),
                        ppd,
                    });
                }
            }
        }
    }
    // Partial automatic endpoints/counts, equal endpoints, and unequal endpoint counts.
    for (start, stop, count) in [
        (Some(-8.0), None, None),
        (None, Some(8.0), Some(4)),
        (Some(0.0), Some(0.0), Some(2)),
        (Some(-2.0), Some(3.0), None),
    ] {
        let (r, t) = generated("noise", 11, 7, true);
        cases.push(Case {
            name: format!("hdr-partial-{start:?}-{stop:?}-{count:?}"),
            group: "Generated HDR",
            w: 11,
            h: 7,
            r,
            t,
            ppd: DEFAULT_PPD,
            options: Some(hdr_options(
                DEFAULT_PPD,
                Tonemapper::Aces,
                start,
                stop,
                count,
            )),
        });
    }
    #[cfg(feature = "image")]
    if full {
        let root = std::env::var("FLIP_RS_REFERENCE").map_err(|_| {
            "set FLIP_RS_REFERENCE to a clone of https://github.com/NVlabs/flip at b475eb4"
        })?;
        let root = Path::new(&root).join("images");
        let r = flip_rs::io::load_srgb(root.join("reference.png"))?;
        let t = flip_rs::io::load_srgb(root.join("test.png"))?;
        for ppd in [20.0, DEFAULT_PPD, 120.0] {
            cases.push(Case {
                name: format!("reference-png-{ppd}"),
                group: "Reference PNG",
                w: r.width(),
                h: r.height(),
                r: r.pixels().to_vec(),
                t: t.pixels().to_vec(),
                options: None,
                ppd,
            });
        }
        // Teaser is a composite rather than a reference/test pair: identity and perturbation.
        let teaser = flip_rs::io::load_srgb(root.join("teaser.png"))?;
        for perturb in [false, true] {
            cases.push(Case {
                name: format!("teaser-{perturb}"),
                group: "Reference PNG",
                w: teaser.width(),
                h: teaser.height(),
                r: teaser.pixels().to_vec(),
                t: teaser
                    .pixels()
                    .iter()
                    .map(|v| if perturb { (v + 0.01).min(1.0) } else { *v })
                    .collect(),
                options: None,
                ppd: DEFAULT_PPD,
            });
        }
        let r = flip_rs::io::load_linear(root.join("reference.exr"))?;
        let t = flip_rs::io::load_linear(root.join("test.exr"))?;
        for ppd in [20.0, DEFAULT_PPD, 120.0] {
            for tonemapper in [Tonemapper::Aces, Tonemapper::Hable, Tonemapper::Reinhard] {
                for auto in [false, true] {
                    cases.push(Case {
                        name: format!("reference-exr-{ppd}-{tonemapper:?}-auto{auto}"),
                        group: "Reference EXR",
                        w: r.width(),
                        h: r.height(),
                        r: r.pixels().to_vec(),
                        t: t.pixels().to_vec(),
                        ppd,
                        options: Some(hdr_options(
                            ppd,
                            tonemapper,
                            (!auto).then_some(-12.0),
                            (!auto).then_some(1.0),
                            (!auto).then_some(5),
                        )),
                    });
                }
            }
        }
    }
    Ok(cases)
}
