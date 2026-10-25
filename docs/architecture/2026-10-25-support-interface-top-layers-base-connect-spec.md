# SPEC: support_interface_top_layers=4 base-support connect_base_support RED

Base `4af042def1d7766b6573b9c47768151f527d1bb1` (main); coordinator ACK
2026-10-25 authorizes docs SPEC + tests-only RED on main, frozen RED then
review request; GREEN is **not** authorized until the in-function fork is
narrowed further (see GREEN precondition).

## Non-negotiable source boundary

Port the divergence-owning behavior of
`OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:2247-2650::Fill::connect_base_support`
as reached from the support base fill chain:

```text
SupportCommon.cpp:1764-1816  generate_support_toolpaths base fill block
  -> SupportCommon.cpp:712-760  fill_expolygons_with_sheath_generate_paths
  -> SupportCommon.cpp:410-431  fill_expolygon_generate_paths
  -> FillRectilinear.cpp:3613-3634  FillSupportBase::fill_surface
       (ExPolygonWithOffset -> make_fill_lines -> connect_base_support -> rotate)
  -> FillBase.cpp:2247-2650  Fill::connect_base_support
```

Rust destination: `crates/ares-core/src/fill/base_support/connect.rs:29
connect_base_support` (stage modules graph/mark/extend/link/take/arches/caps),
reached from `crates/ares-core/src/project_slice/raft/fills/support_base.rs:20
fill_surface` via `support_material/toolpath_cache/base.rs::fill_grid_base` ->
`raft/fills.rs::raft_layer_entities`. The existing Ares port is a
compatibility shell of the cited upstream; this slice fixes its literal
divergence, it does not redesign it.

Included: only the `connect_base_support` stage (and its inputs as already
computed) whose behavior differs at the recorded boundary.
Deferred: everything proven equal must not be touched — organic tree pipeline
(`TreeSupport3D.cpp:1099-1234`, `TreeSupportCommon.hpp:724-753`,
`TreeSupport3D.cpp:2446-2540`, `3673-4103`, `SupportCommon.cpp:47-239`),
`polygons_to_extrude` preparation, fill parameter selection
(`SupportParameters.hpp:120-133`), closing/offset preprocessing
(`SupportCommon.cpp:742-748`), vertical segment generation
(`make_fill_lines`), plus all deferred support/raft keys
(`support_interface_bottom_layers`, `support_interface_pattern`,
`support_on_build_plate_only`, non-organic styles, cooling/temperature).

## Evidence

Dual-side intermediate probes (two rounds, all output-neutral against the
recorded oracle streams; CPU11, threads1, seed0):

1. `support-raft-nonp1s/organic-intermediate-probe/` — organic pipeline and
   interface stages EQUAL; ordered support entities differ in exactly 1 of
   1570 witness records (control 1564/1564 equal), at print_z 0.8.
2. `support-raft-nonp1s/base-fill-connect-probe/` — fill inputs EQUAL per
   layer (FILLIN: `polygons_to_extrude` vertex multisets, pattern
   ipSupportBase, angle 0, spacing 0.377079606, density 0.131063327,
   link_max_length 0, sheath 1, no_sort 0); closing/offset expolygons EQUAL
   (FILLEX); pre-connect vertical segment multisets EQUAL (SEG, every fill
   call); post-connect polylines differ in exactly 1 of 41 at print_z 0.8
   (FILLLINE): native `pts=38 8338494,20441216 -> 7500889,23170434
   len=12473156.7` vs ares `pts=48 5956640,23155876 -> 2185414,20292345
   len=14759755.7`.

Therefore the first fork is inside `connect_base_support` given identical
inputs. Normalized diff hashes for the witness
(`support-raft-nonp1s/support_interface_top_layers_4/`):
official `54c8748bb355d02d2618ec9d92671de9d192e515ee51639099ff1e1ce88cf7b5`,
seeded `2fad802eb8f419f24d9e5ea0f507a7215096f1952731955ac333407e47f1fd39`.

## Test-first plan (this change)

Tests only; no production code. Add
`crates/ares-cli/tests/orca_parity/support_interface_top_layers_base_connect.rs`
through the existing `native_stream_case` seam plus two SHA-pinned fixture
archives under `tests/parity/support-interface-top-layers-base-connect/`
(byte-for-byte from the evidence; no Native output committed). Target =
`top4.3mf` (`support_interface_top_layers=4`, SHA
`a4e969503b241509a5d8e87f02484b41351d6de1883e5391c064c7967f127380`);
control = fixture-default `control.3mf` (top layers 2, SHA
`7a8b9f2abfb2bbc8b74b76bd9affb458ad5a96c81353cfa767b66b773db23239`).
Native replay determinism is qualified by the seam; the callbacks qualify
that both cases extrude support and that the target's `;Z:0.8` layer
extrudes support (the proven fork layer).

RED exits: run both cases in official then seeded mode at CPU11, threads1
(`ARES_PIN_OFFSET=11`), strict complete ordered stream comparison
(generator-line normalization only). Expected: control PASS in both modes;
target FAIL in both modes with the first difference inside the `;Z:0.8`
layer; API==CLI on both. Commit RED, then issue the review request and hold.

Failure classes this RED must catch: any change to `connect_base_support`
chaining that alters other layers or the control stream (control must stay
byte-equal), regression of the witness polyline, and any upstream-equal
stage being touched (organic pipeline, polygons, parameters, preprocessing,
segment generation — regressions there surface as earlier first differences
outside `;Z:0.8`).

## GREEN precondition (not yet authorized)

Before any GREEN: narrow the fork to the specific stage and line inside
`Fill::connect_base_support` (boundary graph construction / mark / extend /
link / take / arches / caps / connection selection) with dual-side
intermediate-quantity probes, then fix only the literal difference in the
corresponding segment of `fill/base_support/`. No whole-function rewrite;
other callers of `connect_base_support` (sparse support infill generally)
must stay byte-equal, proven by the control case and the existing FULL
suite. After GREEN: fmt, workspace all-target clippy, both directed cases
in both modes, FULL via `parity-control-set.sh` (official and seeded), then
review request and hold.
