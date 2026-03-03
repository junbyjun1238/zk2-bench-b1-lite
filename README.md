# B1-Lite Benchmark Harness + Companion Artifact Package

This repository contains:
- a reproducible Halo2 circuit-shape comparison harness used in the note, and
- the companion public artifact package referenced by `wrapper_note_option2.tex`
  (`certificate/checker/manifest/manuscript`).

## Scope

- Backend: Halo2 (`halo2_proofs 0.3.2`)
- Comparison type: structural circuit-shape comparison (not task-equated, not prover-time benchmark)
- Baselines:
  - Repair fragment (promoted-Horner binding fragment)
  - Repair B2 paired package (fragment + carry-normalization q31 add-on)
  - B1-lite schoolbook wrong-field baseline (16x16 limb cross-products + carry chain)

## Repository Contents

- Benchmark harness:
  - `src/main.rs`
  - `results.md`
  - `Cargo.toml`, `Cargo.lock`
- Companion artifact package:
  - `wrapper_note_option2.tex`
  - `certificates/public_certificate.json`
  - `certificates/h2dq_backend_instance.json`
  - `scripts/check_public_certificate.py`
  - `artifact_schema_supplement.md`

## Reproduction

1) Benchmark harness:

```bash
cargo run --release
```

2) Certificate + pinned-backend manifest checks:

```bash
python scripts/check_public_certificate.py \
  --certificate certificates/public_certificate.json \
  --manuscript wrapper_note_option2.tex \
  --backend-instance certificates/h2dq_backend_instance.json
```

## Notes

- `B1-lite` excludes modular reduction and CRT-consistency checks.
- Therefore this baseline is a conservative partial baseline for the implemented schoolbook-limb wrong-field family.

## Public-Goods Follow-Up Roadmap (Grant-Oriented)

This note can be extended into open-source public goods primarily through
engineering/productization work (not new core algebra). The current checker is
treated as a research artifact baseline (PoC), and grant milestones start from
public-good hardening work rather than re-billing existing code.

### M0 (Completed, non-funded): Research Artifact Baseline

Deliverables:
- public checker/certificate/manuscript binding in the current repository.

Exit criteria:
- baseline reproducibility from a pinned commit.

Payment trigger:
- 0 percent (already completed before the grant period).

### M1-1 (1 week): Spec Freeze

Deliverables:
- `public_certificate` schema `v1.0.0`,
- field-level specification and compatibility policy (semver).

Exit criteria:
- checker and schema docs reference the same tagged version.

Payment trigger:
- 7 percent.

### M1-2 (1 week): Deterministic Repro Pack

Deliverables:
- one-command deterministic environment (lockfile/containerized setup).

Exit criteria:
- identical verification outputs from a clean machine run.

Payment trigger:
- 6 percent.

### M1-3 (1 week): Negative-Test Corpus

Deliverables:
- adversarial/failure fixtures (inventory omission, class-map mismatch, digest mismatch).

Exit criteria:
- checker fails deterministically on each injected defect case.

Payment trigger:
- 6 percent.

### M1-4 (1 week): Third-Party Replay

Deliverables:
- independent external replay report/log.

Exit criteria:
- external runner reproduces success and failure cases using only published docs.

Payment trigger:
- 6 percent.

### M2 (4 weeks): Reusable Library (`h2dq-repair`)

Deliverables:
- library crate exposing canonical residue, quotient (`q31`/`q66`), and carry bindings,
- API docs, examples, tests, and CI.

Exit criteria:
- external user can instantiate the repair fragment without editing internals.

Payment trigger:
- 30 percent.

### M3 (4 weeks): Tooling (`h2dq-cert` and `h2dq-lint`)

Deliverables:
- certificate generation/validation CLI,
- lint/compile pass for deferred-quotient coverage and class-map integrity.

Exit criteria:
- automated detection of injected wiring/coverage/certificate faults.

Payment trigger:
- 25 percent.

### M4 (3 weeks): Backend Integration and Public Release

Deliverables:
- Halo2 integration adapter package,
- reproducible benchmark scripts,
- `v1.0.0` tagged release and migration guide.

Exit criteria:
- independent end-to-end installation and run from README only.

Payment trigger:
- 20 percent.
