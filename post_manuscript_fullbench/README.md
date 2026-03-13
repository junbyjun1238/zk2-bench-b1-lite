# Post-manuscript Halo2 verification and benchmark package

This MIT-licensed directory contains the developer-facing follow-up package
produced after the manuscript draft. It combines the public benchmark harness,
the parity-facing evidence bundle, and a small Halo2-facing integration
surface for the released circuits, including a reusable real prove/verify
adapter for the public proof path.

Scope:
- `A_secure` vs `B_note` under the shared benchmark contract
- fixed-`k` local full benchmarks
- repeat-based public timing collection
- external comparison against the `halo2wrong` decomposition baseline
- minimal Halo2-facing integration surface for the released `B_note` path,
  including a reusable real prove/verify adapter
- benchmark reports and raw JSON outputs

If you are evaluating this package as tooling rather than only as a benchmark
bundle, start with:

- `src/integration.rs`
- `examples/halo2_integration_demo.rs`
- `docs/halo2_integration_example.md`

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

## Minimal Halo2 integration surface

This package now also exposes a small Halo2-facing integration layer for the
released benchmark circuits:

- `src/integration.rs`
- `examples/halo2_integration_demo.rs`
- `docs/halo2_integration_example.md`

This is intentionally modest. It does not claim an external production
integration yet. It is a minimal reference path showing that the released
`B_note` circuit can be instantiated and verified from a library-facing surface
instead of only through benchmark runners, and that the same public
`create_proof` / `verify_proof` path can be reached from that surface.

## Canonical public headline evidence

Prefer the shared-input follow-up reports below for the current defended public
timing claim. The older `repeat_local_k13_public.md` table remains archived as
an earlier reference snapshot, but it is no longer the main public entry point.

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
- `docs/repeat_boundary_k18_s4520_r3.md`
- `docs/k_jump_capacity_sweep.md`
- `docs/capacity_frontier_scope.md`
- `docs/provenance_note.md`

These are follow-up benchmark artifacts for the shared-input, fixed-`k` parity-preparation path. They should still be read under the bounded instantiated-family claim boundary described in the manuscript and package notes.
The capacity-frontier memo is harness-specific, not a universal production-sizing recommendation. The provenance note explains why the raw run JSON commit hashes are grouped by generation commit rather than matching the final publishing tree HEAD.

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
  - `docs/halo2_integration_example.md`

## Reproduction

Headline repeat benchmark:

```bash
python scripts/local_repeat_bench.py --scales 16,24,32 --k-run 13 --repeats 2 --out-dir benches/repeat_local_k13_public --out-md docs/repeat_local_k13_public.md
```

Shared-input boundary smoke rerun:

```bash
python scripts/local_repeat_bench.py --scales 1 --k-run 17 --repeats 1 --input-profile boundary --out-dir benches/tmp_repeat_boundary --out-md docs/tmp_repeat_boundary.md
```

Minimal Halo2 integration demo:

```bash
cargo run --example halo2_integration_demo
```

This example performs both a `MockProver` check and a real proof/verification
cycle, so it is intentionally slower than the lightweight smoke tests.

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
- This package is MIT-licensed through the repository root `LICENSE`.
- These results were produced after the manuscript draft and should be read as follow-up implementation evidence, not as part of the theorem claim in the paper.
- The safest wording is: `bounded instantiated family comparison`, not a claim of full-domain semantic equivalence across all wrapper realizations.
- The manuscript-pinned artifact package remains the root-level package in this repository.
