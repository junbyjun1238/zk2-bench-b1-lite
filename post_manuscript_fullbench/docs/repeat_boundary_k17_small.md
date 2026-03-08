# Local Repeat Bench (full-local, fixed-k)

- repeats per point: `3`
- k_run fixed: `17`
- order policy: `alternate`
- input profile: `boundary`

| scale | A prove (ms) | B prove (ms) | B/A prove | A verify (ms) | B verify (ms) | B/A verify | A keygen(vk+pk) ms | B keygen(vk+pk) ms | A proof bytes | B proof bytes |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 15746.79 +/- 591.61 | 13238.93 +/- 143.21 | 0.841 | 6.86 +/- 0.17 | 4.32 +/- 0.23 | 0.630 | 581.25 | 1272.69 | 11072 | 5088 |
| 4 | 14981.44 +/- 231.80 | 12922.92 +/- 128.55 | 0.863 | 6.63 +/- 0.14 | 4.39 +/- 0.29 | 0.662 | 593.30 | 1213.97 | 11072 | 5088 |

Notes:
- Ratios are computed from mean metrics (B_mean / A_mean).
- `proof_bytes` should be deterministic per arm for fixed `k` and circuit shape.
- With `alternate` ordering, odd repeats run A->B and even repeats run B->A.
