//! Compares two images and writes a FLIP heatmap.
//!
//! ```sh
//! cargo run --release --features image --example compare -- reference.png test.png heatmap.png
//! cargo run --release --features image --example compare -- reference.exr test.exr heatmap.png 67.02 hable
//! ```
//!
//! `.exr` inputs are evaluated with HDR-FLIP, anything else with LDR-FLIP.

use flip_rs::{hdr_flip, io, ldr_flip, HdrOptions, Tonemapper, DEFAULT_PPD};
use std::path::Path;

const USAGE: &str = "usage: compare REFERENCE TEST HEATMAP.png [PPD] [aces|hable|reinhard]";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [reference, test, heatmap, rest @ ..] = &args[..] else {
        return Err(USAGE.into());
    };
    let ppd = match rest.first() {
        Some(ppd) => ppd.parse()?,
        None => DEFAULT_PPD,
    };
    let hdr = Path::new(reference)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("exr"));
    let map = if hdr {
        let mut options = HdrOptions::default();
        options.ppd = ppd;
        options.tonemapper = match rest.get(1).map_or("aces", String::as_str) {
            "aces" => Tonemapper::Aces,
            "hable" => Tonemapper::Hable,
            "reinhard" => Tonemapper::Reinhard,
            other => return Err(format!("unknown tone mapper {other:?}\n{USAGE}").into()),
        };
        let result = hdr_flip(
            &io::load_linear(reference)?,
            &io::load_linear(test)?,
            options,
        )?;
        println!("{:?}", result.parameters);
        result.error_map
    } else {
        ldr_flip(&io::load_srgb(reference)?, &io::load_srgb(test)?, ppd)?
    };
    let s = map.statistics();
    println!(
        "mean {:.6}, weighted median {:.6}, quartiles {:.6}..{:.6}, range {:.6}..{:.6}",
        s.mean, s.weighted_median, s.first_quartile, s.third_quartile, s.min, s.max
    );
    io::save_heatmap(&map, heatmap)?;
    Ok(())
}
