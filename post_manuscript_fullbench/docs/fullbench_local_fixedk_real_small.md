# Local Full-Bench (Fixed-k) Report

- mode: `full-local`
- k_run_fixed: `13`
- row_capacity(2^k): `8192`
- scales: `2,4,8`

## Point Table

| scale | occupancy | bucket | A prove(ms) | B prove(ms) | B/A prove | A verify(ms) | B verify(ms) | B/A verify |
|---:|---:|---|---:|---:|---:|---:|---:|---:|
| 2 | 0.007 | low(<25%) | 10425.074 | 7868.668 | 0.755 | 41.639 | 26.028 | 0.625 |
| 4 | 0.014 | low(<25%) | 10504.943 | 7827.204 | 0.745 | 41.663 | 25.833 | 0.620 |
| 8 | 0.028 | low(<25%) | 10503.283 | 8830.261 | 0.841 | 41.407 | 25.851 | 0.624 |

## Bucket Summary

| bucket | points | scale range | avg B/A prove | avg B/A verify | avg B/A rss |
|---|---:|---:|---:|---:|---:|
| low(<25%) | 3 | 2-8 | 0.780 | 0.623 | 0.333 |

Interpretation rule:
- If `B/A prove` consistently decreases or stays < 1 across low+mid buckets, local-only trend is considered meaningful.
- If trend is unstable across low+mid buckets, escalate to cloud/high-scale extension.
