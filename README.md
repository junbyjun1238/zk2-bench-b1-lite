# BN254 Exactness Companion Artifact and Benchmark Evidence

This MIT-licensed repository is the public base for the BN254 deferred-quotient
exactness repair work. It serves two related but distinct roles:

- the **manuscript-pinned companion artifact** for the original repair note,
  including checker inputs, backend manifests, and manuscript-side schema
  materials, and
- the **benchmark-evidence archive** for the released follow-up Halo2 reruns
  under `post_manuscript_fullbench/`.

If you are approaching this work as developer-facing Halo2 tooling, use the
standalone tooling repository instead:

- `https://github.com/junbyjun1238/halo2-exactness-tooling`

## Current Public Assets

- Paper PDF:
  - `deferred-quotient-vacuity-in-bn254-wrappers-a-repair-note-on-algebraic-exactness.pdf`
- Public release:
  - `v0.1.0`
- DOI-backed archive:
  - Zenodo DOI `10.5281/zenodo.18910795`
- Open-source license:
  - `MIT` (`LICENSE`)
- Follow-up benchmark package:
  - `post_manuscript_fullbench/`

## Claim Boundary

The paper is a theorem-level repair note for a concrete BN254/M31 family. The follow-up benchmark materials in this repository should be read under the same disciplined boundary:

- the benchmark evidence is a **bounded instantiated-family comparison**,
- it is **not** a claim of universal superiority across all wrapper realizations,
- it is **not** a claim of backend closure or Fiat-Shamir closure.

## Repository Layout

### 1. Benchmark-evidence archive

- `post_manuscript_fullbench/README.md`
- `post_manuscript_fullbench/scripts/`
- `post_manuscript_fullbench/docs/`

It contains:

- release-binary Halo2 benchmark runners,
- shared-input parity-facing reruns,
- first post-jump validation,
- provenance / capacity notes,
- raw JSON outputs and reviewer-facing scripts.

### 2. Manuscript-pinned companion artifact

This is the root-level package cited by the manuscript for certificate/checker/manuscript materials:

- `wrapper_note_option2.tex`
- `certificates/public_certificate.json`
- `certificates/h2dq_backend_instance.json`
- `scripts/check_public_certificate.py`
- `artifact_schema_supplement.md`

The manuscript cites the pinned artifact tree at:

- `2b1b28f7216bd85c75108caefc45dd1e437be061`

That pinned tree matters for the theorem companion materials. The repository has since grown additional public benchmark and release assets, but the manuscript-pinned artifact reference remains the root-level source of truth for the original companion package.

## Current Benchmark Evidence

For the current public benchmark narrative, prefer the shared-input follow-up reports in `post_manuscript_fullbench/docs/`.

The most important public follow-up evidence is:

- shared-input repeated reruns at `k_run = 17` for `scale = 1,4,8,16,32`,
- the first post-jump repeated validation at `scale = 4520`, `k_run = 18`, `repeats = 3`,
- `k_jump_capacity_sweep.md`,
- `capacity_frontier_scope.md`,
- `provenance_note.md`.

The older `repeat_local_k13_public.md` report is still useful as an earlier canonical repeat table, but it should no longer be read as the whole public story by itself. The newer shared-input parity-facing reports are the stronger public evidence bundle.

## Reproduction

### Root-level companion checks

```bash
python scripts/check_public_certificate.py \
  --certificate certificates/public_certificate.json \
  --manuscript wrapper_note_option2.tex \
  --backend-instance certificates/h2dq_backend_instance.json
```

### Follow-up benchmark package

```bash
cd post_manuscript_fullbench
python scripts/local_repeat_bench.py --scales 1,4 --k-run 17 --repeats 3 --input-profile boundary --out-dir benches/repeat_boundary_k17_small --out-md docs/repeat_boundary_k17_small.md
```

For fuller benchmark commands and report entry points, see:

- `post_manuscript_fullbench/README.md`

## Notes

- This repository is MIT-licensed. See `LICENSE`.
- The benchmark package uses release binaries, not debug binaries.
- `full-cloud` is intentionally disabled in the public harness; cloud machines should execute the same `full-local` path.
- External comparison scripts remain available for supplementary use, but they are not the core public evidence bundle.
- The benchmark materials are already publicly released; future engineering work should be read as tooling/productization work built on top of this base, not as funding for generating the already-published evidence.
