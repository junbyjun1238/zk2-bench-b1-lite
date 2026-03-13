# Wrapper-Level Parity Status for the Current Public Reruns

This note records the wrapper-level parity conditions satisfied by the current
public reruns. The purpose is to show that the published `A_secure` vs
`B_note` comparisons are using the same measured proof path rather than
silently comparing different backend surfaces.

The immediate goal is not a full backend-soundness theorem. The goal is
narrower:

- prevent the current public reruns from being attacked as a comparison of
  different proof paths rather than different circuit encodings.

## Current satisfied conditions

### W0. Same proving / verification API path

The current published reruns send both arms through the same proving and
verification path exposed by the harness:

- same `full-local` runner mode,
- same `create_proof` / `verify_proof` backend path,
- same transcript type,
- same commitment backend,
- same parameter family.

In the current harness this means:

- same KZG backend,
- same Blake2b transcript family,
- same local proof runner structure.

### W1. Same public-input routing contract

If either arm changes how public inputs are routed into the circuit or proof
invocation, the benchmark stops being apples-to-apples.

The current public reruns use:

- the same number of instance columns,
- the same empty/non-empty public-input shape for the measured path,
- the same proof-call argument structure.

### W2. Same fixed-k measurement contract

For each reported comparison point in the public package:

- both arms run under the same declared `k_run`,
- both arms expose the same `k_min` discovery semantics,
- both arms report those values in the same output schema.

### W3. Same runner semantics and output schema

The public package does not compare:

- one arm measured in a structural-only or mocked path,
- against another arm measured in a real proving path.

Both arms produce the same schema fields and status semantics, including:

- `k_min`, `k_run`,
- `prove_ms`, `verify_ms`, `peak_rss_mb`,
- `proof_bytes`,
- provenance metadata.

### W4. Same input-profile labeling

When the public reruns use `standard`, `boundary`, or `adversarial` generated
inputs, both arms are measured under the same declared input profile.

That profile must stay visible in the run metadata or report text.

## What is not required yet

This status note does not claim:

- Fiat-Shamir closure,
- backend extraction,
- system-wide wrapper soundness.

Those remain paper-level or backend-level proof obligations rather than
benchmark-alignment claims.

## Publication rule for upgraded claims

If any of the following diverges across arms in future reruns, do not publish
the result as an upgraded apples-to-apples benchmark:

- transcript family,
- commitment backend,
- public-input routing shape,
- proof API path,
- `k_run` contract,
- input-profile label.

## Bottom line for the current public package

For the current public reruns, wrapper-level parity means:

- same proof system path,
- same transcript family,
- same public-input plumbing,
- same fixed-k contract,
- same input-profile declaration.

Under those conditions, the remaining credibility argument can focus on
semantic-domain parity rather than accidental harness asymmetry.
