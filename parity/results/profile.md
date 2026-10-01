# Single-thread stage profile

Timers were inserted into a temporary copy of the sources by
`parity/profile.py`. Release profile, one warmup followed by three samples.
HDR stage times sum all three exposures per sample before taking the median.
Pooling is outside the comparison API. Stage medians need not add up to the
separately measured API median. Hardware is recorded in
[qualification.md](qualification.md).

| Case | Stage | Median seconds |
|---|---|---:|
| LDR 1920x1080 | LDR conversion/setup | 0.089211 |
| LDR 1920x1080 | Color horizontal | 0.054509 |
| LDR 1920x1080 | Color vertical/Lab metric | 0.126692 |
| LDR 1920x1080 | Feature luminance normalization | 0.002156 |
| LDR 1920x1080 | Feature horizontal | 0.020489 |
| LDR 1920x1080 | Feature vertical/final metric | 0.050686 |
| LDR 1920x1080 | API total | 0.354398 |
| LDR 1920x1080 | Pooling (outside API) | 0.058851 |
| HDR 1920x1080 | HDR setup/resolve | 0.004875 |
| HDR 1920x1080 | HDR tone/opponent conversion | 0.048681 |
| HDR 1920x1080 | Color horizontal | 0.121660 |
| HDR 1920x1080 | Color vertical/Lab metric | 0.379501 |
| HDR 1920x1080 | Feature luminance normalization | 0.005711 |
| HDR 1920x1080 | Feature horizontal | 0.068751 |
| HDR 1920x1080 | Feature vertical/final metric | 0.167305 |
| HDR 1920x1080 | HDR max/exposure merge | 0.003518 |
| HDR 1920x1080 | API total | 0.803407 |
| HDR 1920x1080 | Pooling (outside API) | 0.061865 |
| LDR 3840x2160 | LDR conversion/setup | 0.393467 |
| LDR 3840x2160 | Color horizontal | 0.273361 |
| LDR 3840x2160 | Color vertical/Lab metric | 0.507524 |
| LDR 3840x2160 | Feature luminance normalization | 0.007930 |
| LDR 3840x2160 | Feature horizontal | 0.088842 |
| LDR 3840x2160 | Feature vertical/final metric | 0.222909 |
| LDR 3840x2160 | API total | 1.547075 |
| LDR 3840x2160 | Pooling (outside API) | 0.245633 |
| HDR 3840x2160 | HDR setup/resolve | 0.017044 |
| HDR 3840x2160 | HDR tone/opponent conversion | 0.212999 |
| HDR 3840x2160 | Color horizontal | 0.502358 |
| HDR 3840x2160 | Color vertical/Lab metric | 1.520268 |
| HDR 3840x2160 | Feature luminance normalization | 0.022710 |
| HDR 3840x2160 | Feature horizontal | 0.266010 |
| HDR 3840x2160 | Feature vertical/final metric | 0.661363 |
| HDR 3840x2160 | HDR max/exposure merge | 0.033348 |
| HDR 3840x2160 | API total | 3.399873 |
| HDR 3840x2160 | Pooling (outside API) | 0.240021 |

SHA-256 of the measured sources before instrumentation:

- `Cargo.toml`: `aebc8ff864f32d40f209b79330a7d2a2be27fde20ecd00d4a2ca5972cfc5a19c`
- `Cargo.lock`: `4e4fc4ff20f311e0daf9a9745c8d8d3e0a9dee320be259961936d22ea6e4d0b3`
- `src/lib.rs`: `cc594f87ed3a729735f948e7fc6f8571eb89f52c7d2dc44c537b263d0415b501`
- `src/color.rs`: `a4e85f80771d75b036530c0151e56e5686671609c21ef3835ed1fb2e7b735ba5`
- `src/filters.rs`: `9e737959215ff2f390d67b5a3a235c1fd9896b0592f9fe8163e850769d3927b1`
- `src/hdr.rs`: `ac42cd2b71b41bc7c33934daf4a1f5f64c4ed70f09f4542739f3bb8eebbe9a02`

Raw stage timings:

```text
CASE|LDR 1920x1080|0
STAGE|LDR conversion/setup|0.084249481
STAGE|Color horizontal|0.052261802
STAGE|Color vertical/Lab metric|0.115241968
STAGE|Feature luminance normalization|0.002429070
STAGE|Feature horizontal|0.021361368
STAGE|Feature vertical/final metric|0.051782269
STAGE|API total|0.336336170
STAGE|Pooling (outside API)|0.060988580
CASE|LDR 1920x1080|1
STAGE|LDR conversion/setup|0.085242820
STAGE|Color horizontal|0.052930339
STAGE|Color vertical/Lab metric|0.126691930
STAGE|Feature luminance normalization|0.002156427
STAGE|Feature horizontal|0.024165874
STAGE|Feature vertical/final metric|0.056070677
STAGE|API total|0.354398141
STAGE|Pooling (outside API)|0.058851159
CASE|LDR 1920x1080|2
STAGE|LDR conversion/setup|0.094266206
STAGE|Color horizontal|0.060692833
STAGE|Color vertical/Lab metric|0.137550519
STAGE|Feature luminance normalization|0.002515152
STAGE|Feature horizontal|0.020489467
STAGE|Feature vertical/final metric|0.047758800
STAGE|API total|0.367929357
STAGE|Pooling (outside API)|0.054394975
CASE|LDR 1920x1080|3
STAGE|LDR conversion/setup|0.089211436
STAGE|Color horizontal|0.054508629
STAGE|Color vertical/Lab metric|0.120140434
STAGE|Feature luminance normalization|0.002001184
STAGE|Feature horizontal|0.020218418
STAGE|Feature vertical/final metric|0.050685686
STAGE|API total|0.341887003
STAGE|Pooling (outside API)|0.059205856
CASE|HDR 1920x1080|0
STAGE|HDR setup/resolve|0.004170236
STAGE|HDR tone/opponent conversion|0.036506972
STAGE|Color horizontal|0.062501896
STAGE|Color vertical/Lab metric|0.126466876
STAGE|Feature luminance normalization|0.002089591
STAGE|Feature horizontal|0.021129431
STAGE|Feature vertical/final metric|0.053895385
STAGE|HDR max/exposure merge|0.012236271
STAGE|HDR tone/opponent conversion|0.011452065
STAGE|Color horizontal|0.030318389
STAGE|Color vertical/Lab metric|0.120242256
STAGE|Feature luminance normalization|0.002386750
STAGE|Feature horizontal|0.020417232
STAGE|Feature vertical/final metric|0.052231675
STAGE|HDR max/exposure merge|0.001354779
STAGE|HDR tone/opponent conversion|0.010196413
STAGE|Color horizontal|0.028921521
STAGE|Color vertical/Lab metric|0.116205932
STAGE|Feature luminance normalization|0.002462482
STAGE|Feature horizontal|0.021365917
STAGE|Feature vertical/final metric|0.054536582
STAGE|HDR max/exposure merge|0.000765370
STAGE|API total|0.802198632
STAGE|Pooling (outside API)|0.065474769
CASE|HDR 1920x1080|1
STAGE|HDR setup/resolve|0.005234308
STAGE|HDR tone/opponent conversion|0.028212657
STAGE|Color horizontal|0.059662235
STAGE|Color vertical/Lab metric|0.135813050
STAGE|Feature luminance normalization|0.002272886
STAGE|Feature horizontal|0.021996202
STAGE|Feature vertical/final metric|0.057918955
STAGE|HDR max/exposure merge|0.001639364
STAGE|HDR tone/opponent conversion|0.010970820
STAGE|Color horizontal|0.029622910
STAGE|Color vertical/Lab metric|0.118275616
STAGE|Feature luminance normalization|0.002836797
STAGE|Feature horizontal|0.019980479
STAGE|Feature vertical/final metric|0.054253048
STAGE|HDR max/exposure merge|0.001426123
STAGE|HDR tone/opponent conversion|0.009497949
STAGE|Color horizontal|0.027784331
STAGE|Color vertical/Lab metric|0.116670737
STAGE|Feature luminance normalization|0.001828339
STAGE|Feature horizontal|0.023921063
STAGE|Feature vertical/final metric|0.055133054
STAGE|HDR max/exposure merge|0.000754980
STAGE|API total|0.798099369
STAGE|Pooling (outside API)|0.066819349
CASE|HDR 1920x1080|2
STAGE|HDR setup/resolve|0.004875122
STAGE|HDR tone/opponent conversion|0.030261532
STAGE|Color horizontal|0.059626156
STAGE|Color vertical/Lab metric|0.134563870
STAGE|Feature luminance normalization|0.002134857
STAGE|Feature horizontal|0.028550984
STAGE|Feature vertical/final metric|0.061460428
STAGE|HDR max/exposure merge|0.001289395
STAGE|HDR tone/opponent conversion|0.012574487
STAGE|Color horizontal|0.033439981
STAGE|Color vertical/Lab metric|0.134874855
STAGE|Feature luminance normalization|0.001821888
STAGE|Feature horizontal|0.028070429
STAGE|Feature vertical/final metric|0.061872713
STAGE|HDR max/exposure merge|0.001474274
STAGE|HDR tone/opponent conversion|0.009854509
STAGE|Color horizontal|0.028593614
STAGE|Color vertical/Lab metric|0.110062424
STAGE|Feature luminance normalization|0.001754571
STAGE|Feature horizontal|0.018785702
STAGE|Feature vertical/final metric|0.045765170
STAGE|HDR max/exposure merge|0.000662827
STAGE|API total|0.823131914
STAGE|Pooling (outside API)|0.058580870
CASE|HDR 1920x1080|3
STAGE|HDR setup/resolve|0.004389047
STAGE|HDR tone/opponent conversion|0.027659567
STAGE|Color horizontal|0.056783118
STAGE|Color vertical/Lab metric|0.123020703
STAGE|Feature luminance normalization|0.001696662
STAGE|Feature horizontal|0.023339981
STAGE|Feature vertical/final metric|0.052194004
STAGE|HDR max/exposure merge|0.001292311
STAGE|HDR tone/opponent conversion|0.010025661
STAGE|Color horizontal|0.032219907
STAGE|Color vertical/Lab metric|0.142717409
STAGE|Feature luminance normalization|0.001873755
STAGE|Feature horizontal|0.022474062
STAGE|Feature vertical/final metric|0.051456257
STAGE|HDR max/exposure merge|0.001450950
STAGE|HDR tone/opponent conversion|0.009551329
STAGE|Color horizontal|0.035408635
STAGE|Color vertical/Lab metric|0.119265658
STAGE|Feature luminance normalization|0.002137902
STAGE|Feature horizontal|0.022936531
STAGE|Feature vertical/final metric|0.051997945
STAGE|HDR max/exposure merge|0.000774397
STAGE|API total|0.803406685
STAGE|Pooling (outside API)|0.061864918
CASE|LDR 3840x2160|0
STAGE|LDR conversion/setup|0.331820424
STAGE|Color horizontal|0.225254679
STAGE|Color vertical/Lab metric|0.516252823
STAGE|Feature luminance normalization|0.008645725
STAGE|Feature horizontal|0.082477318
STAGE|Feature vertical/final metric|0.240769187
STAGE|API total|1.448087915
STAGE|Pooling (outside API)|0.272172727
CASE|LDR 3840x2160|1
STAGE|LDR conversion/setup|0.394228224
STAGE|Color horizontal|0.276940196
STAGE|Color vertical/Lab metric|0.524290885
STAGE|Feature luminance normalization|0.009290950
STAGE|Feature horizontal|0.088842262
STAGE|Feature vertical/final metric|0.223346600
STAGE|API total|1.568660280
STAGE|Pooling (outside API)|0.245632556
CASE|LDR 3840x2160|2
STAGE|LDR conversion/setup|0.393466901
STAGE|Color horizontal|0.273361314
STAGE|Color vertical/Lab metric|0.507523610
STAGE|Feature luminance normalization|0.007930059
STAGE|Feature horizontal|0.089735533
STAGE|Feature vertical/final metric|0.222514484
STAGE|API total|1.547074991
STAGE|Pooling (outside API)|0.232927674
CASE|LDR 3840x2160|3
STAGE|LDR conversion/setup|0.374860427
STAGE|Color horizontal|0.240787211
STAGE|Color vertical/Lab metric|0.503756403
STAGE|Feature luminance normalization|0.006861168
STAGE|Feature horizontal|0.076790639
STAGE|Feature vertical/final metric|0.222908685
STAGE|API total|1.465809644
STAGE|Pooling (outside API)|0.247305834
CASE|HDR 3840x2160|0
STAGE|HDR setup/resolve|0.015149271
STAGE|HDR tone/opponent conversion|0.119075099
STAGE|Color horizontal|0.261419717
STAGE|Color vertical/Lab metric|0.569990762
STAGE|Feature luminance normalization|0.007727097
STAGE|Feature horizontal|0.112529666
STAGE|Feature vertical/final metric|0.220156287
STAGE|HDR max/exposure merge|0.006058448
STAGE|HDR tone/opponent conversion|0.036807888
STAGE|Color horizontal|0.122760634
STAGE|Color vertical/Lab metric|0.443049152
STAGE|Feature luminance normalization|0.007428415
STAGE|Feature horizontal|0.089686620
STAGE|Feature vertical/final metric|0.221695463
STAGE|HDR max/exposure merge|0.006086150
STAGE|HDR tone/opponent conversion|0.037847984
STAGE|Color horizontal|0.169231741
STAGE|Color vertical/Lab metric|0.548925270
STAGE|Feature luminance normalization|0.008741696
STAGE|Feature horizontal|0.125973648
STAGE|Feature vertical/final metric|0.238773822
STAGE|HDR max/exposure merge|0.002532214
STAGE|API total|3.417279066
STAGE|Pooling (outside API)|0.233043572
CASE|HDR 3840x2160|1
STAGE|HDR setup/resolve|0.017043956
STAGE|HDR tone/opponent conversion|0.133945647
STAGE|Color horizontal|0.243204117
STAGE|Color vertical/Lab metric|0.489907989
STAGE|Feature luminance normalization|0.007655002
STAGE|Feature horizontal|0.083247255
STAGE|Feature vertical/final metric|0.215949613
STAGE|HDR max/exposure merge|0.024648191
STAGE|HDR tone/opponent conversion|0.038812319
STAGE|Color horizontal|0.116111905
STAGE|Color vertical/Lab metric|0.453968425
STAGE|Feature luminance normalization|0.007513215
STAGE|Feature horizontal|0.083210858
STAGE|Feature vertical/final metric|0.225852193
STAGE|HDR max/exposure merge|0.005709772
STAGE|HDR tone/opponent conversion|0.040240696
STAGE|Color horizontal|0.120702000
STAGE|Color vertical/Lab metric|0.479143718
STAGE|Feature luminance normalization|0.007541437
STAGE|Feature horizontal|0.088288420
STAGE|Feature vertical/final metric|0.219560877
STAGE|HDR max/exposure merge|0.002989985
STAGE|API total|3.155744223
STAGE|Pooling (outside API)|0.240021109
CASE|HDR 3840x2160|2
STAGE|HDR setup/resolve|0.016703364
STAGE|HDR tone/opponent conversion|0.134880245
STAGE|Color horizontal|0.231800774
STAGE|Color vertical/Lab metric|0.501604444
STAGE|Feature luminance normalization|0.009851294
STAGE|Feature horizontal|0.089761351
STAGE|Feature vertical/final metric|0.280540730
STAGE|HDR max/exposure merge|0.025283887
STAGE|HDR tone/opponent conversion|0.041602277
STAGE|Color horizontal|0.123663772
STAGE|Color vertical/Lab metric|0.526779247
STAGE|Feature luminance normalization|0.008312278
STAGE|Feature horizontal|0.111746502
STAGE|Feature vertical/final metric|0.228563924
STAGE|HDR max/exposure merge|0.005461855
STAGE|HDR tone/opponent conversion|0.041157682
STAGE|Color horizontal|0.146893426
STAGE|Color vertical/Lab metric|0.491884178
STAGE|Feature luminance normalization|0.009114157
STAGE|Feature horizontal|0.080170517
STAGE|Feature vertical/final metric|0.230993896
STAGE|HDR max/exposure merge|0.003597939
STAGE|API total|3.399873350
STAGE|Pooling (outside API)|0.285215435
CASE|HDR 3840x2160|3
STAGE|HDR setup/resolve|0.020375333
STAGE|HDR tone/opponent conversion|0.135308440
STAGE|Color horizontal|0.285946771
STAGE|Color vertical/Lab metric|0.585806656
STAGE|Feature luminance normalization|0.007377329
STAGE|Feature horizontal|0.102510346
STAGE|Feature vertical/final metric|0.238181969
STAGE|HDR max/exposure merge|0.022750461
STAGE|HDR tone/opponent conversion|0.036186579
STAGE|Color horizontal|0.151256864
STAGE|Color vertical/Lab metric|0.498792274
STAGE|Feature luminance normalization|0.007501062
STAGE|Feature horizontal|0.079845706
STAGE|Feature vertical/final metric|0.208369332
STAGE|HDR max/exposure merge|0.005175127
STAGE|HDR tone/opponent conversion|0.035809540
STAGE|Color horizontal|0.154748072
STAGE|Color vertical/Lab metric|0.519484402
STAGE|Feature luminance normalization|0.007053079
STAGE|Feature horizontal|0.083654172
STAGE|Feature vertical/final metric|0.206246689
STAGE|HDR max/exposure merge|0.002796782
STAGE|API total|3.437132587
STAGE|Pooling (outside API)|0.230407352
```
