# Local A/B Sweep (full-local)

| scale | A_prove_ms | B_prove_ms | B/A prove | A_verify_ms | B_verify_ms | B/A verify | A_peak_rss_mb | B_peak_rss_mb | B/A rss |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 176 | 11596.028 | 8877.951 | 0.766 | 41.279 | 26.000 | 0.630 | 209.121 | 69.293 | 0.331 |
| 208 | 11814.189 | 8954.304 | 0.758 | 42.015 | 25.985 | 0.618 | 209.305 | 69.465 | 0.332 |
| 240 | 11977.425 | 9147.894 | 0.764 | 42.828 | 26.779 | 0.625 | 209.273 | 69.422 | 0.332 |

Notes:
- Both arms use the same `run_ab_bench.py` contract and schema.
- Both arms enforce the same row-family semantics.
- `A_secure` uses explicit bit decomposition; `B_note` uses lookup-assisted binding.
- This sweep is local-only (`full-local`) and intended for operational-fit baseline.
