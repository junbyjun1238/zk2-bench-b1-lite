# Local Repeat Bench (full-local, fixed-k)

- repeats per point: `2`
- k_run fixed: `13`

| scale | A prove (ms) | B prove (ms) | B/A prove | A verify (ms) | B verify (ms) | B/A verify | A keygen(vk+pk) ms | B keygen(vk+pk) ms | A proof bytes | B proof bytes |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 16 | 12389.29 ± 421.41 | 9213.91 ± 13.34 | 0.744 | 43.30 ± 1.61 | 28.13 ± 0.87 | 0.650 | 602.63 | 880.38 | 11072 | 5088 |
| 24 | 11930.64 ± 188.05 | 8697.83 ± 44.78 | 0.729 | 42.48 ± 0.57 | 26.06 ± 0.00 | 0.613 | 671.17 | 829.64 | 11072 | 5088 |
| 32 | 11596.18 ± 31.20 | 8617.56 ± 130.06 | 0.743 | 43.54 ± 0.36 | 27.59 ± 0.01 | 0.634 | 696.37 | 885.93 | 11072 | 5088 |

Notes:
- Ratios are computed from mean metrics (B_mean / A_mean).
- `proof_bytes` should be deterministic per arm for fixed `k` and circuit shape.
