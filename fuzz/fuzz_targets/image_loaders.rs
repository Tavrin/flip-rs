#![no_main]
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    // One process per target; a fixed PID-scoped file avoids per-input directory
    // churn while exercising the public path-based loaders and format checks.
    let path = std::env::temp_dir().join(format!("flip-fuzz-loader-{}.bin", std::process::id()));
    if std::fs::write(&path, data).is_ok() {
        let _ = flip_rs::io::load_srgb(&path);
        let _ = flip_rs::io::load_linear(&path);
        let _ = std::fs::remove_file(path);
    }
});
