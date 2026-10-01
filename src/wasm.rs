use crate::{dimensions, ldr_flip, FlipError, RgbImage};
use wasm_bindgen::prelude::*;

/// LDR-FLIP for the browser, exported to JavaScript as
/// `ldrFlip(reference, test, width, height, ppd)`.
///
/// `reference` and `test` are RGBA bytes, such as `ImageData.data`; alpha is
/// ignored. Returns one error per pixel as a `Float32Array`, or throws an
/// `Error` for invalid input. Requires the `wasm` feature.
#[wasm_bindgen(js_name = ldrFlip)]
pub fn ldr_flip_rgba(
    reference: &[u8],
    test: &[u8],
    width: u32,
    height: u32,
    ppd: f32,
) -> Result<js_sys::Float32Array, js_sys::Error> {
    let to_rgb = |rgba: &[u8]| -> Result<RgbImage<u8>, FlipError> {
        let (w, h) = (width as usize, height as usize);
        if rgba.len() != dimensions(w, h, 4)? {
            return Err(FlipError::SizeMismatch);
        }
        let rgb = rgba
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2]]);
        RgbImage::new(w, h, rgb.collect())
    };
    let map = to_rgb(reference)
        .and_then(|r| ldr_flip(&r, &to_rgb(test)?, ppd))
        .map_err(|e| js_sys::Error::new(&e.to_string()))?;
    Ok(js_sys::Float32Array::from(map.pixels()))
}
