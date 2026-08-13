# Task 22O.75 — full-grouping sparse anchoring

## Goal and source boundary

Port pinned OrcaSlicer
`8500fcdccaa10b5099ac20d252af3a7c560046f1`'s caller relationship at
`src/libslic3r/Fill/Fill.cpp:1394-1407`: sparse-infill anchoring must consume
the same complete `group_fills` result as `Layer::make_fills`, then select only
`stInternal` groups. Reuse the KSR-active CrossHatch dispatch already ported
from `Fill.cpp:1409-1499`.

## Required Rust behavior

`project_slice::prepare_infill::bridge_over_infill::sparse_anchoring` receives
`&PreparedPostExternalSurfaces`, `object_index`, and `layer_index`. It calls
`project_slice::group_fills::group_fills` exactly once and iterates returned
`surface_fills` in order. Non-`Internal` representatives are skipped. For each
KSR-active CrossHatch group, each authoritative ExPolygon is passed in order to
the existing CrossHatch filler using the returned params:

- `spacing`, `angle`, `multiline`, `anchor_length`, and `anchor_length_max`
  come from the grouped result, while its percentage `density` is converted by
  source expression `float(0.01 * density)`;
- `z` comes from the aligned planned layer;
- `overlap` remains zero for this source anchoring call;
- output remains owned and deterministic.

The full grouping error is returned unchanged as `SliceError`; reached filler
Clipper errors retain the bridge transaction's existing geometry mapping.
Inputs remain borrowed and unchanged on success and failure.

## No fallback

Delete `sparse_anchoring/grouping.rs`. No reduced comparator, three-pattern
enum, local priority clipping, caller-built `SparseAnchoringLayer`, wrapper,
alias, test seam, feature flag, or compatibility branch may remain. The bridge
transaction passes the already owned prepared graph and aligned indices. It
must not reconstruct options from individual fields or parse the 3MF again.

## Tests

TDD crosses the graph-native anchoring seam and owning bridge transaction.
Required evidence:

1. a RED test showing the old caller-built seam cannot satisfy the new graph
   entry;
2. the fixed-MSVC KSR 18-layer anchor oracle remains exact or is replaced only
   by independently captured output from the same pinned source caller;
3. repeatability and prepared-graph immutability;
4. grouped priority/narrow behavior reaches anchoring before the `Internal`
   filter;
5. full-grouping errors are atomic;
6. static absence of `sparse_anchoring/grouping.rs`, `group_and_prioritize`, and
   `SparseAnchoringLayer`.

Tests stay in separate ordinary modules. Every changed/new Rust file stays
below 400 LOC. Rust source splitting must not use `include!` or
`include_bytes!`.

## Included and deferred

Included: the `Fill.cpp:1394-1407` full-grouping caller replacement and the
KSR-active CrossHatch continuation.

Deferred: unsupported fill generators and rotation templates, adaptive or
lightning state, `Layer::make_fills`, extrusion entities, ironing, motion,
G-code, CLI completion, and normalized golden parity. Public slicing remains
`ProjectSlicingIncomplete`.
