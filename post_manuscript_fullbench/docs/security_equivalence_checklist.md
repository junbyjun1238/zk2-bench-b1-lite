# Security Equivalence Status (Current Public Package)

This note records the current status of the public `A_secure` vs `B_note`
comparison package. It is meant to show which equivalence checks are already
implemented and exercised in the released bounded instantiated-family
benchmark path.

## Common Security-Equivalence Checks

- [x] Canonical residue enforcement  
  The released circuits bind designated residue-style values into their intended
  canonical ranges in the measured path.

- [x] Quotient/carry range binding  
  Quotient and carry witnesses are bound to their declared classes in the
  released `A_secure` and `B_note` constructions.

- [x] No-wrap interval guarantees  
  The public manuscript, certificates, and checker path expose no-wrap
  assumptions explicitly, and the benchmark package treats them as part of the
  defended bounded-comparison contract.

- [x] Wiring/inventory completeness  
  The public checker and related fixtures validate quotient/carry inventory and
  reject missing or malformed wiring information.

## Negative-Case Rejection

- [x] 1) `p^{-1}` witness toy attack  
  The negative-case suite rejects the toy vacuous witness attack in the public
  verification path.

- [x] 2) omitted quotient wiring  
  Deliberately omitted quotient wiring is rejected.

- [x] 3) class-map mismatch (31/66)  
  Incorrect 31/66 class-map tagging is rejected.

- [x] 4) digest mismatch  
  Certificate/manuscript/backend digest mismatches are rejected.

- [x] 5) inactive-row zero-extension violation  
  Inactive-row zero-extension violations are rejected.

## Operational Status

- [x] The public A/B runs use the same machine-readable output schema in
  `docs/results_schema.json`.
- [x] Failure-oriented runs surface their outcome through `status` and `notes`
  fields rather than silent divergence.

## Evidence Pointers

- Current checker entry point: `scripts/check_public_certificate.py`
- Current benchmark runner: `scripts/run_ab_bench.py`
- Current parity-facing reruns: `docs/repeat_boundary_k17_small.md`,
  `docs/repeat_standard_k17_small.md`, `docs/repeat_boundary_k18_s4520_r3.md`
