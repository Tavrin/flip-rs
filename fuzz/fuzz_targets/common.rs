#![allow(dead_code)]
use flip_rs::{ErrorMap, RgbImage};

pub fn word(data: &[u8], offset: usize) -> u32 {
    let mut bytes = [0; 4];
    for (i, dst) in bytes.iter_mut().enumerate() {
        *dst = data.get(offset + i).copied().unwrap_or(0);
    }
    u32::from_le_bytes(bytes)
}
pub fn float(data: &[u8], offset: usize) -> f32 {
    f32::from_bits(word(data, offset))
}
pub fn dimensions(data: &[u8]) -> (usize, usize) {
    match word(data, 0) % 8 {
        0 => (usize::MAX, 2),
        1 => (178_956_971, 1), // RGB exceeds wasm32 isize byte limit.
        2 => (0, 0),
        _ => (
            1 + word(data, 4) as usize % 8,
            1 + word(data, 8) as usize % 8,
        ),
    }
}
pub fn image(data: &[u8], offset: usize) -> Option<RgbImage<f32>> {
    let (w, h) = dimensions(data);
    let len = if w <= 8 && h <= 8 { w * h * 3 } else { 0 };
    let pixels = (0..len).map(|i| float(data, offset + i * 4)).collect();
    RgbImage::new(w, h, pixels).ok()
}
pub fn map(data: &[u8]) -> Option<ErrorMap> {
    let (w, h) = dimensions(data);
    let len = if w <= 8 && h <= 8 { w * h } else { 0 };
    ErrorMap::new(w, h, (0..len).map(|i| float(data, 12 + i * 4)).collect()).ok()
}
