# Local Full-Bench (Fixed-k) Report

- mode: `full-local`
- k_run_fixed: `13`
- row_capacity(2^k): `8192`
- scales: `2,4,8,16,24,32,48,64,80,96,128,160`

## Point Table

| scale | occupancy | bucket | A prove(ms) | B prove(ms) | B/A prove | A verify(ms) | B verify(ms) | B/A verify |
|---:|---:|---|---:|---:|---:|---:|---:|---:|
| 2 | 0.007 | low(<25%) | 34.870 | 21.277 | 0.610 | 146.847 | 50.388 | 0.343 |
| 4 | 0.014 | low(<25%) | 54.891 | 28.325 | 0.516 | 149.828 | 53.459 | 0.357 |
| 8 | 0.028 | low(<25%) | 91.027 | 42.200 | 0.464 | 152.760 | 58.085 | 0.380 |
| 16 | 0.057 | low(<25%) | 167.382 | 73.233 | 0.438 | 174.265 | 65.871 | 0.378 |
| 24 | 0.085 | low(<25%) | 246.806 | 100.834 | 0.409 | 177.810 | 74.562 | 0.419 |
| 32 | 0.113 | low(<25%) | 318.238 | 132.084 | 0.415 | 188.429 | 81.389 | 0.432 |
| 48 | 0.170 | low(<25%) | 472.063 | 186.812 | 0.396 | 203.440 | 97.065 | 0.477 |
| 64 | 0.227 | low(<25%) | 634.743 | 251.934 | 0.397 | 220.843 | 113.546 | 0.514 |
| 80 | 0.283 | mid(25-60%) | 779.574 | 311.495 | 0.400 | 239.846 | 129.224 | 0.539 |
| 96 | 0.340 | mid(25-60%) | 934.718 | 359.008 | 0.384 | 263.075 | 150.086 | 0.571 |
| 128 | 0.453 | mid(25-60%) | 1228.503 | 489.795 | 0.399 | 299.597 | 176.189 | 0.588 |
| 160 | 0.566 | mid(25-60%) | 1556.686 | 599.296 | 0.385 | 339.344 | 210.956 | 0.622 |

## Bucket Summary

| bucket | points | scale range | avg B/A prove | avg B/A verify | avg B/A rss |
|---|---:|---:|---:|---:|---:|
| low(<25%) | 8 | 2-64 | 0.455 | 0.413 | 0.267 |
| mid(25-60%) | 4 | 80-160 | 0.392 | 0.580 | 0.222 |

Interpretation rule:
- If `B/A prove` consistently decreases or stays < 1 across low+mid buckets, local-only trend is considered meaningful.
- If trend is unstable across low+mid buckets, escalate to cloud/high-scale extension.
