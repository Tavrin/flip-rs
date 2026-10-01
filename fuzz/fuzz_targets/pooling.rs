#![no_main]
mod common;
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    if let Some(map) = common::map(data) {
        let s = map.statistics();
        assert_eq!(
            s.histogram.counts.iter().sum::<usize>() + s.histogram.out_of_range,
            map.pixels().len()
        );
        if s.finite {
            assert!(s.mean.is_finite() && s.weighted_median.is_finite());
        }
    }
});
