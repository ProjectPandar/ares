# Support interface projection RED evidence

## Source and fixture boundary

Authority: OrcaSlicer v2.5.0-beta, commit
`345efec64c7b95a05ce5cf96df0dde66a158835c`.
`SupportCommon.cpp:220–233,305–354` owns interface projection count derivation
and slab selection. The Rust destination is
`crates/ares-core/src/project_slice/support_material/interface_grid.rs`.

The existing public API/CLI/Native support and raft witnesses are reused; no
comparison or assertion is changed. Their artifact metadata now cites this
support boundary alongside the existing skirt sources. The count derivation is
not yet implemented at this RED point.

## Reproduction

Base main: `444364eaf2c82096cc00de06fa963a7ac40c6475`.
ARD parent: `0a8886064`.
Use the pinned Native parity environment, verified source profiles, required
Native execution, and an external artifact root, then run:

```sh
cargo nextest run -p ares-cli --test orca_parity \
  --test-threads 4 --no-fail-fast --status-level pass \
  -E 'test(support) | test(raft)'
```

The family runs 202 tests: 112 passed, 90 failed. Both target witnesses fail
before applying the count derivation:

| Existing strict witness | First full-output difference |
| --- | --- |
| `per_object_skirt_support_red_matches_native` | Line 88: Native `G1 X106.329 Y193.562 E.1063`; Ares `M73 P2 R41` |
| `per_object_skirt_raft_red_matches_native` | Line 238: Native `G1 X92.51 Y157.012 E.40698`; Ares `M73 P2 R42` |

The earlier M73 placement is not evidence of an independent timing defect.
The ARD records the first measured polygon difference at interface generation:
12 of 30 records differ, while preceding projection and object-trim records
match. The GREEN port must eliminate that polygon difference without changing
timing arithmetic, clipping, or strict comparison policy.

The preset-refresh footer keys (`compatible_process_expression_group` and
`pellet_flow_coefficient`) remain an explicit strict configuration residual.
Geometry-scoped GREEN must not be reported as full-output PASS while those keys
remain absent. Rustfmt and strict workspace, all-target Clippy pass at this RED
point. Raw diagnostics and verifiable G-code artifacts remain outside the
repository.
