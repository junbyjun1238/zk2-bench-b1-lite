# External Standard-Library Comparison (full-local)

- k_run: `17`
- scales: `8,16`

## Prove/Verify Time Table (ms)

| scale | A prove | B prove | ext prove | B/A | B/ext | A verify | B verify | ext verify | B/A | B/ext |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 8 | 108168.380 | 80297.540 | 24699.758 | 0.742 | 3.251 | 46.740 | 26.335 | 22.530 | 0.563 | 1.169 |
| 16 | 107652.089 | 80558.520 | 26147.466 | 0.748 | 3.081 | 42.355 | 27.559 | 19.770 | 0.651 | 1.394 |

## Notes

- `ext_halo2wrong` uses row-family-equivalent `maingate::MainGate::to_bits` decomposition workload from external standard library.
- This external arm is a decomposition-only baseline (it does not include this repo's full row-family relation gates, digest binding, or certificate checks).
- This is an external library baseline and is intentionally kept separate from frozen `v1.0.0` arm schema.

Raw outputs: `benches/external_h2w_compare`
