mod support;
mod sweep;
use flip_rs::{Tonemapper, DEFAULT_PPD};
use support::{compare, corpus, generated, hdr_options, Case, Oracle, Result};

fn main() -> Result<()> {
    if std::env::args().any(|s| s == "--sweep") {
        return sweep::run();
    }
    let bin = std::env::var("FLIP_RS_PARITY_BIN")?;
    let oracle = Oracle::new(bin)?;
    let benchmark = std::env::args().any(|s| s == "--bench");
    let out = std::env::var("FLIP_RS_REPORT").unwrap_or_else(|_| {
        if benchmark {
            "parity/build/benchmarks.md"
        } else {
            "parity/build/parity.md"
        }
        .into()
    });
    let mut report = String::new();
    if benchmark {
        report.push_str(
            "| Case | C++ -O2 single thread (s) | Rust (s) | C++ / Rust |\n|---|---:|---:|---:|\n",
        );
        for (w, h) in [(1920, 1080), (3840, 2160)] {
            for hdr in [false, true] {
                let (r, t) = generated("noise", w, h, hdr);
                let options = hdr.then(|| {
                    hdr_options(
                        DEFAULT_PPD,
                        Tonemapper::Aces,
                        Some(-4.0),
                        Some(4.0),
                        Some(3),
                    )
                });
                let case = Case {
                    name: format!(
                        "{} {w}×{h}",
                        if hdr { "HDR ACES, 3 exposures" } else { "LDR" }
                    ),
                    group: "Bench",
                    w,
                    h,
                    r,
                    t,
                    options,
                    ppd: DEFAULT_PPD,
                };
                let c = oracle.cpp(&case, 3)?;
                case.rust()?; // warmup
                let mut times = Vec::new();
                for _ in 0..3 {
                    let r = case.rust()?;
                    compare(&r, &c)?;
                    times.push(r.seconds);
                }
                times.sort_by(f64::total_cmp);
                let row = format!(
                    "| {} | {:.6} | {:.6} | {:.2}× |\n",
                    case.name,
                    c.seconds,
                    times[1],
                    c.seconds / times[1]
                );
                print!("{row}");
                report.push_str(&row);
            }
        }
    } else {
        let mut groups = std::collections::BTreeMap::<&str, (usize, f32, f32, f32)>::new();
        for case in corpus(true)? {
            let r = case.rust()?;
            let c = oracle.cpp(&case, 1)?;
            let (pixels, stats, exposure) =
                compare(&r, &c).map_err(|e| format!("{}: {e}", case.name))?;
            let group = groups.entry(case.group).or_default();
            group.0 += 1;
            group.1 = group.1.max(pixels);
            group.2 = group.2.max(stats);
            group.3 = group.3.max(exposure);
            println!(
                "PASS {} pixel={pixels:e} pooled={stats:e} exposure={exposure:e}",
                case.name
            );
        }
        report.push_str("| Corpus | Cases | Max pixel difference | Max pooled difference | Max exposure-map difference |\n|---|---:|---:|---:|---:|\n");
        for (name, (n, p, s, e)) in groups {
            report.push_str(&format!("| {name} | {n} | {p:.9e} | {s:.9e} | {e:.9e} |\n"));
        }
        print!("{report}");
    }
    std::fs::write(out, report)?;
    Ok(())
}
