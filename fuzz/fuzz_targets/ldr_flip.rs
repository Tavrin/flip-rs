#![no_main]
mod common;
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    if let (Some(r), Some(t)) = (common::image(data, 16), common::image(data, 784)) {
        if let Ok(map) = flip_rs::ldr_flip(&r, &t, common::float(data, 12)) {
            assert_eq!(map.pixels().len(), r.width() * r.height());
            assert!(map.pixels().iter().all(|v| v.is_finite() && *v >= 0.0));
        }
    }
});
