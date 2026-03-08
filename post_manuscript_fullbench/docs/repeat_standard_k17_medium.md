# Local Repeat Bench (full-local, fixed-k)

- repeats per point: `3`
- k_run fixed: `17`
- order policy: `alternate`
- input profile: `standard`

| scale | A prove (ms) | B prove (ms) | B/A prove | A verify (ms) | B verify (ms) | B/A verify | A keygen(vk+pk) ms | B keygen(vk+pk) ms | A proof bytes | B proof bytes |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 8 | 15274.84 +/- 346.12 | 13088.40 +/- 326.02 | 0.857 | 7.02 +/- 0.21 | 4.60 +/- 0.42 | 0.654 | 587.18 | 1248.47 | 11072 | 5088 |

Notes:
- Ratios are computed from mean metrics (B_mean / A_mean).
- `proof_bytes` should be deterministic per arm for fixed `k` and circuit shape.
- With `alternate` ordering, odd repeats run A->B and even repeats run B->A.
