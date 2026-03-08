# Local Full-Bench (Fixed-k) Report

- mode: `full-local`
- k_run_fixed: `13`
- row_capacity(2^k): `8192`
- scales: `16,24,32`

## Point Table

| scale | occupancy | bucket | A prove(ms) | B prove(ms) | B/A prove | A verify(ms) | B verify(ms) | B/A verify |
|---:|---:|---|---:|---:|---:|---:|---:|---:|
| 16 | 0.057 | low(<25%) | 10571.951 | 8042.712 | 0.761 | 41.020 | 25.729 | 0.627 |
| 24 | 0.085 | low(<25%) | 10719.314 | 8100.761 | 0.756 | 41.533 | 25.840 | 0.622 |
| 32 | 0.113 | low(<25%) | 10738.378 | 8209.514 | 0.765 | 41.269 | 26.280 | 0.637 |

## Bucket Summary

| bucket | points | scale range | avg B/A prove | avg B/A verify | avg B/A rss |
|---|---:|---:|---:|---:|---:|
| low(<25%) | 3 | 16-32 | 0.760 | 0.629 | 0.334 |

Interpretation rule:
- If `B/A prove` consistently decreases or stays < 1 across low+mid buckets, local-only trend is considered meaningful.
- If trend is unstable across low+mid buckets, escalate to cloud/high-scale extension.
