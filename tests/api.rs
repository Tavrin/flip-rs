use flip_rs::{
    hdr_flip, ldr_flip, pixels_per_degree, ErrorMap, FlipError, HdrOptions, RgbImage, Tonemapper,
    DEFAULT_PPD, MAGMA,
};

fn rgb(width: usize, height: usize, pixels: &[f32]) -> RgbImage<f32> {
    RgbImage::new(width, height, pixels.to_vec()).unwrap()
}

fn explicit(tonemapper: Tonemapper, start: f32, stop: f32, count: usize) -> HdrOptions {
    let mut options = HdrOptions::default();
    options.start_exposure = Some(start);
    options.stop_exposure = Some(stop);
    options.num_exposures = Some(count);
    options.tonemapper = tonemapper;
    options
}

#[test]
fn layout_and_parameters_are_checked() {
    assert!(matches!(
        RgbImage::<u8>::new(0, 1, vec![]),
        Err(FlipError::InvalidDimensions)
    ));
    assert!(matches!(
        RgbImage::<u8>::new(usize::MAX, 2, vec![]),
        Err(FlipError::InvalidDimensions)
    ));
    assert!(matches!(
        RgbImage::<u8>::new(2, 3, vec![0; 17]),
        Err(FlipError::SizeMismatch)
    ));
    let a = RgbImage::new(1, 1, vec![0.0; 3]).unwrap();
    let b = RgbImage::new(2, 1, vec![0.0; 6]).unwrap();
    assert!(matches!(
        ldr_flip(&a, &b, DEFAULT_PPD),
        Err(FlipError::SizeMismatch)
    ));
    for ppd in [0.0, -1.0, f32::INFINITY, f32::NAN, 1e-30] {
        assert!(ldr_flip(&a, &a, ppd).is_err());
    }
    let nan = RgbImage::new(1, 1, vec![f32::NAN; 3]).unwrap();
    assert!(matches!(
        ldr_flip(&nan, &a, DEFAULT_PPD),
        Err(FlipError::NonFiniteInput)
    ));
    assert!(matches!(
        hdr_flip(
            &rgb(1, 1, &[1.0; 3]),
            &rgb(3, 1, &[1.0; 9]),
            HdrOptions::default()
        ),
        Err(FlipError::SizeMismatch)
    ));
    assert!(ErrorMap::new(1, 1, vec![-0.1]).is_err());
    assert!(ErrorMap::new(1, 1, vec![f32::INFINITY]).is_err());
    assert_eq!(pixels_per_degree(0.7, 3840.0, 0.7).unwrap(), DEFAULT_PPD);
    assert!((DEFAULT_PPD - 67.020645).abs() < 1e-5);
    assert!(pixels_per_degree(0.7, 3840.0, 0.0).is_err());
}

#[test]
fn bytes_floats_clamping_and_identity() {
    let pixels = vec![0_u8, 128, 255, 43, 18, 229];
    let byte = RgbImage::new(2, 1, pixels.clone()).unwrap();
    let float = RgbImage::new(
        2,
        1,
        pixels.into_iter().map(|v| f32::from(v) / 255.0).collect(),
    )
    .unwrap();
    let changed = RgbImage::new(2, 1, vec![0.1_f32; 6]).unwrap();
    assert_eq!(
        ldr_flip(&byte, &changed, DEFAULT_PPD).unwrap().pixels(),
        ldr_flip(&float, &changed, DEFAULT_PPD).unwrap().pixels()
    );
    assert_eq!(
        ldr_flip(&byte, &float, DEFAULT_PPD).unwrap().pixels(),
        &[0.0, 0.0]
    );
    let outside = RgbImage::new(1, 1, vec![-10.0, 2.0, 0.5]).unwrap();
    let clamped = RgbImage::new(1, 1, vec![0.0, 1.0, 0.5]).unwrap();
    assert_eq!(
        ldr_flip(&outside, &clamped, DEFAULT_PPD).unwrap().pixels(),
        &[0.0]
    );
}

#[test]
fn reference_pooling_and_magma_rounding() {
    let map = ErrorMap::new(5, 1, vec![0.0, 0.1, 0.2, 0.3, 0.4]).unwrap();
    let s = map.statistics();
    assert_eq!(s.mean, 0.2);
    assert_eq!(
        (s.first_quartile, s.weighted_median, s.third_quartile),
        (0.2, 0.3, 0.4)
    );
    assert_eq!(s.histogram.counts.iter().sum::<usize>(), 5);
    let zero = ErrorMap::new(1, 1, vec![0.0]).unwrap();
    assert_eq!(zero.statistics().max, f32::MIN_POSITIVE); // Upstream quirk.
    assert_eq!(zero.statistics().weighted_median, 0.0);
    assert_eq!(zero.colorize().pixels(), &MAGMA[0]);
    let map = ErrorMap::new(3, 1, vec![0.5, 1.0, 2.0]).unwrap();
    assert_eq!(
        map.colorize().pixels(),
        [MAGMA[128], MAGMA[255], MAGMA[255]].concat()
    );
    assert_eq!(map.statistics().histogram.counts[99], 1);
    assert_eq!(map.statistics().histogram.out_of_range, 1);
}

#[test]
fn hdr_defined_black_inputs_and_ties() {
    let black = rgb(1, 1, &[0.0; 3]);
    assert!(hdr_flip(&black, &black, HdrOptions::default()).is_err());
    let (dark, light) = (rgb(1, 1, &[0.1; 3]), rgb(1, 1, &[0.2; 3]));
    for tonemapper in [Tonemapper::Aces, Tonemapper::Hable, Tonemapper::Reinhard] {
        let options = explicit(tonemapper, -4.0, 4.0, 3);
        let black = rgb(2, 3, &[0.0; 18]);
        let result = hdr_flip(&black, &black, options).unwrap();
        assert_eq!(result.error_map.pixels(), &[0.0; 6]);
        assert_eq!(result.exposure_map.unwrap().pixels(), &[0.0; 6]);

        let mut no_map = options;
        no_map.return_exposure_map = false;
        assert!(hdr_flip(&dark, &light, no_map)
            .unwrap()
            .exposure_map
            .is_none());

        let equal = hdr_flip(&dark, &light, explicit(tonemapper, 0.0, 0.0, 3)).unwrap();
        assert_eq!(equal.exposure_map.unwrap().pixels(), &[0.0]);

        assert!(hdr_flip(&dark, &light, explicit(tonemapper, -4.0, 4.0, 1)).is_err());
        assert!(hdr_flip(&dark, &light, explicit(tonemapper, 10.0, 4.0, 3)).is_err());
    }
}

#[test]
fn hdr_reused_workspace_matches_independent_exposures() {
    let (w, h) = (129, 3);
    let reference: Vec<_> = (0..w * h * 3).map(|i| (i % 127) as f32 * 0.125).collect();
    let test: Vec<_> = reference
        .iter()
        .enumerate()
        .map(|(i, v)| v * (0.75 + (i % 17) as f32 / 32.0))
        .collect();
    let (reference, test) = (rgb(w, h, &reference), rgb(w, h, &test));
    for tonemapper in [Tonemapper::Aces, Tonemapper::Hable, Tonemapper::Reinhard] {
        let combined = hdr_flip(&reference, &test, explicit(tonemapper, -4.0, 4.0, 3)).unwrap();
        let mut errors = vec![0.0; w * h];
        let mut exposures = vec![0.0; w * h];
        for (index, exposure) in [-4.0, 0.0, 4.0].into_iter().enumerate() {
            let mut options = explicit(tonemapper, exposure, exposure, 2);
            options.return_exposure_map = false;
            let isolated = hdr_flip(&reference, &test, options).unwrap();
            for (i, &error) in isolated.error_map.pixels().iter().enumerate() {
                if error > errors[i] {
                    errors[i] = error;
                    exposures[i] = index as f32 / 2.0;
                }
            }
        }
        assert_eq!(combined.error_map.pixels(), errors);
        assert_eq!(combined.exposure_map.unwrap().pixels(), exposures);
    }
}

#[cfg(feature = "image")]
#[test]
fn png_and_exr_helpers_round_trip() {
    let dir = std::env::temp_dir().join(format!("flip-image-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let rgb = RgbImage::new(2, 1, vec![0.0, 0.5, 1.0, 0.1, 10.0, 1000.0]).unwrap();
    flip_rs::io::save_rgb(&rgb, dir.join("test.exr")).unwrap();
    assert_eq!(
        flip_rs::io::load_linear(dir.join("test.exr"))
            .unwrap()
            .pixels(),
        rgb.pixels()
    );
    let map = ErrorMap::new(2, 1, vec![0.0, 1.0]).unwrap();
    flip_rs::io::save_heatmap(&map, dir.join("test.png")).unwrap();
    let loaded = flip_rs::io::load_srgb(dir.join("test.png")).unwrap();
    assert_eq!((loaded.width(), loaded.height()), (2, 1));
    for (&a, &b) in loaded.pixels().iter().zip(map.colorize().pixels()) {
        assert!((a - b).abs() <= 0.5 / 255.0 + f32::EPSILON);
    }
    std::fs::remove_dir_all(dir).unwrap();
}
