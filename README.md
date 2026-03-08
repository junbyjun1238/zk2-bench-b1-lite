# B1-Lite Benchmark Harness + Companion Artifact Package

This repository contains:
- a reproducible Halo2 circuit-shape comparison harness used in the note, and
- the companion public artifact package referenced by `wrapper_note_option2.tex`
  (`certificate/checker/manifest/manuscript`).

For the exact public artifact tree cited by the manuscript, use the pinned
commit:
`2b1b28f7216bd85c75108caefc45dd1e437be061`.

Follow-up full local benchmark materials produced after the manuscript draft
are available under:
`post_manuscript_fullbench/`

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
- Post-manuscript full benchmark package:
  - `post_manuscript_fullbench/`
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
