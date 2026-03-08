# Local Repeat Bench (full-local, fixed-k)

- repeats per point: `3`
- k_run fixed: `18`
- order policy: `alternate`
- input profile: `boundary`

| scale | A prove (ms) | B prove (ms) | B/A prove | A verify (ms) | B verify (ms) | B/A verify | A keygen(vk+pk) ms | B keygen(vk+pk) ms | A proof bytes | B proof bytes |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 4520 | 32371.62 +/- 453.79 | 26709.88 +/- 389.95 | 0.825 | 6.81 +/- 0.22 | 4.38 +/- 0.16 | 0.643 | 3911.21 | 4220.62 | 11072 | 5088 |

Notes:
- Ratios are computed from mean metrics (B_mean / A_mean).
- `proof_bytes` should be deterministic per arm for fixed `k` and circuit shape.
- With `alternate` ordering, odd repeats run A->B and even repeats run B->A.
