> Historical: measured before the 0.1.0 hardening changes, with no recorded Rust revision or executable hash. It does not describe the current code; see [qualification.md](qualification.md) for current measurements.

# Single-thread stage profile, untiled filters

Stage profile of the earlier untiled filter implementation, for comparison
with [profile.md](profile.md). Timers were inserted into a temporary copy of
the sources. Release profile, one warmup followed by three samples. HDR stage
times sum all three exposures per sample before taking the median. Pooling is
outside the comparison API. Stage medians need not add up to the separately
measured API median. Hardware is recorded in
[qualification.md](qualification.md).

| Case | Stage | Median seconds |
|---|---|---:|
| LDR 1920x1080 | LDR conversion/setup | 0.108794 |
| LDR 1920x1080 | Color horizontal | 0.069799 |
| LDR 1920x1080 | Color vertical/Lab metric | 0.234710 |
| LDR 1920x1080 | Feature horizontal | 0.072929 |
| LDR 1920x1080 | Feature vertical/final metric | 0.099664 |
| LDR 1920x1080 | API total | 0.594978 |
| LDR 1920x1080 | Pooling (outside API) | 0.053193 |
| HDR 1920x1080 | HDR setup/resolve | 0.004678 |
| HDR 1920x1080 | HDR tone/opponent conversion | 0.126468 |
| HDR 1920x1080 | Color horizontal | 0.210754 |
| HDR 1920x1080 | Color vertical/Lab metric | 0.758359 |
| HDR 1920x1080 | Feature horizontal | 0.225987 |
| HDR 1920x1080 | Feature vertical/final metric | 0.312275 |
| HDR 1920x1080 | HDR max/exposure merge | 0.003946 |
| HDR 1920x1080 | API total | 1.685300 |
| HDR 1920x1080 | Pooling (outside API) | 0.059202 |
| LDR 3840x2160 | LDR conversion/setup | 0.500902 |
| LDR 3840x2160 | Color horizontal | 0.272410 |
| LDR 3840x2160 | Color vertical/Lab metric | 0.948506 |
| LDR 3840x2160 | Feature horizontal | 0.292381 |
| LDR 3840x2160 | Feature vertical/final metric | 0.439452 |
| LDR 3840x2160 | API total | 2.499559 |
| LDR 3840x2160 | Pooling (outside API) | 0.226606 |
| HDR 3840x2160 | HDR setup/resolve | 0.018288 |
| HDR 3840x2160 | HDR tone/opponent conversion | 0.653653 |
| HDR 3840x2160 | Color horizontal | 1.136103 |
| HDR 3840x2160 | Color vertical/Lab metric | 3.471036 |
| HDR 3840x2160 | Feature horizontal | 1.309120 |
| HDR 3840x2160 | Feature vertical/final metric | 1.761130 |
| HDR 3840x2160 | HDR max/exposure merge | 0.036023 |
| HDR 3840x2160 | API total | 8.677525 |
| HDR 3840x2160 | Pooling (outside API) | 0.269964 |

SHA-256 of the measured sources before instrumentation:

- `Cargo.toml`: `aebc8ff864f32d40f209b79330a7d2a2be27fde20ecd00d4a2ca5972cfc5a19c`
- `Cargo.lock`: `4e4fc4ff20f311e0daf9a9745c8d8d3e0a9dee320be259961936d22ea6e4d0b3`
- `src/lib.rs`: `d555a49a61f98635c8007e2e2ebc1036cadbae5adfa3fd0822bef4bb4a099ddc`
- `src/color.rs`: `c043b39e8a7070764fa84a68e00657245a09ce1ee13bf9154d4ae28713cccb6f`
- `src/filters.rs`: `139529d52c000e4a2578314be45099625d472335a93d04ef4fc6791c51dd5d47`
- `src/hdr.rs`: `02fb6564aa58c60f674b410b0c5517c59e649f3984570e97275933c814fdda6b`

Raw stage timings:

```text
CASE|LDR 1920x1080|0
STAGE|LDR conversion/setup|0.121863706
STAGE|Color horizontal|0.066074458
STAGE|Color vertical/Lab metric|0.229664145
STAGE|Feature horizontal|0.074128381
STAGE|Feature vertical/final metric|0.102046464
STAGE|API total|0.602438398
STAGE|Pooling (outside API)|0.054635267
CASE|LDR 1920x1080|1
STAGE|LDR conversion/setup|0.111908407
STAGE|Color horizontal|0.069941383
STAGE|Color vertical/Lab metric|0.234709528
STAGE|Feature horizontal|0.074651144
STAGE|Feature vertical/final metric|0.102372978
STAGE|API total|0.607813480
STAGE|Pooling (outside API)|0.053192933
CASE|LDR 1920x1080|2
STAGE|LDR conversion/setup|0.106053892
STAGE|Color horizontal|0.067696921
STAGE|Color vertical/Lab metric|0.235349821
STAGE|Feature horizontal|0.072928523
STAGE|Feature vertical/final metric|0.093816701
STAGE|API total|0.590152875
STAGE|Pooling (outside API)|0.053007716
CASE|LDR 1920x1080|3
STAGE|LDR conversion/setup|0.108794489
STAGE|Color horizontal|0.069798564
STAGE|Color vertical/Lab metric|0.231097201
STAGE|Feature horizontal|0.071705312
STAGE|Feature vertical/final metric|0.099664201
STAGE|API total|0.594978493
STAGE|Pooling (outside API)|0.053366440
CASE|HDR 1920x1080|0
STAGE|HDR setup/resolve|0.004739957
STAGE|HDR tone/opponent conversion|0.041283758
STAGE|Color horizontal|0.067263215
STAGE|Color vertical/Lab metric|0.237489818
STAGE|Feature horizontal|0.072666971
STAGE|Feature vertical/final metric|0.100713455
STAGE|HDR max/exposure merge|0.004692398
STAGE|HDR tone/opponent conversion|0.040731519
STAGE|Color horizontal|0.069075714
STAGE|Color vertical/Lab metric|0.249802812
STAGE|Feature horizontal|0.076738942
STAGE|Feature vertical/final metric|0.104320211
STAGE|HDR max/exposure merge|0.001507576
STAGE|HDR tone/opponent conversion|0.042118109
STAGE|Color horizontal|0.070919854
STAGE|Color vertical/Lab metric|0.255741926
STAGE|Feature horizontal|0.077679030
STAGE|Feature vertical/final metric|0.100408342
STAGE|HDR max/exposure merge|0.000978411
STAGE|API total|1.662907815
STAGE|Pooling (outside API)|0.060140975
CASE|HDR 1920x1080|1
STAGE|HDR setup/resolve|0.004782037
STAGE|HDR tone/opponent conversion|0.040216100
STAGE|Color horizontal|0.070732752
STAGE|Color vertical/Lab metric|0.247938225
STAGE|Feature horizontal|0.077766655
STAGE|Feature vertical/final metric|0.105571505
STAGE|HDR max/exposure merge|0.001697433
STAGE|HDR tone/opponent conversion|0.044681451
STAGE|Color horizontal|0.071109380
STAGE|Color vertical/Lab metric|0.266436526
STAGE|Feature horizontal|0.081350097
STAGE|Feature vertical/final metric|0.101941946
STAGE|HDR max/exposure merge|0.001588909
STAGE|HDR tone/opponent conversion|0.043822054
STAGE|Color horizontal|0.068911545
STAGE|Color vertical/Lab metric|0.258038657
STAGE|Feature horizontal|0.073994849
STAGE|Feature vertical/final metric|0.099821808
STAGE|HDR max/exposure merge|0.000829521
STAGE|API total|1.702566687
STAGE|Pooling (outside API)|0.059876268
CASE|HDR 1920x1080|2
STAGE|HDR setup/resolve|0.004620833
STAGE|HDR tone/opponent conversion|0.041316991
STAGE|Color horizontal|0.074379392
STAGE|Color vertical/Lab metric|0.246134522
STAGE|Feature horizontal|0.075169199
STAGE|Feature vertical/final metric|0.109039229
STAGE|HDR max/exposure merge|0.001639365
STAGE|HDR tone/opponent conversion|0.042949262
STAGE|Color horizontal|0.071572131
STAGE|Color vertical/Lab metric|0.254956819
STAGE|Feature horizontal|0.075594278
STAGE|Feature vertical/final metric|0.105103204
STAGE|HDR max/exposure merge|0.001549004
STAGE|HDR tone/opponent conversion|0.042136984
STAGE|Color horizontal|0.068706871
STAGE|Color vertical/Lab metric|0.254375125
STAGE|Feature horizontal|0.071648795
STAGE|Feature vertical/final metric|0.098132470
STAGE|HDR max/exposure merge|0.000757836
STAGE|API total|1.679436841
STAGE|Pooling (outside API)|0.057875403
CASE|HDR 1920x1080|3
STAGE|HDR setup/resolve|0.004677981
STAGE|HDR tone/opponent conversion|0.039762576
STAGE|Color horizontal|0.068678758
STAGE|Color vertical/Lab metric|0.251960903
STAGE|Feature horizontal|0.076287272
STAGE|Feature vertical/final metric|0.109063355
STAGE|HDR max/exposure merge|0.001583388
STAGE|HDR tone/opponent conversion|0.042905339
STAGE|Color horizontal|0.069800037
STAGE|Color vertical/Lab metric|0.251278269
STAGE|Feature horizontal|0.074429376
STAGE|Feature vertical/final metric|0.104920330
STAGE|HDR max/exposure merge|0.001572489
STAGE|HDR tone/opponent conversion|0.043799832
STAGE|Color horizontal|0.068913289
STAGE|Color vertical/Lab metric|0.255119816
STAGE|Feature horizontal|0.075270629
STAGE|Feature vertical/final metric|0.099980947
STAGE|HDR max/exposure merge|0.000754680
STAGE|API total|1.685299872
STAGE|Pooling (outside API)|0.059201518
CASE|LDR 3840x2160|0
STAGE|LDR conversion/setup|0.506167990
STAGE|Color horizontal|0.279078900
STAGE|Color vertical/Lab metric|0.970676387
STAGE|Feature horizontal|0.299595819
STAGE|Feature vertical/final metric|0.457212289
STAGE|API total|2.568312744
STAGE|Pooling (outside API)|0.272279399
CASE|LDR 3840x2160|1
STAGE|LDR conversion/setup|0.505840295
STAGE|Color horizontal|0.272410325
STAGE|Color vertical/Lab metric|0.969353858
STAGE|Feature horizontal|0.303159895
STAGE|Feature vertical/final metric|0.447981483
STAGE|API total|2.554981372
STAGE|Pooling (outside API)|0.252223938
CASE|LDR 3840x2160|2
STAGE|LDR conversion/setup|0.500901643
STAGE|Color horizontal|0.268636174
STAGE|Color vertical/Lab metric|0.938898371
STAGE|Feature horizontal|0.282991441
STAGE|Feature vertical/final metric|0.418713209
STAGE|API total|2.462449449
STAGE|Pooling (outside API)|0.223142896
CASE|LDR 3840x2160|3
STAGE|LDR conversion/setup|0.493530025
STAGE|Color horizontal|0.272621522
STAGE|Color vertical/Lab metric|0.948505845
STAGE|Feature horizontal|0.292381067
STAGE|Feature vertical/final metric|0.439452056
STAGE|API total|2.499558813
STAGE|Pooling (outside API)|0.226605992
CASE|HDR 3840x2160|0
STAGE|HDR setup/resolve|0.018380210
STAGE|HDR tone/opponent conversion|0.172750972
STAGE|Color horizontal|0.279783917
STAGE|Color vertical/Lab metric|0.992690872
STAGE|Feature horizontal|0.312710864
STAGE|Feature vertical/final metric|0.449877359
STAGE|HDR max/exposure merge|0.006183403
STAGE|HDR tone/opponent conversion|0.185051654
STAGE|Color horizontal|0.285492327
STAGE|Color vertical/Lab metric|1.007212092
STAGE|Feature horizontal|0.308022372
STAGE|Feature vertical/final metric|0.458454155
STAGE|HDR max/exposure merge|0.005533841
STAGE|HDR tone/opponent conversion|0.175825767
STAGE|Color horizontal|0.299142637
STAGE|Color vertical/Lab metric|1.042118031
STAGE|Feature horizontal|0.307341151
STAGE|Feature vertical/final metric|0.431841147
STAGE|HDR max/exposure merge|0.003107606
STAGE|API total|6.922037147
STAGE|Pooling (outside API)|0.232406814
CASE|HDR 3840x2160|1
STAGE|HDR setup/resolve|0.018288447
STAGE|HDR tone/opponent conversion|0.172561636
STAGE|Color horizontal|0.279401597
STAGE|Color vertical/Lab metric|1.006149371
STAGE|Feature horizontal|0.307310593
STAGE|Feature vertical/final metric|0.499431607
STAGE|HDR max/exposure merge|0.021418596
STAGE|HDR tone/opponent conversion|0.182794928
STAGE|Color horizontal|0.281651079
STAGE|Color vertical/Lab metric|1.032657762
STAGE|Feature horizontal|0.306279714
STAGE|Feature vertical/final metric|0.448827224
STAGE|HDR max/exposure merge|0.005610374
STAGE|HDR tone/opponent conversion|0.172862582
STAGE|Color horizontal|0.289791554
STAGE|Color vertical/Lab metric|1.009188038
STAGE|Feature horizontal|0.296914956
STAGE|Feature vertical/final metric|0.423051090
STAGE|HDR max/exposure merge|0.003141931
STAGE|API total|6.940829419
STAGE|Pooling (outside API)|0.226413599
CASE|HDR 3840x2160|2
STAGE|HDR setup/resolve|0.017583300
STAGE|HDR tone/opponent conversion|0.167463485
STAGE|Color horizontal|0.275839296
STAGE|Color vertical/Lab metric|0.961443415
STAGE|Feature horizontal|0.292566485
STAGE|Feature vertical/final metric|0.430226148
STAGE|HDR max/exposure merge|0.020462457
STAGE|HDR tone/opponent conversion|0.171922394
STAGE|Color horizontal|0.281544118
STAGE|Color vertical/Lab metric|0.970196893
STAGE|Feature horizontal|0.454310018
STAGE|Feature vertical/final metric|0.716533317
STAGE|HDR max/exposure merge|0.009286701
STAGE|HDR tone/opponent conversion|0.314266680
STAGE|Color horizontal|0.578719293
STAGE|Color vertical/Lab metric|1.539395284
STAGE|Feature horizontal|0.562243016
STAGE|Feature vertical/final metric|0.685919743
STAGE|HDR max/exposure merge|0.006274114
STAGE|API total|8.677525389
STAGE|Pooling (outside API)|0.397800665
CASE|HDR 3840x2160|3
STAGE|HDR setup/resolve|0.038212370
STAGE|HDR tone/opponent conversion|0.283737574
STAGE|Color horizontal|0.579216208
STAGE|Color vertical/Lab metric|1.556418390
STAGE|Feature horizontal|0.554795325
STAGE|Feature vertical/final metric|0.664622255
STAGE|HDR max/exposure merge|0.029917885
STAGE|HDR tone/opponent conversion|0.256307460
STAGE|Color horizontal|0.535256645
STAGE|Color vertical/Lab metric|1.546747485
STAGE|Feature horizontal|0.579556969
STAGE|Feature vertical/final metric|0.577716596
STAGE|HDR max/exposure merge|0.006975433
STAGE|HDR tone/opponent conversion|0.203114065
STAGE|Color horizontal|0.328234698
STAGE|Color vertical/Lab metric|1.168537296
STAGE|Feature horizontal|0.354618144
STAGE|Feature vertical/final metric|0.518790707
STAGE|HDR max/exposure merge|0.004261296
STAGE|API total|10.013076370
STAGE|Pooling (outside API)|0.269964463
```
