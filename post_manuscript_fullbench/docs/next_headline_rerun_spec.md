# Next Headline Rerun Spec

## Purpose

This document fixes the exact contract for the next benchmark rerun.

The immediate goal is not to publish a universal equivalence claim. The immediate goal is to produce a stronger benchmark package that EthResearch readers can treat as a credible shared-input semantic-domain-parity comparison.

## Claim label

The next rerun should be described as:

- `shared-input semantic-domain-parity benchmark`

It should **not** yet be described as:

- `full equivalence benchmark`
- `universal drop-in replacement benchmark`

## Comparator

- `A_secure`
- `B_note`

No external comparator belongs in the headline table.

## Execution mode

- mode: `full-local`
- same local proof / verify path for both arms
- same transcript family
- same backend

## Input policy

The next rerun must use the shared generator path now implemented in `src/shared_inputs.rs`.

The minimum publishable rerun should include at least two profiles:

1. `standard`
2. `boundary`

Optional third profile:

3. `adversarial`

If only one profile is published first, it should be `boundary`, because it is easier to defend against the objection that the benchmark uses overly easy witnesses.

## Fixed-k policy

For each published comparison bucket:

- both arms must use the same `k_run`
- the report must expose both `k_min` and `k_run`

Recommended first rerun buckets:

- `scale = 1` sanity bucket
- `scale = 4` small bucket
- `scale = 8` medium-small bucket if runtime remains tolerable

Do not jump straight to a wide sweep until the first small shared-input rerun is stable.

## Metrics to publish

For each point, publish:

- `prove_ms`
- `verify_ms`
- `keygen_vk_ms`
- `keygen_pk_ms`
- `peak_rss_mb`
- `proof_bytes`
- `k_min`
- `k_run`
- `input_profile`
- git-pinned provenance

## Minimum repeat policy

For any result intended for public discussion, use:

- repeats >= 3

A 1-repeat run is acceptable only for smoke validation, not for headline evidence.

## Report policy

Publish one canonical report family only.

Recommended report shape:

- one markdown table per input profile
- one JSON summary per input profile
- same schema as the current repeat runner

## Go / no-go checklist

Do not publish the next rerun as upgraded evidence unless all are true:

- [ ] `B_note` is using full-table normalization in the measured path.
- [ ] both arms are using the shared input generator.
- [ ] the chosen input profile is visible in the report metadata.
- [ ] both arms run at the same `k_run` per point.
- [ ] repeats are at least `3` for any public headline number.
- [ ] the rerun happens inside the public git repo.
- [ ] one canonical report family is selected and older exploratory outputs are not mixed into the claim.

## Recommended immediate sequence

1. sync the shared-input and full-table changes into the public package
2. validate one-point `boundary` repeat run in the public repo
3. run `standard` and `boundary` small-bucket repeats with `repeats = 3`
4. only then decide whether to add a medium bucket
