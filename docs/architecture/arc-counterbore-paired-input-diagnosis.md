# Arc prerequisite paired diagnosis — STOP: different inputs

Status: **DIAGNOSTIC REVIEW REQUEST; no production/comparator changes.**
Branch head during execution: `4a7c29960` on main `e46f44462`.
Rebase from `325ea6983` retained all three commits with range-diff `=`.
The former fitter-only prerequisite is **not proven responsible** for this RED.
Do not start its numeric repair: its mandatory input-equality gate failed.

## Authorized execution / trace authority

All execution used outer `taskset -c 15`, `CARGO_BUILD_JOBS=1`, test-threads1.
Rust build used independent `/tmp/worker4-arc-paired-eyIt/target` (nextest no-run).
START/END and probe SHA receipts are in `${TMP_ROOT}/ares-coord-inbox.log`.
No production, test assertion, geometry, breakpoint source, or comparator edits.
No push. Temporary debugger scripts are outside the repo and never committed.

Approved read-only probe sequence:
- `892df024...d89f87aa`: Native/Rust GDB scripts; Native succeeded, Rust initial
  source-line breakpoint selected an inline closure without `points` scope.
- `58bb705c...f19c0ad8`: select outer `simplify_points` frame; variables available,
  but absent Rust pretty-printer yielded vector headers/pointers, not elements.
- `a8efa6da...1ab3ad1a`: read inferior memory only; finally obtained raw point and
  fitting records. No write_memory, set var, inferior call or source mutation.

Native trace root: `/tmp/worker4-arc-paired-eyIt`.
Full diagnostic `native/plate_1.gcode` equals the frozen seeded None stream under
established generator-only normalization; witness
`native-full-output-equivalence.json` says `equal: true`. No motion-only gate.
Diagnostic raw SHA: `9b65daa01a00fd767c5e854d51f5a3066d5516ac9c87cb44e3d6440e37ae141e`.
Frozen raw SHA: `1021e9b7df8ce773699d238bfd56b894f6565197adb5a2de5ef455e03502cd77`.
Both traces use existing seeded ELF, not a patched producer. The trace captured
137 complete pre/post pairs. Inferred SysV PathFittingData stride128 was validated
against every captured arc's simplified integer start/end and bounds/type;
raw bytes are retained and padding bytes are not treated as semantic hashes.

Final Rust trace root: `/tmp/worker4-arc-rust-memory-BcZy`.
Files: `ares-trace.jsonl`, `ares-gdb.log`, `probe.sha256`, `paired-input-stop.json`.
The unchanged public None test reproduces the identical byte6053/line256 arc RED.
137 post-fit records were captured. Multiple inline breakpoint locations produce
380 pre records; do not compare global call ordinals to Native. No optimized-out
variables appeared in the final capture. Rust FittedMove stride56/field offsets
come from DWARF, and each point element is retained as raw f64 bytes as well as
numeric values. Convert to scaled integers using the recorded units_per_mm=1e6;
this reverses the existing integer-to-mm conversion at path_simplification.

## Pair identification and failed gate

Pair Native call13 with Rust pre/post id86 by the target retained arc's two
endpoints, its serialized I/J, surrounding Bottom-surface path, tolerance and
**58 identical initial scaled points**, not merely by equal call ordinal.
The Native captured path has 105 pre-fit points; Rust has 106.
Both tolerances are equivalent: Native12000 scaled, Rust0.012mm at1e6 units/mm.

First difference is **zero-based input point58**, before any fitter runs:

| Input index | Native scaled point | Rust scaled point |
| --- | --- | --- |
| 57 (identical arc start) | (-4858264, -6393015) | (-4858264, -6393015) |
| 58 | (-5038780, -6393015) | (-5037147, -6393015) |
| 59 | (-5402511, -6342795) | (-5267953, -6372926) |
| 60 | (-5416104, -6336941) | (-5405120, -6341672) |
| 61 | next linear infill endpoint | (-5416104, -6336941) |

There are also later 1–2 scaled-unit differences. Full original sequences are
saved in `paired-input-stop.json`; no point was deleted, merged or rounded for
acceptance. The additional connector point is not a G-code footer difference.

Post-fit target arcs both have retained start57/end59 after DP, but centers differ:
- Native scaled center (-4819151,-3201093), scaled start
  (-4858264,-6393015): I0.039113/J3.191922 → `I.039 J3.192`.
- Rust mm center (-4.804417,-3.054517), mm start
  (-4.858264,-6.393015): I0.053847/J3.338498 → `I.054 J3.338`.
- Both arc endpoints are (-5416104,-6336941) scaled.

This directly ties the paired records to the saved G2 mismatch, but **does not
prove either fitter is correct on the other's input**. Because input differs,
no hypothesis1 candidate/radius numeric modification is authorized.

## Owning input path / next source boundary

Pin remains `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
The active G-code section is Bottom surface with `bottom_surface_pattern=monotonic`.
Therefore the immediate upstream input owner is monotonic fill's contour-following
polyline generation, not GCodeWriter, the arc processor or an Arachne variable-width
wall extrusion:

- `OrcaSlicer/src/libslic3r/Fill/FillRectilinear.cpp::fill_surface_by_lines:2751`
  prepares the fill slice; its monotonic emission path at `:2595–2720` appends
  contour points between vertical strokes.
- `emit_perimeter_prev_next_segment:592` and
  `emit_perimeter_segment_on_vertical_line:656`, called by monotonic emission
  at `:2634,2674,2693,2713`, own copying those contour segments into the polyline.
- Its contour/offset preparation at `:397–427` supplies the vertices being copied.
- Rust destinations already exist:
  `fill/rectilinear/surface.rs::{fill_monotonic_surface,fill_surface}`,
  `rectilinear/emit.rs::emit_monotonic_polylines`,
  `rectilinear/perimeter.rs::{emit_horizontal_arc,emit_vertical_arc,append_contour_segment}`,
  `rectilinear/segments.rs::{prepare_rectilinear_contours,prepare_contours}`.

Arachne's earlier inner contour (`PerimeterGenerator.cpp:2148–2176,2475–2533`,
`Arachne/WallToolPaths.cpp::getInnerContour:779`) is a possible antecedent boundary
supplier. Current traces alone do not identify whether first drift occurs there,
in fill contour preparation, or during copying/rotation. No connector or Arachne
module is authorized for production edits by the fitter-only allowlist.

## Requested coordinator decision

Record this as a falsified fitter-only gate and STOP. Request a separately owned
**read-only monotonic input-provenance follow-up**, with paired contour inputs and
emitted connector points at the functions above to locate the first drift.
This report does not request or authorize a Fill/grid production edit; coordinate
with W6 ownership before a separate source-cited implementation scope. W3 centering
and W5 interface-shell owners remain read-only too. If the actual differing
contours originate earlier, report that exact upstream boundary before a further
scope amendment. Keep the complete None RED, all geometry and assertions frozen.
No additional runtime, fitter repair, counterbore GREEN or old guard migration
is undertaken after the input-difference STOP in this report.
