# Support connection costs per call

## Rewrite boundary

Authority: OrcaSlicer v2.5.0-beta, commit
`345efec64c7b95a05ce5cf96df0dde66a158835c`.

| Upstream | Behavior | Rust destination |
| --- | --- | --- |
| `Fill/FillBase.cpp:2697-2701` (`Fill::connect_base_support`) | `cost_low`, `cost_high` and `cost_veryhigh` are ordinary locals derived from the `line_spacing` of the current call. The comment at `:2697-2698` rules out `static` because the spacing varies per call | `fill/base_support/connect.rs` |
| `Fill/FillBase.cpp:2703-2740` | the arch selection reads `cost_low` and `cost_high` | `fill/base_support/link.rs::select_support_arches` |
| `Fill/FillBase.cpp:2873,2876` | reads `cost_high` | `fill/base_support/link.rs::connect_t_joints` |
| `Fill/FillBase.cpp:2894,2911` | reads `cost_veryhigh` | `fill/base_support/caps.rs::emit_very_long_arches` |

The bounded 2.4.2 slice ported these three values as function-local `static`s. The
target has no such lifetime.

## Measured owner

Fixture: `support_base_tree_auto_overhang`, the organic tree route with a rectilinear
base. The first difference from Native is in the `Support` role, part of the way down
the object, and every lower layer inherits it.

Native and Ares were compared at three successive points:

- the areas entering `generate_support_toolpaths` match, including polygon order,
- the arguments entering `connect_base_support` from `FillSupportBase::fill_surface`,
  meaning the outer polygons and fill lines, match on every call and in the same order,
- the polylines leaving that function differ on some of those calls.

Most calls in the fixture share one line spacing, and a few, including the first, use a
smaller one. With the costs fixed at the first call, the arch selection for the common
spacing runs with thresholds about a third of the upstream values.

## Rust divergence

| Site | Rust today | Upstream |
| --- | --- | --- |
| `connect.rs:106` | `context.support_connection_costs(line_spacing)` returns the value cached by the first call of the slice | locals from the current `line_spacing` |
| `slicer_context.rs:10,18,30-34` and `slicer_context/support_connection.rs` | a cache that exists only to hold these values | none |
| `support_connection.rs:1` | documents the costs as first-call statics | no such lifetime |

## Change

1. In `connect_base_support`, derive the three costs from the current `line_spacing`
   and hand them to the phases that read them.
2. Delete `SlicerContext::support_connection_costs`, its field and
   `slicer_context/support_connection.rs`. `SlicerContext` keeps the seed and the
   random stream.
3. Remove the `context` parameter from each function where this leaves it unused,
   starting at `connect_base_support` and following its callers. A function that still
   reads the seed or the random stream keeps it.
4. Correct the comments in `fill/base_support/` that describe the costs as first-call
   or point at the 2.4.2 line numbers for them.
5. Update `ard-0084`, `ard-0085`, `ard-0086` and `ard-0093`, which recorded the first-call
   lifetime as the contract. They now cite `Fill/FillBase.cpp:2697-2701` at the commit
   above.
6. Update the `Fill/FillBase.cpp` row of
   `2026-10-30-upstream-baseline-v2.5.0-beta-inventory-fill-support.md`: the status stays
   `M`, and the scope records the connection costs as updated to the target while the
   rest of the file stays a bounded 2.4.2 slice.

No API changes.

## Included and deferred

Included: the lifetime of the three connection costs and the code that existed only to
hold them.

Deferred:

- `fill/base_support/link.rs:271` sorts the T-joint candidates with a stable sort while
  upstream uses `std::sort`, and the arch selection in the same file already uses the
  ported unstable sort. Equal costs could order differently. This is unproven: the
  connected output on the fixture is exact without touching it.
- The rest of `Fill/FillBase.cpp`, and the other tree and manual support failures, which
  this change does not move.

## Failing cases and acceptance

No test code is added. The failing cases already replay through the public 3MF path and
compare the complete stream with a Native reference produced at runtime, with no stored
coordinate or count:

- `support_base_tree_auto_overhang`,
- `mk4_tree_support_base_connect_witness_matches_native`,
- `mk4_tree_support_default_short_edge_control_matches_native`,
- `support_interface_top_layers_base_connect_target_matches_native` and its control,
- `support_flow_and_gap_dependencies_preserve_complete_raft_output`,
- `normal_snug_sparse_interfaces_preserve_complete_output`.

Acceptance, in official and seeded modes:

- each case fails on the unchanged tree and passes with the change,
- no test moves from pass to fail in the
  `lightning | tree | support | ironing_pattern | coplanar_outer_wall` selection or in
  the `arc | spiral | m73 | lift | time | klipper | marlin | helix | scarf` selection,
- the output of `support_base_tree_auto_overhang` is byte-identical when the slicer may
  use 1, 2, 7 or 16 CPUs, and on a repeat at each width.

Support fills appear throughout the parity fixtures, so the full suite is the final check.
