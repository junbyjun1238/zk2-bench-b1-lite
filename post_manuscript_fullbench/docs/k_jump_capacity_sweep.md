# K-Jump Capacity Sweep

Date: 2026-03-08

Goal: locate the first workload scale at which the shared-input public benchmark no longer fits inside `k_run = 17` and therefore forces `k_min = 18`.

## Method

We probed the public-package binaries directly with `--probe-k-min` under the shared-input path:

- `cargo run --release --quiet --bin a_secure_full_local -- --scale <S> --probe-k-min --input-profile standard`
- `cargo run --release --quiet --bin b_note_full_local -- --scale <S> --probe-k-min --input-profile standard`

Scales checked near the expected boundary:

- `4096`
- `4518`
- `4519`
- `4520`
- `4521`
- `5000`

## Empirical Result

For both `A_secure` and `B_note`:

- `scale = 4519` still fits in `k_min = 17`
- `scale = 4520` is the first point that forces `k_min = 18`

Measured probe output:

```text
A_secure
  scale=4096: k_min=17
  scale=4518: k_min=17
  scale=4519: k_min=17
  scale=4520: k_min=18
  scale=4521: k_min=18
  scale=5000: k_min=18
B_note
  scale=4096: k_min=17
  scale=4518: k_min=17
  scale=4519: k_min=17
  scale=4520: k_min=18
  scale=4521: k_min=18
  scale=5000: k_min=18
```

## Why The Threshold Is 4520

The current fixed-`k` shared-input benchmark has observed physical-row growth

- `physical_rows = 29 * scale`

which matches the measured reports so far:

- `scale = 1  -> 29 rows`
- `scale = 4  -> 116 rows`
- `scale = 8  -> 232 rows`
- `scale = 16 -> 464 rows`
- `scale = 32 -> 928 rows`

For `k = 17`, the domain has `2^17 = 131072` rows. With the current Halo2 usable-row convention used by the harness, the effective limit is `131072 - 6 = 131066` usable rows.

Therefore:

- `4519 * 29 = 131051` fits
- `4520 * 29 = 131080` exceeds the `k = 17` usable-row limit

So the first forced jump is exactly:

- `scale = 4520 -> k_min = 18`

## Local/Cloud Guidance

Under the current shared-input rerun protocol, another same-`k = 17` sweep point is not the next cloud trigger. The next meaningful resource checkpoint is:

1. the first rerun at or beyond `scale = 4520`, where `k = 18` becomes mandatory, or
2. a wrapper-level parity benchmark that materially enlarges the proving path.
