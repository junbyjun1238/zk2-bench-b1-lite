# Local Repeat Bench (full-local, fixed-k)

- repeats per point: `3`
- k_run fixed: `17`
- order policy: `alternate`
- input profile: `standard`

| scale | A prove (ms) | B prove (ms) | B/A prove | A verify (ms) | B verify (ms) | B/A verify | A keygen(vk+pk) ms | B keygen(vk+pk) ms | A proof bytes | B proof bytes |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 15997.12 +/- 592.68 | 13136.11 +/- 418.37 | 0.821 | 7.02 +/- 0.41 | 4.43 +/- 0.20 | 0.631 | 635.50 | 1275.93 | 11072 | 5088 |
| 4 | 15455.09 +/- 175.78 | 12931.51 +/- 412.15 | 0.837 | 6.82 +/- 0.32 | 4.59 +/- 0.32 | 0.672 | 615.19 | 1251.83 | 11072 | 5088 |

Notes:
- Ratios are computed from mean metrics (B_mean / A_mean).
- `proof_bytes` should be deterministic per arm for fixed `k` and circuit shape.
- With `alternate` ordering, odd repeats run A->B and even repeats run B->A.
