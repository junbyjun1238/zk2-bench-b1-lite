# Artifact Schema Supplement (Implementation Metadata)

This supplement records one concrete, machine-verifiable serialization/digest
schema for the public synthesis certificates referenced by
`wrapper_note_option2.tex`.

It is **optional implementation metadata** and is not required for the algebraic
exactness theorem proved in the note.

## Per-Family Records

For each row family `f` in `{inv, EF, EE, hor, car}`, publish a record:

`Rec(f) = (id_f, classBits_f, boundDecL_f, boundDecD_f, selectorTag_f, coeffLDigest_f, coeffDDigest_f, intervalDigest_f)`

Field meanings:
- `id_f`: one of `inv`, `EF`, `EE`, `hor`, `car`
- `classBits_f`: `31` or `66` (the declared family class)
- `boundDecL_f`: decimal encoding of `U_f^audit` (family max for `L`)
- `boundDecD_f`: decimal encoding of `U_f^{D,audit}` (family max for `D`)
- `selectorTag_f`: canonical selector-family tag used by selector wiring certificates
- `coeffLDigest_f`: digest committing to normalized left-template coefficients for family `f`
- `coeffDDigest_f`: digest committing to normalized right-template coefficients for family `f`
- `intervalDigest_f`: digest committing to certified per-variable input intervals for family `f`

## Global Digests

Additionally publish global digests:
- `templateLibDigest`
- `selectorMapQDigest`
- `selectorMapNQDigest`
- `selectorMapRDigest`
- `rowActivityDigest`

These commit to the template library version and the selector maps / row-activity
metadata used by the compiler/synthesizer.

## Deterministic Validation Rules

An artifact is valid only if:
1. Exactly one record exists per family id.
2. `classBits_f` matches the declared family class for `f`.
3. `coeffLDigest_f`, `coeffDDigest_f`, and `intervalDigest_f` match the fixed template library indicated by `templateLibDigest`.
4. `boundDecL_f` and `boundDecD_f` equal the deterministic compiler outputs on that fixed library, and satisfy the public no-wrap/headroom checks:
   - `boundDecL_f < p * B_f + p`
   - `boundDecD_f + p * (B_f - 1) < r`
5. The selector maps are total/functional on active rows and consistent with the wiring/realization/coherence certificates (quotient selector, residue/carry selector, relation realization, template assignment).
6. Row-activity metadata is total on `(t,j)` and enforces zero extension on inactive entries.

