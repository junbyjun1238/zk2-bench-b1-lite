# Minimal Halo2 Integration Example

This package is still primarily a benchmark and verification artifact, but it now
includes a small Halo2-facing integration surface that a third party can invoke
without going through the benchmark runners.

## What is included

- public helper API in `src/integration.rs`
- runnable example at `examples/halo2_integration_demo.rs`
- reusable constructors:
  - `build_a_secure_circuit(...)`
  - `build_b_note_circuit(...)`
- simple verification helpers:
  - `verify_a_secure_mock(...)`
  - `verify_b_note_mock(...)`
  - `verify_mock(...)`
- public metadata for the current contract:
  - recommended `k`
  - per-repetition row count
  - logical lookup / multiplicative / linear counts

## Why this exists

The benchmark package already contained real Halo2 circuits, but it was shaped as
an artifact/reproduction bundle. This example is the minimal proof that the
released `B_note` path can be consumed as a Halo2-facing library surface rather
than only as a benchmark runner.

This is still **not** a full external codebase integration. It is a minimal
reference integration path inside the released package.

## Run the example

```bash
cargo run --example halo2_integration_demo
```

Expected output includes:

- the recommended `k` for the `B_note` example path,
- the current per-repetition row and logical-count metadata,
- a successful `MockProver` verification on the `boundary` profile.

## Example downstream usage

```rust
use zcg_ab_bench::integration::{build_b_note_circuit, verify_b_note_mock};
use zcg_ab_bench::shared_inputs::InputProfile;

let circuit = build_b_note_circuit(1, InputProfile::Boundary);
verify_b_note_mock(1, InputProfile::Boundary).unwrap();
```

## Claim boundary

This example only demonstrates a minimal Halo2-facing integration path for the
released `B_note` circuit surface.

It does **not** by itself establish:

- backend closure,
- full wrapper-level parity,
- PCS / Fiat-Shamir soundness,
- integration against an external third-party codebase.
