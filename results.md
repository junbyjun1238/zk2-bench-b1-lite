# Results (current)

## Environment

- OS: Windows (local workspace machine)
- Rust: `rustc 1.89.0`, `cargo 1.89.0`
- Crates: `halo2_proofs 0.3.2`, `halo2curves 0.9.0`

## Circuits

- Repair fragment: promoted-Horner binding fragment only.
- Repair B2 paired package: fragment + carry-normalization add-on (`4` q31 cells).
- B1-lite baseline: 256-bit schoolbook mul (16x16 limbs) with explicit cross-products + carry chain.

## Numbers

| Item | Metric | Value | Notes |
|---|---:|---:|---|
| Repair fragment | rows | 72 | 44 residue + 12 q66 + 12 boolean + 4 u8 |
| Repair fragment | lookup cells | 152 | Matches paper accounting |
| Repair fragment | multiplicative constraints | 56 | 44 non-equality + 12 booleanity |
| Repair fragment | linear constraints | 56 | 44 residue recomposition + 12 q66 recomposition |
| Repair B2 paired package | rows | 76 | fragment + 4 q31 carry-normalization cells |
| Repair B2 paired package | lookup cells | 160 | fragment + 8 lookup cells |
| Repair B2 paired package | multiplicative constraints | 56 | unchanged from fragment |
| Repair B2 paired package | linear constraints | 60 | fragment + 4 q31 recomposition constraints |
| B1-lite baseline | rows | 352 | 16x16 cross-product rows + carry/range rows |
| B1-lite baseline | lookup cells | 126 | `T16` limbs + `T15/T5` carry decomposition |
| B1-lite baseline | multiplicative constraints | 256 | one multiplication constraint per cross-product |
| B1-lite baseline | linear constraints | 63 | carry recomposition + carry-chain equations |

## Ratio (B1-lite / Repair)

- versus fragment:
  - rows: `4.89x`
  - lookup cells: `0.83x`
  - multiplicative constraints: `4.57x`
  - linear constraints: `1.12x`
- versus B2 paired package:
  - rows: `4.63x`
  - lookup cells: `0.79x`
  - multiplicative constraints: `4.57x`
  - linear constraints: `1.05x`

## Scope note

- `B1-lite` intentionally excludes modular reduction and CRT-consistency checks.
- So this is a conservative partial baseline for the implemented schoolbook-limb wrong-field family.

## Reproduction

```powershell
Set-Location "C:\Users\parks\Desktop\새 폴더\zk2\bench_b1_lite"
cargo run --release
```
