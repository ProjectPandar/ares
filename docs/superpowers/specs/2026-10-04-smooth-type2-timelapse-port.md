# Timelapse state and smooth Type2 selective rewrite

Owning sources and destination decisions are in ARD-0105. Included work:

1. Reproduce stock BBL H2S / P1P / P1S first-travel Z-restatement differences
   at the public complete-output seam; port the applicable `_extrude` /
   `GCodeWriter` state transition into existing Rust motion emission.
2. Port the complete activated Type2 boundary from `WipeTower2.cpp/.hpp`,
   `Print::_make_wipe_tower` and `ToolOrdering`, selectively reusing historical
   arithmetic rather than its guarded single-object/single-tool planner.
   Included: multi-tool partitions and purge assignment, priming, raft-gap
   layers, rectangle/cone/rib geometry, sparse layers, unload/load/wipe,
   occupancy/placeholders, travel/wipe handoff, statistics and Adaptive PA
   reset. Preserve current object/region/logical-tool ownership.
3. Verify source activation and full output, including tower-disabled controls,
   no-skirt bounds, thin layers, multiple objects/materials, raft, PA, offsets
   and unloading configurations. The plan records source sub-boundaries and
   their required E2E witnesses before production implementation.

Complete activation is required: no standalone unconnected writer/geometry
completion claims. Several commits may implement the slice, but each must cover
a complete activated path with official and seeded full-output parity and no
new guard.

Deferred/unclaimed: independent Type1 tower algorithms, exhaustive printer/
option coverage, hardware operation and native Windows/macOS runtime evidence.
Shared Type1-header data required by Type2 is included. Exceptions are recorded with source consumers in ARD-0105: the Type1-only finish
flag, GUI extrusion color tag, and unread result/writer timing bookkeeping are
deferred until their consumers are ported. Command tool identity (including
RepRap M572), geometry, filament accounting and processed G-code timing remain
included. These exclusions do not
permit guards or silent omission of active Type2 source behavior; do not weaken
fixtures or comparisons to make a partial generator appear complete.

Multi-transform PrintObject preparation and `PrintObject.cpp:85–114`
centering are a separate slice. This is a general Print/PrintApply prerequisite, not part of the tower slice.
`raw_intersections.rs:290–305` stays unchanged. Its native-accepted RED witness
is not part of the tower suite; see ARD-0105.
Same-transform actual-copy ownership remains included.

The BBL state repair may land as an independently green prerequisite;
it must not be represented as completion of the Type2 tower port.
