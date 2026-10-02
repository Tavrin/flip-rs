use flip_rs::{
    hdr_flip, ldr_flip, pixels_per_degree, ErrorMap, FlipError, HdrOptions, RgbImage, Tonemapper,
    Weighting::{Unweighted, Weighted},
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
    assert_eq!(zero.colorize().unwrap().pixels(), &MAGMA[0]);
    let map = ErrorMap::new(3, 1, vec![0.5, 1.0, 2.0]).unwrap();
    assert_eq!(
        map.colorize().unwrap().pixels(),
        [MAGMA[128], MAGMA[255], MAGMA[255]].concat()
    );
    assert_eq!(map.statistics().histogram.counts[99], 1);
    assert_eq!(map.statistics().histogram.out_of_range, 1);
}

#[test]
fn arbitrary_percentiles_keep_reference_index_and_strict_weighted_threshold() {
    let map = ErrorMap::new(5, 1, vec![0.4, 0.0, 0.3, 0.1, 0.2]).unwrap();
    let pooled = map.percentiles().unwrap();
    for (p, weighted, unweighted) in [
        (0.0, 0.1_f32, 0.0_f32),
        (0.01, 0.1, 0.1),
        (0.25, 0.2, 0.2),
        (0.5, 0.3, 0.3),
        (0.75, 0.4, 0.4),
        (0.8, 0.4, 0.4),
    ] {
        for (weighting, expected) in [(Weighted, weighted), (Unweighted, unweighted)] {
            assert_eq!(
                pooled.percentile(p, weighting).unwrap().to_bits(),
                expected.to_bits()
            );
            assert_eq!(
                map.percentile(p, weighting).unwrap().to_bits(),
                expected.to_bits()
            );
        }
    }
    assert_eq!(pooled.percentile(0.99, Weighted).unwrap(), 0.4);
    assert_eq!(pooled.percentile(1.0, Weighted).unwrap(), 0.0);
    let tied = ErrorMap::new(4, 1, vec![0.0, 0.25, 0.25, 0.5]).unwrap();
    assert_eq!(tied.percentile(0.5, Weighted).unwrap(), 0.5);
    assert_eq!(map.pixels(), &[0.4, 0.0, 0.3, 0.1, 0.2]);
    let zero = ErrorMap::new(1, 1, vec![0.0]).unwrap();
    for p in [0.0, 0.99, 1.0] {
        assert_eq!(
            zero.percentile(p, Weighted).unwrap().to_bits(),
            0.0_f32.to_bits()
        );
    }
    assert_eq!(zero.percentile(0.0, Unweighted).unwrap(), 0.0);
    // The total must keep row-major rounding, rather than be re-summed sorted.
    let rounded = ErrorMap::new(3, 1, vec![1e8, 4.0, 4.0]).unwrap();
    assert_eq!(rounded.percentile(1.0, Weighted).unwrap(), 1e8);
    let overflow = ErrorMap::new(2, 1, vec![f32::MAX; 2]).unwrap();
    for p in [0.0, 0.5, 1.0] {
        assert_eq!(overflow.percentile(p, Weighted).unwrap(), 0.0);
    }
}

#[test]
fn percentile_queries_reject_invalid_fractions_and_undefined_indices() {
    let map = ErrorMap::new(3, 1, vec![0.1, 0.2, 0.3]).unwrap();
    let pooled = map.percentiles().unwrap();
    for p in [-0.1, 1.01, f32::NAN, f32::NEG_INFINITY, f32::INFINITY] {
        for weighting in [Weighted, Unweighted] {
            assert!(matches!(
                map.percentile(p, weighting),
                Err(FlipError::InvalidParameter(_))
            ));
            assert!(matches!(
                pooled.percentile(p, weighting),
                Err(FlipError::InvalidParameter(_))
            ));
        }
    }
    // f32 multiplication rounds (2/3)*3 to exactly 2; its successor rounds above 2.
    let last = 2.0_f32 / 3.0;
    assert_eq!(pooled.percentile(last, Unweighted).unwrap(), 0.3);
    for p in [f32::from_bits(last.to_bits() + 1), 0.9, 0.99, 1.0] {
        assert!(matches!(
            map.percentile(p, Unweighted),
            Err(FlipError::InvalidParameter(_))
        ));
        assert!(matches!(
            pooled.percentile(p, Unweighted),
            Err(FlipError::InvalidParameter(_))
        ));
    }
    let single = ErrorMap::new(1, 1, vec![0.2]).unwrap();
    assert!(matches!(
        single.percentile(f32::from_bits(1), Unweighted),
        Err(FlipError::InvalidParameter(_))
    ));
    assert!(matches!(
        ErrorMap::new(0, 1, vec![]),
        Err(FlipError::InvalidDimensions)
    ));
}

#[test]
fn arbitrary_weighted_quartiles_match_statistics_bit_for_bit() {
    for values in [
        vec![0.0],
        vec![0.4, 0.0, 0.3, 0.1, 0.2],
        vec![1e8, 4.0, 4.0],
        vec![f32::MAX; 2],
    ] {
        let map = ErrorMap::new(values.len(), 1, values).unwrap();
        let stats = map.statistics();
        let pooled = map.percentiles().unwrap();
        for (p, expected) in [
            (0.25, stats.first_quartile),
            (0.5, stats.weighted_median),
            (0.75, stats.third_quartile),
        ] {
            assert_eq!(
                pooled.percentile(p, Weighted).unwrap().to_bits(),
                expected.to_bits()
            );
        }
    }
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
fn hostile_ppd_is_bounded_and_input_validation_precedes_filters() {
    let image = rgb(1, 1, &[0.0; 3]);
    assert!(matches!(
        ldr_flip(&image, &image, 1e10),
        Err(FlipError::InvalidParameter(_))
    ));
    let nan = rgb(1, 1, &[f32::NAN; 3]);
    assert!(matches!(
        ldr_flip(&nan, &image, 1e10),
        Err(FlipError::NonFiniteInput)
    ));
    let mut options = explicit(Tonemapper::Aces, 0.0, 0.0, 2);
    options.ppd = 1e10;
    assert!(matches!(
        hdr_flip(&image, &image, options),
        Err(FlipError::InvalidParameter(_))
    ));
}

#[test]
fn hdr_workload_is_bounded_and_equal_endpoints_preserve_semantics() {
    let a = rgb(1, 1, &[0.1; 3]);
    let b = rgb(1, 1, &[0.2; 3]);
    for count in [129, i32::MAX as usize, usize::MAX] {
        assert!(matches!(
            hdr_flip(&a, &b, explicit(Tonemapper::Aces, 0.0, 0.0, count)),
            Err(FlipError::InvalidParameter(_))
        ));
    }
    let two = hdr_flip(&a, &b, explicit(Tonemapper::Aces, 0.0, 0.0, 2)).unwrap();
    let many = hdr_flip(&a, &b, explicit(Tonemapper::Aces, 0.0, 0.0, 128)).unwrap();
    assert_eq!(two.error_map.pixels(), many.error_map.pixels());
    assert_eq!(many.exposure_map.unwrap().pixels(), &[0.0]);
    assert_eq!(many.parameters.num_exposures, 128);
    let mut auto = explicit(Tonemapper::Aces, -80.0, 80.0, 2);
    auto.num_exposures = None;
    assert!(hdr_flip(&a, &b, auto).is_err());
}

#[test]
fn pooling_flags_nonfinite_total_without_changing_reference_arithmetic() {
    let s = ErrorMap::new(2, 1, vec![f32::MAX; 2]).unwrap().statistics();
    assert!(!s.finite);
    assert!(s.mean.is_infinite());
    assert_eq!(s.histogram.out_of_range, 2);
    assert!(ErrorMap::new(1, 1, vec![0.5]).unwrap().statistics().finite);
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
    assert!(flip_rs::io::load_linear(dir.join("test.png")).is_err());
    assert!(flip_rs::io::load_srgb(dir.join("test.exr")).is_err());
    assert_eq!((loaded.width(), loaded.height()), (2, 1));
    for (&a, &b) in loaded.pixels().iter().zip(map.colorize().unwrap().pixels()) {
        assert!((a - b).abs() <= 0.5 / 255.0 + f32::EPSILON);
    }
    std::fs::remove_dir_all(dir).unwrap();
}
