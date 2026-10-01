#![no_main]
mod common;
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    if let (Some(r), Some(t)) = (common::image(data, 32), common::image(data, 800)) {
        let mut options = flip_rs::HdrOptions::default();
        options.ppd = common::float(data, 12);
        options.start_exposure = (common::word(data, 0) & 8 != 0).then(|| common::float(data, 16));
        options.stop_exposure = (common::word(data, 0) & 16 != 0).then(|| common::float(data, 20));
        options.num_exposures =
            (common::word(data, 0) & 32 != 0).then(|| common::word(data, 24) as usize);
        options.tonemapper = match common::word(data, 28) % 3 {
            0 => flip_rs::Tonemapper::Aces,
            1 => flip_rs::Tonemapper::Hable,
            _ => flip_rs::Tonemapper::Reinhard,
        };
        if let Ok(result) = flip_rs::hdr_flip(&r, &t, options) {
            assert!((2..=128).contains(&result.parameters.num_exposures));
            assert!(result
                .error_map
                .pixels()
                .iter()
                .all(|v| v.is_finite() && *v >= 0.0));
        }
    }
});
