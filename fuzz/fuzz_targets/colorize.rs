#![no_main]
mod common;
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    if let Some(map) = common::map(data) {
        if let Ok(rgb) = map.colorize() {
            assert_eq!(rgb.pixels().len(), 3 * map.pixels().len());
            assert!(rgb.pixels().iter().all(|v| (0.0..=1.0).contains(v)));
        }
    }
});
