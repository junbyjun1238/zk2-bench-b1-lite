# Local Repeat Bench (full-local, fixed-k)

- repeats per point: `1`
- k_run fixed: `18`
- order policy: `alternate`
- input profile: `boundary`

| scale | A prove (ms) | B prove (ms) | B/A prove | A verify (ms) | B verify (ms) | B/A verify | A keygen(vk+pk) ms | B keygen(vk+pk) ms | A proof bytes | B proof bytes |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 4520 | 33263.33 +/- 0.00 | 27789.66 +/- 0.00 | 0.835 | 7.45 +/- 0.00 | 4.30 +/- 0.00 | 0.577 | 4215.01 | 4316.73 | 11072 | 5088 |

Notes:
- Ratios are computed from mean metrics (B_mean / A_mean).
- `proof_bytes` should be deterministic per arm for fixed `k` and circuit shape.
- With `alternate` ordering, odd repeats run A->B and even repeats run B->A.
