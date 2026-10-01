#[path = "../examples/support/mod.rs"]
#[allow(dead_code)] // Timing/report fields are used by the benchmark example.
mod support;

#[path = "../examples/sweep/mod.rs"]
#[allow(dead_code)]
mod sweep;

#[test]
fn hdr_near_ties_preserve_reference_exposure_indices() -> support::Result<()> {
    let Ok(bin) = std::env::var("FLIP_RS_PARITY_BIN") else {
        eprintln!("SKIP C++ near-tie parity: set FLIP_RS_PARITY_BIN");
        return Ok(());
    };
    let oracle = support::Oracle::new(bin)?;
    for seed in [15158193341402106541, 12092506619092908091] {
        let case = sweep::case(seed);
        support::compare(&case.rust()?, &oracle.cpp(&case, 1)?)
            .map_err(|e| format!("near-tie seed {seed}: {e}"))?;
    }
    Ok(())
}

#[test]
fn cpp_reference_parity() -> support::Result<()> {
    let Ok(bin) = std::env::var("FLIP_RS_PARITY_BIN") else {
        eprintln!("SKIP C++ parity: set FLIP_RS_PARITY_BIN to the compiled oracle");
        return Ok(());
    };
    let oracle = support::Oracle::new(bin)?;
    for case in support::corpus(false)? {
        let rust = case.rust().map_err(|e| format!("{}: {e}", case.name))?;
        let cpp = oracle.cpp(&case, 1)?;
        support::compare(&rust, &cpp).map_err(|e| format!("{}: {e}", case.name))?;
    }
    Ok(())
}

#[test]
fn cpp_reference_tile_boundaries() -> support::Result<()> {
    let Ok(bin) = std::env::var("FLIP_RS_PARITY_BIN") else {
        eprintln!("SKIP C++ tile parity: set FLIP_RS_PARITY_BIN to the compiled oracle");
        return Ok(());
    };
    let oracle = support::Oracle::new(bin)?;
    // Narrower than the padding, one full tile, and partial tiles in both passes.
    for (w, h) in [(1, 19), (128, 5), (129, 7), (257, 3)] {
        for ppd in [20.0, flip_rs::DEFAULT_PPD, 120.0] {
            for hdr in [false, true] {
                let (r, t) = support::generated("noise", w, h, hdr);
                let case = support::Case {
                    name: format!("tile-{w}x{h}-{ppd}-hdr{hdr}"),
                    group: "Tile boundaries",
                    w,
                    h,
                    r,
                    t,
                    ppd,
                    options: hdr.then(|| {
                        support::hdr_options(
                            ppd,
                            flip_rs::Tonemapper::Aces,
                            Some(-4.0),
                            Some(4.0),
                            Some(3),
                        )
                    }),
                };
                let rust = case.rust().map_err(|e| format!("{}: {e}", case.name))?;
                let cpp = oracle.cpp(&case, 1)?;
                support::compare(&rust, &cpp).map_err(|e| format!("{}: {e}", case.name))?;
            }
        }
    }
    Ok(())
}
