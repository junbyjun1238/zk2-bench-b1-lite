# Local A/B Sweep (full-local)

| scale | A_prove_ms | B_prove_ms | B/A prove | A_verify_ms | B_verify_ms | B/A verify | A_peak_rss_mb | B_peak_rss_mb | B/A rss |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 2 | 10425.074 | 7868.668 | 0.755 | 41.639 | 26.028 | 0.625 | 208.078 | 69.051 | 0.332 |
| 4 | 10504.943 | 7827.204 | 0.745 | 41.663 | 25.833 | 0.620 | 208.129 | 69.539 | 0.334 |
| 8 | 10503.283 | 8830.261 | 0.841 | 41.407 | 25.851 | 0.624 | 208.770 | 69.629 | 0.334 |

Notes:
- Both arms use the same `run_ab_bench.py` contract and schema.
- Both arms enforce the same row-family semantics.
- `A_secure` uses explicit bit decomposition; `B_note` uses lookup-assisted binding.
- This sweep is local-only (`full-local`) and intended for operational-fit baseline.
