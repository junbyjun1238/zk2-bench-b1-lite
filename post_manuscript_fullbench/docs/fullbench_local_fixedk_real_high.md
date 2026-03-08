# Local Full-Bench (Fixed-k) Report

- mode: `full-local`
- k_run_fixed: `13`
- row_capacity(2^k): `8192`
- scales: `176,208,240`

## Point Table

| scale | occupancy | bucket | A prove(ms) | B prove(ms) | B/A prove | A verify(ms) | B verify(ms) | B/A verify |
|---:|---:|---|---:|---:|---:|---:|---:|---:|
| 176 | 0.623 | mid-high(60-90%) | 11596.028 | 8877.951 | 0.766 | 41.279 | 26.000 | 0.630 |
| 208 | 0.736 | mid-high(60-90%) | 11814.189 | 8954.304 | 0.758 | 42.015 | 25.985 | 0.618 |
| 240 | 0.850 | mid-high(60-90%) | 11977.425 | 9147.894 | 0.764 | 42.828 | 26.779 | 0.625 |

## Bucket Summary

| bucket | points | scale range | avg B/A prove | avg B/A verify | avg B/A rss |
|---|---:|---:|---:|---:|---:|
| mid-high(60-90%) | 3 | 176-240 | 0.762 | 0.625 | 0.332 |

Interpretation rule:
- If `B/A prove` consistently decreases or stays < 1 across low+mid buckets, local-only trend is considered meaningful.
- If trend is unstable across low+mid buckets, escalate to cloud/high-scale extension.
