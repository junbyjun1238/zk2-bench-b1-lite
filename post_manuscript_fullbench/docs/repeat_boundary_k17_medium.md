# Local Repeat Bench (full-local, fixed-k)

- repeats per point: `3`
- k_run fixed: `17`
- order policy: `alternate`
- input profile: `boundary`

| scale | A prove (ms) | B prove (ms) | B/A prove | A verify (ms) | B verify (ms) | B/A verify | A keygen(vk+pk) ms | B keygen(vk+pk) ms | A proof bytes | B proof bytes |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 8 | 15761.38 +/- 536.93 | 13156.00 +/- 362.42 | 0.835 | 6.64 +/- 0.08 | 4.44 +/- 0.31 | 0.668 | 555.36 | 1281.45 | 11072 | 5088 |

Notes:
- Ratios are computed from mean metrics (B_mean / A_mean).
- `proof_bytes` should be deterministic per arm for fixed `k` and circuit shape.
- With `alternate` ordering, odd repeats run A->B and even repeats run B->A.
