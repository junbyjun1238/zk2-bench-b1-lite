# Post-manuscript full-benchmark package

This directory contains the follow-up benchmark package produced after the manuscript draft.

Scope:
- `A_secure` vs `B_note` under the shared benchmark contract
- fixed-`k` local full benchmarks
- repeat-based public timing collection
- external comparison against the `halo2wrong` decomposition baseline
- benchmark reports and raw JSON outputs

## Reviewer-facing scripts

These are the current tracked scripts intended for audit and re-execution:

- `scripts/run_ab_bench.py`
  - single arm / single scale runner producing one JSON output
- `scripts/local_repeat_bench.py`
  - repeat-based headline timing collector (`A_secure` vs `B_note`)
- `scripts/local_fixedk_fullbench.py`
  - fixed-`k` local sweep + occupancy-bucket report generator
- `scripts/run_external_compare.py`
  - external `halo2wrong` comparison runner (on-demand only; not a tracked public evidence bundle)
- `scripts/local_sweep.py`
  - lower-level sweep helper used by the fixed-`k` report

The script set above matches the latest workspace versions used for the current benchmark package.
Exploratory fixed-`k` output snapshots from earlier iterations were removed from the tracked public tree.
Reviewer-facing timing runs now use release binaries, not debug binaries.
The `full-cloud` mode is intentionally disabled; cloud machines should execute the same `full-local` path.
Shared-input profiles are available via `--input-profile {standard,boundary,adversarial}`; `boundary` is the recommended first parity-facing profile.

## Canonical public headline evidence

Prefer these files for the defended public timing claim:

- `docs/repeat_local_k13_public.md`
- `benches/repeat_local_k13_public/summary.json`

Older timing reports are not part of the tracked public tree for this package.

## Latest parity-facing follow-up evidence

For the newer shared-input parity-facing reruns, start with:

- `docs/repeat_boundary_k17_small.md`
- `docs/repeat_standard_k17_small.md`
- `docs/repeat_boundary_k17_medium.md`
- `docs/repeat_standard_k17_medium.md`
- `docs/repeat_boundary_k17_s16.md`
- `docs/repeat_standard_k17_s16.md`
- `docs/repeat_boundary_k17_s32.md`
- `docs/repeat_standard_k17_s32.md`
- `docs/repeat_boundary_k18_s4520.md`
- `docs/k_jump_capacity_sweep.md`

These are follow-up benchmark artifacts for the shared-input, fixed-`k` parity-preparation path. They should still be read under the bounded instantiated-family claim boundary described in the manuscript and package notes.

## Contents

- Rust benchmark project:
  - `Cargo.toml`, `Cargo.lock`
  - `src/`
- Python orchestration scripts:
  - `scripts/run_ab_bench.py`
  - `scripts/local_repeat_bench.py`
  - `scripts/local_fixedk_fullbench.py`
  - `scripts/local_sweep.py`
  - `scripts/run_external_compare.py`
  - `scripts/scale_search.py`
  - `scripts/plot_fullbench.py`
- Benchmark inputs / checks:
  - `certificates/public_certificate.json`
  - `certificates/h2dq_backend_instance.json`
  - `docs/results_schema.json`
  - `docs/security_equivalence_checklist.md`
- Reports / outputs:
  - `docs/repeat_local_k13_public.md`
  - `benches/repeat_local_k13_public/`

## Reproduction

Headline repeat benchmark:

```bash
python scripts/local_repeat_bench.py --scales 16,24,32 --k-run 13 --repeats 2 --out-dir benches/repeat_local_k13_public --out-md docs/repeat_local_k13_public.md
```

Shared-input boundary smoke rerun:

```bash
python scripts/local_repeat_bench.py --scales 1 --k-run 17 --repeats 1 --input-profile boundary --out-dir benches/tmp_repeat_boundary --out-md docs/tmp_repeat_boundary.md
```

Fixed-k local sweep report:

```bash
python scripts/local_fixedk_fullbench.py --k-run 13 --scales 16,24,32 --out-dir benches/fullbench_local_fixedk_current --out-md docs/fullbench_local_fixedk_current.md
```

This command generates a fresh fixed-k sweep snapshot on demand. It is not a tracked canonical public output.

External comparison:

```bash
python scripts/run_external_compare.py --k-run 17 --scales 8,16 --out-dir benches/external_h2w_compare --report docs/external_h2w_compare.md --require-cert
```

This script remains available for supplementary use, but its outputs are not tracked as part of the current public evidence bundle.

Notes:
- These results were produced after the manuscript draft and should be read as follow-up implementation evidence, not as part of the theorem claim in the paper.
- The safest wording is: `bounded instantiated family comparison`, not a claim of full-domain semantic equivalence across all wrapper realizations.
- The manuscript-pinned artifact package remains the root-level package in this repository.
