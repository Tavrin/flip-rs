use crate::support::{compare, hdr_options, Case, Oracle, Probe, Result};
use flip_rs::Tonemapper;

// SplitMix64: each case has its own seed, independent of earlier case sizes.
struct Random(u64);
impl Random {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
    fn pick(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / 16_777_215.0
    }
}

pub(crate) fn case(seed: u64) -> Case {
    let mut rng = Random(seed);
    let (w, h) = match rng.pick(20) {
        0 => (1, 1 + rng.pick(512)),
        1 => (1 + rng.pick(512), 1),
        2 => {
            let primes = [2, 3, 7, 17, 31, 127, 257, 509];
            (primes[rng.pick(8)], primes[rng.pick(8)])
        }
        3 => (1 + rng.pick(512), 1 + rng.pick(512)),
        4..=6 => (1 + rng.pick(128), 1 + rng.pick(128)),
        _ => (1 + rng.pick(32), 1 + rng.pick(32)),
    };
    let hdr = rng.pick(2) == 0;
    let kind = rng.pick(7);
    let scale = if hdr {
        10.0_f32.powf(-4.0 + 8.0 * rng.unit())
    } else {
        1.0
    };
    let ppd = 1.0 + 199.0 * rng.unit();
    let constant = rng.unit();
    let mut r = Vec::with_capacity(w * h * 3);
    let mut t = Vec::with_capacity(w * h * 3);
    for y in 0..h {
        for x in 0..w {
            for ch in 0..3 {
                let noise = rng.unit();
                let v = match kind {
                    0 => noise,
                    1 => (x + y + ch) as f32 / (w + h + 2) as f32,
                    2 => {
                        if x < w / 2 {
                            0.0
                        } else {
                            1.0
                        }
                    }
                    3 => constant,
                    4 => noise * 1e-4,
                    5 => {
                        if ch == (x + y) % 3 {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    _ => 10.0_f32.powf(-4.0 + 8.0 * noise),
                };
                let a = v * scale;
                r.push(a);
                t.push(if hdr {
                    a * (0.5 + rng.unit())
                } else {
                    (a + (rng.unit() - 0.5) * 0.1).clamp(0.0, 1.0)
                });
            }
        }
    }
    let options = hdr.then(|| {
        let tm = [Tonemapper::Aces, Tonemapper::Hable, Tonemapper::Reinhard][rng.pick(3)];
        let start = -16.0 + 16.0 * rng.unit();
        let stop = start + 16.0 * rng.unit();
        let count = if w * h <= 16 && rng.pick(8) == 0 {
            64
        } else {
            2 + rng.pick(7)
        };
        let mode = rng.pick(8);
        let (start, stop, count) = match mode {
            0 => (None, None, None),
            1 => (None, None, Some(count)),
            2 => (Some(-40.0), None, Some(count)),
            3 => (None, Some(40.0), Some(count)),
            4 => (Some(start), Some(start), Some(count)),
            5 => (Some(start), Some(stop), None),
            _ => (Some(start), Some(stop), Some(count)),
        };
        hdr_options(ppd, tm, start, stop, count)
    });
    Case {
        name: format!("seed-{seed}-{w}x{h}-hdr{hdr}-kind{kind}"),
        group: "Sweep",
        w,
        h,
        r,
        t,
        options,
        ppd,
    }
}

fn degenerate(seed: u64, which: usize) -> (Case, &'static str) {
    let mut c = case(seed);
    c.w = 1;
    c.h = 1;
    c.r = vec![0.1; 3];
    c.t = vec![0.2; 3];
    c.ppd = 67.020645;
    c.options = Some(hdr_options(
        c.ppd,
        Tonemapper::Aces,
        Some(0.0),
        Some(0.0),
        Some(2),
    ));
    let reason = match which {
        0 => {
            c.ppd = 1.0;
            c.options = None;
            "zero feature-filter normalizers divide by zero; NaN intermediates may be masked"
        }
        1 => {
            c.r.fill(0.0);
            c.options = Some(hdr_options(c.ppd, Tonemapper::Aces, None, None, None));
            "black auto reference has infinite start and reversed endpoints"
        }
        2 => {
            c.r.fill(1e-20);
            c.options = Some(hdr_options(c.ppd, Tonemapper::Hable, None, None, None));
            "epsilon-clamped median makes auto endpoints reversed"
        }
        3 => {
            c.options = Some(hdr_options(
                c.ppd,
                Tonemapper::Reinhard,
                Some(4.0),
                Some(-4.0),
                Some(2),
            ));
            "reversed endpoints: upstream exits"
        }
        4 => {
            c.options = Some(hdr_options(
                c.ppd,
                Tonemapper::Aces,
                Some(0.0),
                Some(0.0),
                Some(1),
            ));
            "one exposure divides by zero; NaN intermediate values are hidden by max selection"
        }
        _ => {
            c.options = Some(hdr_options(
                c.ppd,
                Tonemapper::Aces,
                Some(0.0),
                Some(0.0),
                Some(0),
            ));
            "zero exposures evaluate no image; not a defined HDR comparison"
        }
    };
    (c, reason)
}

pub fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let value = |key: &str| args.windows(2).find(|a| a[0] == key).map(|a| a[1].as_str());
    let count: usize = value("--count").unwrap_or("3000").parse()?;
    if count == 0 {
        return Err("sweep count must be positive".into());
    }
    let seed: u64 = value("--seed").unwrap_or("3572951").parse()?;
    let isolated = value("--case-seed").map(str::parse::<u64>).transpose()?;
    let oracle = Oracle::new(std::env::var("FLIP_RS_PARITY_BIN")?)?;
    let mut random = Random(seed);
    let mut maxima = [(0.0_f32, 0_u64); 3];
    let mut passed = 0;
    let mut failures = Vec::new();
    let mut undefined = Vec::new();
    let mut coverage = [0_usize; 3];
    let mut shape_max = (0, 0);
    for index in 0..count {
        let case_seed = isolated.unwrap_or_else(|| random.next());
        // Dedicated probes are part of the count. --case-seed generates the
        // ordinary case directly, so random failures can be replayed alone.
        let (c, reason) = if isolated.is_none() && index < 6 {
            let (c, reason) = degenerate(case_seed, index);
            (c, Some(reason))
        } else {
            (case(case_seed), None)
        };
        coverage[usize::from(c.options.is_some())] += 1;
        if c.w == 1 || c.h == 1 {
            coverage[2] += 1;
        }
        shape_max.0 = shape_max.0.max(c.w);
        shape_max.1 = shape_max.1.max(c.h);
        let rust = c.rust();
        // Always execute the reference, even when Rust rejects the input.
        let cpp = oracle.probe(&c, 1)?;
        match (rust, cpp) {
            (Ok(r), Probe::Output(cpp)) if reason.is_none() => match compare(&r, &cpp) {
                Ok((p, s, e)) => {
                    passed += 1;
                    for (dst, v) in maxima.iter_mut().zip([p, s, e]) {
                        if v > dst.0 {
                            *dst = (v, case_seed);
                        }
                    }
                }
                Err(e) => failures.push(format!(
                    "{}: {e}; PPD={}, options={:?}",
                    c.name, c.ppd, c.options
                )),
            },
            (Err(e), probe) => {
                let error = e.to_string();
                let inferred = if error == "PPD produces degenerate reference filters" {
                    Some("feature-filter normalizers underflow to zero")
                } else if error.starts_with("exposure endpoints must be finite and ordered")
                    && matches!(
                        &probe,
                        Probe::Rejected {
                            code: Some(255),
                            ..
                        }
                    )
                {
                    Some("automatic exposure endpoints are nonfinite or reversed; upstream exits")
                } else {
                    None
                };
                if let Some(reason) = reason.or(inferred) {
                    let behavior = match probe {
                        Probe::Output(o) => format!(
                            "exit 0, max error {:.9e}, resolved {:?}",
                            o.map.statistics().max,
                            o.used
                        ),
                        Probe::Rejected { code, diagnostic } => {
                            format!("exit {code:?}: {diagnostic}")
                        }
                    };
                    undefined.push(format!(
                        "| {index} | {case_seed} | {reason} | {error} | {behavior} |"
                    ));
                } else {
                    failures.push(format!("{}: unexpected Rust rejection: {error}", c.name));
                }
            }
            (Ok(_), Probe::Rejected { code, diagnostic }) => {
                failures.push(format!("{}: C++ exit {code:?}: {diagnostic}", c.name))
            }
            (Ok(_), Probe::Output(_)) => failures.push(format!(
                "{}: degenerate input unexpectedly accepted by Rust",
                c.name
            )),
        }
        if (index + 1) % 100 == 0 {
            println!(
                "sweep {}/{count}: {passed} parity, {} undefined, {} failures",
                index + 1,
                undefined.len(),
                failures.len()
            );
        }
    }
    let mut report = format!("# Randomized parity sweep\n\nSeed: `{seed}` (SplitMix64); cases: **{count}**; parity passes: **{passed}**; documented undefined cases: **{}**; failures: **{}**.\n\nLDR: {}; HDR: {}; one-dimensional: {}; maximum width/height: {} / {}.\n\nReference: `$FLIP_RS_REFERENCE`, pinned `b475eb4bf394ab877c42166c9eb0a84a02cc5b14`; clean checkout required. Inputs are shared raw f32 buffers. Gates: pixels <= 1e-5, pooled/exposure/endpoints <= 1e-6, exact histograms and exposure counts.\n\n| Difference | Maximum | Case seed |\n|---|---:|---:|\n", undefined.len(), failures.len(), coverage[0], coverage[1], coverage[2], shape_max.0, shape_max.1);
    for (name, (v, seed)) in ["Pixel", "Pooled", "Exposure map"].into_iter().zip(maxima) {
        report.push_str(&format!("| {name} | {v:.9e} | {seed} |\n"));
    }
    report.push_str("\nReplay a random case with `./parity/sweep.sh --count 1 --case-seed SEED`; replay dedicated degeneracies with the full seed and count >= 6.\n\n## Undefined reference inputs\n\nC++ exit 3 is a driver guard after the actual reference calculation found nonfinite pixels, before undefined float-to-integer histogram conversion. C++ exit 255 is the upstream exit(-1). Finite zero maps do not make a division by zero or zero-exposure comparison defined.\n\n| Index | Case seed | Justification | Rust error | Observed C++ behavior |\n|---|---:|---|---|---|\n");
    for row in undefined {
        report.push_str(&row);
        report.push('\n');
    }
    report.push_str("\n## Outliers\n\n");
    if failures.is_empty() {
        report.push_str(
            "None above the gates. The maximum-difference case seeds are recorded above.\n",
        );
    }
    for failure in &failures {
        report.push_str(&format!("- {failure}\n"));
    }
    let out = std::env::var("FLIP_RS_REPORT").unwrap_or_else(|_| "parity/results/sweep.md".into());
    std::fs::write(out, report)?;
    if !failures.is_empty() {
        return Err(format!("{} sweep failures; see sweep report", failures.len()).into());
    }
    Ok(())
}
