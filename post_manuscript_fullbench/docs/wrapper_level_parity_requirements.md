# Wrapper-Level Parity Requirements Before the Next Headline Rerun

## Purpose

This note records the minimum wrapper-level parity assumptions that must hold before the next benchmark is presented as stronger than a bounded instantiated-family comparison.

The immediate goal is not a full backend-soundness theorem. The goal is narrower:

- prevent the next benchmark from being attacked as a comparison of different proof paths rather than different circuit encodings.

## Minimum requirements

### W0. Same proving / verification API path

Both arms must go through the same proving and verification path exposed by the current harness:

- same `full-local` runner mode,
- same `create_proof` / `verify_proof` backend path,
- same transcript type,
- same commitment backend,
- same parameter family.

In the current harness that means:

- same KZG backend,
- same Blake2b transcript family,
- same local proof runner structure.

### W1. Same public-input routing contract

If either arm changes how public inputs are routed into the circuit or proof invocation, the benchmark stops being apples-to-apples.

Before the next rerun, both arms must continue to use:

- the same number of instance columns,
- the same empty/non-empty public-input shape for the measured path,
- the same proof-call argument structure.

### W2. Same fixed-k measurement contract

For each reported comparison point:

- both arms must run under the same declared `k_run`,
- both arms must expose the same `k_min` discovery semantics,
- both arms must report those values in the same output schema.

### W3. Same runner semantics and output schema

The benchmark package must not compare:

- one arm measured in a structural-only or mocked path,
- against another arm measured in a real proving path.

Both arms must produce the same schema fields and status semantics, including:

- `k_min`, `k_run`,
- `prove_ms`, `verify_ms`, `peak_rss_mb`,
- `proof_bytes`,
- provenance metadata.

### W4. Same input-profile labeling

If the next rerun uses `standard`, `boundary`, or `adversarial` generated inputs, both arms must be measured under the same declared input profile.

That profile must be visible in the run metadata or report text.

## What is not required yet

This milestone does not require proving:

- Fiat-Shamir closure,
- backend extraction,
- system-wide wrapper soundness.

Those are paper-level or backend-level proof obligations, not immediate benchmark-alignment blockers.

## Go / no-go rule

If any of the following diverges across arms, do not publish the rerun as an upgraded apples-to-apples benchmark:

- transcript family,
- commitment backend,
- public-input routing shape,
- proof API path,
- `k_run` contract,
- input-profile label.

## Bottom line

For the next rerun, wrapper-level parity means:

- same proof system path,
- same transcript family,
- same public-input plumbing,
- same fixed-k contract,
- same input-profile declaration.

If these hold, the remaining credibility argument can focus on semantic-domain parity rather than accidental harness asymmetry.
