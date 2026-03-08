# Post-manuscript full-benchmark package

This directory contains the follow-up full local benchmark package used after the manuscript draft.

Scope:
- `A_secure` vs `B_note` under the shared benchmark contract
- fixed-`k` local full benchmarks
- external comparison against the `halo2wrong` decomposition baseline
- benchmark reports and raw JSON outputs

Canonical public headline evidence in this package:
- `docs/repeat_local_k13_public.md`
- `benches/repeat_local_k13_public/summary.json`

These files are the preferred public source for the headline timing ratio claim.
Older timing reports in this package should be treated as exploratory snapshots,
not as the canonical public headline table.

## Contents

- Rust benchmark project:
  - `Cargo.toml`, `Cargo.lock`
  - `src/`
- Python orchestration scripts:
  - `scripts/run_ab_bench.py`
  - `scripts/local_sweep.py`
  - `scripts/local_fixedk_fullbench.py`
  - `scripts/run_external_compare.py`
  - `scripts/scale_search.py`
- Benchmark inputs / checks:
  - `certificates/public_certificate.json`
  - `certificates/h2dq_backend_instance.json`
  - `docs/results_schema.json`
  - `docs/security_equivalence_checklist.md`
- Reports / outputs:
  - `docs/repeat_local_k13_public.md`
  - `benches/repeat_local_k13_public/`
  - `docs/fullbench_local_fixedk*.md`
  - `docs/external_h2w_compare.md`
  - `benches/fullbench_local_fixedk*`
  - `benches/external_h2w_compare`

## Reproduction

Fixed-k local full bench:

```bash
python scripts/local_fixedk_fullbench.py --k-run 13 --scales 16,24,32 --out-dir benches/fullbench_local_fixedk_real_mid --out-md docs/fullbench_local_fixedk_real_mid.md
```

External comparison:

```bash
python scripts/run_external_compare.py --k-run 17 --scales 8,16 --out-dir benches/external_h2w_compare --report docs/external_h2w_compare.md --require-cert
```

Notes:
- These results were produced after the manuscript draft and should be read as follow-up implementation evidence, not as part of the theorem claim in the paper.
- The safest wording is: `bounded instantiated family comparison`, not a claim of full-domain semantic equivalence across all wrapper realizations.
- The manuscript-pinned artifact package remains the root-level package in this repository.
