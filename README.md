# B1-Lite Benchmark Harness

This artifact provides a reproducible Halo2 circuit-shape comparison used by the wrapper repair note.

## Scope

- Backend: Halo2 (`halo2_proofs 0.3.2`)
- Comparison type: structural circuit-shape comparison (not task-equated, not prover-time benchmark)
- Baselines:
  - Repair fragment (promoted-Horner binding fragment)
  - Repair paired package (fragment + carry-normalization q31 add-on)
  - B1-lite schoolbook wrong-field baseline (16x16 limb cross-products + carry chain)

## Repository contents

- `src/main.rs`: reproducible harness and printed metrics
- `results.md`: current measured numbers and ratios
- `Cargo.toml`, `Cargo.lock`: pinned dependencies for reproduction

## Reproduction

```powershell
Set-Location "C:\Users\parks\Desktop\새 폴더\zk2\bench_b1_lite"
cargo run --release
```

## Notes

- `B1-lite` excludes modular reduction and CRT-consistency checks.
- Therefore this baseline is a conservative partial baseline for the implemented schoolbook-limb wrong-field family.
