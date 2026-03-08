# Local Repeat Bench (full-local, fixed-k)

- repeats per point: `3`
- k_run fixed: `17`
- order policy: `alternate`
- input profile: `boundary`

| scale | A prove (ms) | B prove (ms) | B/A prove | A verify (ms) | B verify (ms) | B/A verify | A keygen(vk+pk) ms | B keygen(vk+pk) ms | A proof bytes | B proof bytes |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 32 | 15097.18 +/- 386.18 | 13247.57 +/- 246.29 | 0.877 | 6.66 +/- 0.12 | 4.27 +/- 0.03 | 0.641 | 585.49 | 1236.95 | 11072 | 5088 |

Notes:
- Ratios are computed from mean metrics (B_mean / A_mean).
- `proof_bytes` should be deterministic per arm for fixed `k` and circuit shape.
- With `alternate` ordering, odd repeats run A->B and even repeats run B->A.
