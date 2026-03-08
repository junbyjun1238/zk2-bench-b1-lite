# Capacity Frontier Scope Note

Date: 2026-03-08

## What this capacity frontier does claim

The capacity-frontier material in this package is a property of the current public benchmark harness.

It answers the following narrow question:

- for the current shared-input, fixed-`k` rerun protocol, at what workload scale does the measured path stop fitting inside the current `k_run` and force the next `k`?

That is the scope of:

- `docs/k_jump_capacity_sweep.md`
- `docs/repeat_boundary_k18_s4520.md`
- `docs/repeat_boundary_k18_s4520_r3.md`

In that scope, the result is concrete and empirically checked:

- both arms stay at `k_min = 17` through `scale = 4519`
- both arms first jump to `k_min = 18` at `scale = 4520`

## What it does not claim

This is **not** a claim about all practical deployment scales used by downstream systems.

In particular, it does not try to answer:

- what every real-world wrapper deployment should choose for `k`
- what the best production `k` is for all provers or machines
- the global capacity frontier for every operational environment

Those are deployment-specific questions. Real users may have different public-input plumbing, larger wrapper paths, different proving hardware, or different backend parameter choices.

## Why this still matters

Even with that narrow scope, the frontier is useful because it identifies the first domain-size jump **for this benchmark contract**. That matters for two reasons:

1. it separates same-`k` sweep behavior from post-jump behavior, and
2. it explains why `scale = 1,4,8,16,32` stayed nearly flat in runtime and RSS under `k_run = 17`.

So the frontier should be read as:

- a harness-specific threshold for the current public benchmark package,
- not as a universal deployment recommendation.

## Bottom line

`capacity frontier` in this package means:

- the first `k` jump of the current shared-input public benchmark harness,
- not the operational frontier of every downstream production deployment.
