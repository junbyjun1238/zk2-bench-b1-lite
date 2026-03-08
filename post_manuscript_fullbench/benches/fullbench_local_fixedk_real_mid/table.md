# Local A/B Sweep (full-local)

| scale | A_prove_ms | B_prove_ms | B/A prove | A_verify_ms | B_verify_ms | B/A verify | A_peak_rss_mb | B_peak_rss_mb | B/A rss |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 16 | 10571.951 | 8042.712 | 0.761 | 41.020 | 25.729 | 0.627 | 208.113 | 69.383 | 0.333 |
| 24 | 10719.314 | 8100.761 | 0.756 | 41.533 | 25.840 | 0.622 | 208.238 | 69.316 | 0.333 |
| 32 | 10738.378 | 8209.514 | 0.765 | 41.269 | 26.280 | 0.637 | 208.262 | 69.691 | 0.335 |

Notes:
- Both arms use the same `run_ab_bench.py` contract and schema.
- Both arms enforce the same row-family semantics.
- `A_secure` uses explicit bit decomposition; `B_note` uses lookup-assisted binding.
- This sweep is local-only (`full-local`) and intended for operational-fit baseline.
