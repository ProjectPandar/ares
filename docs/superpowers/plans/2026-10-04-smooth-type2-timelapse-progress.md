# Type2 execution evidence (WIP)

Continuation of [the complete-activation plan](2026-10-04-smooth-type2-timelapse-port.md).
The source boundary remains pinned libslic3r `WipeTower2.cpp`,
`ToolOrdering.cpp::WipingExtrusions`, `GCode.cpp::WipeTowerIntegration`, and
`PrintApply.cpp::print_objects_from_model_object`, with private Rust destinations
in `project_slice::gcode_emit` and `project::effective_config`.

## C/D work in progress (not a completed integration boundary)

Private `wipe_tower/` code now contains source-owned material projection,
writer, rectangle/cone/rib walls, rounding, depth planning, the five last-wipe
simulations, per-layer finish, priming, unloading/loading/ramming/cooling and
wiping. These were saved in checkpoint `3519d10d` but remain **not connected to production emission**.
B's whole-collection purge assignment and E–G remain pending; the full-output
RED evidence above is unchanged. Do not publish these modules as a completed
Type2 implementation or infer parity from their unit tests.

Precision regression: `scaled(Vec2f)` divides in float precision, whereas
`Point::new_scale` uses the double `scale_` macro (`Point.hpp:204–206,647–704`).
The new 0.7-mm test first failed with 699999 instead of 700000, then passed with
the separate source conversions. Evidence:
`/tmp/ares-pi-tower-type2-C-precision-{red,green}.log` (4/5 then 5/5 PASS).
Depth/activation/writer/geometry checks: 8 PASS in
`/tmp/ares-pi-tower-type2-C-plan-tests.log`. The additional generation test
covers all three shapes and two-tool transitions; it passed in
`/tmp/ares-pi-tower-type2-D-generator-tests.log`. These are supplementary kernel
checks, not the official/seeded comparisons or full gates.

Source reachability matters: `WipeTowerWriter2` comments out its arc-setting
initializer (cpp 578), leaving its private flag false even when the print
setting requests arcs. Its reachable linear path is ported; do not silently
activate the dormant arc branch. The wall gap caller supplies an empty skip
vector (cpp 2526–2528); linear path discontinuities still retract and travel.

Open source defect for final review: the last priming tool reads
`tools[idx_tool-1]` even for a singleton (cpp 1480). Rust must not reproduce the
out-of-bounds read. Current WIP uses the same-tool matrix entry in that case;
this needs explicit single-used-tool/unused-configured-tool reference evidence
before the final parity claim. No guard rejects single-tool activation.
Combined B/C/D kernel checks now pass 22 tests (including partial-axis SEMM reset):
`/tmp/ares-pi-tower-type2-BCD-placement-tests.log`. Clippy completed with 354
warnings (`...-placement-clippy.log`), failing zero-new-warning (baseline 249).
No suppressions were added. Consumers must be connected and warnings resolved
before review. These checks do not establish complete-output parity.

B's original collection-volume snapshot is now captured before island
flattening and `path_simplification::apply`, following `Print.cpp:2425,2571`.
It retains original 3D polyline length, not fitted-move lengths, and thin-fill
IDs follow the ordinary fill volumes. Capture is limited to objects requesting
infill/object flushing. Assignment and production consumers remain pending;
`/tmp/ares-pi-tower-type2-B-raw-volume-check.log` records the compile check.
The strict LargeBed volume fixture was corrected to `100.00000000000001`,
confirmed by a native C++ scalar probe; no tolerance was introduced.
A private source `WipingExtrusions` allocator now has four passing checks for
whole collections/per-copy overrides, dedicated-object priority, support overrides
with double subtraction, and the source boolean-as-tool ordering expression.
`/tmp/ares-pi-tower-type2-B-dedicated-tests.log` is kernel evidence only. Grouped
instance indices now flow into `ResolvedPrintObjectConfig`, following
`PrintApply.cpp:129–167`; actual emission/integration remain unfinished. A two-copy CLI witness is being added.

## Authorized compilable checkpoint

The coordinator explicitly requested saving the current compilable WIP to the
feature branch before continuing production integration. This checkpoint is
not a completed B–G boundary or a REVIEW-REQUEST. Main's current Clippy baseline
is approximately 219 warnings, not the older 249 count; the branch's 354 warnings
must lose all introduced diagnostics before review, without suppressions.
The checkpoint was pushed as `3519d10d03283e71862ea015fa8a4df7a35e5faf`;
workspace `cargo check` and formatting passed before publication.

## Object-island instance dispatch

`layers/instances.rs` now resolves real printable members, caches their integer
shifts once per print object, and supplies tool-bucket/copy dispatch. This ports
`PrintApply.cpp:129–167`, `ShortestPath.cpp:2015–2036`, and `GCode.cpp:5106–5138,5380`.
Tool-specific ordering chains only participating instances, in source object/copy
order, using the configured raw-coordinate tower seed. No millimetre conversion
is used for the integer chaining input. The emitter installs each actual shift
and tracks `(print object, local copy ordinal)` for external-planner handoff.
The consuming island adapter clones additional copies and moves the last one;
single-instance emission does not clone geometry. Original region/collection
identities survive those replays. Purge allocation and tower emission remain
pending; copy-aware brim/labels/obstacle geometry still need end-to-end audit.
In particular, AutoLift still indexes a ModelInstance by transform-group index.

Validation for this dispatch slice:
- 18 focused tests PASS: `/tmp/ares-pi-tower-copy-dispatch-final-unit.log`.
- Clippy remains 354 warnings (no increase): `...-final-clippy.log`.
- Official and seeded runs now assert positive Ares extrusion at both contours,
  in addition to the existing native assertions and unchanged full comparator.
  The two-copy Ares stream grows from 121630 to 200157 bytes, M73 R9 to R14;
  native remains 831091 bytes, R69. Both full-output comparisons remain RED.
  Evidence: `/tmp/ares-pi-tower-copy-dispatch-assert-{official,seeded}.log`.
- All five BBL/PA control test functions pass in each reference run (official:
  `...-final-official.log`; seeded: `...-assert-seeded.log`).
- Core/adapter WASM build, formatting and diff checks PASS:
  `/tmp/ares-pi-tower-copy-dispatch-wasm.log`.
No generator/purge integration completion or REVIEW-REQUEST is implied.


## Copy ownership and full-output witness

- Source-instance members now survive transform grouping into
  `ResolvedPrintObjectConfig::instance_indices`; local copy ordinals are positions
  in that list, not the original ModelInstance indices themselves.
- RED: missing member lookup (E0599), `/tmp/ares-pi-tower-copy-members-red.log`.
  GREEN: 104 related tests, `/tmp/ares-pi-tower-copy-members-green.log`.
- The CLI SEMM/infill fixture now has two translated instances of one object.
  It uses fresh native project/output paths, avoiding stale exported-case cache
  reuse. Native comments label both copies `copy 0`; assertions instead verify
  real extrusion at both cube contours (X90.31 and X120.31, Y114.69).
- Official and seeded references separately pass fixture validation but remain
  full-output RED: 831091 reference bytes versus 121630 Ares bytes; first
  difference at byte465, M73 R69 versus R9. Evidence:
  `/tmp/ares-pi-tower-type2-copies-official-final.log` and
  `/tmp/ares-pi-tower-type2-copies-seeded-final.log`. Earlier cache-reusing and
  copy-label assertion runs are diagnostics, not valid parity evidence.
- Current focused regression run: 126 PASS,
  `/tmp/ares-pi-tower-copy-final-unit.log`; fmt/diff checks clean.
  Clippy completed with 354 warnings,
  `/tmp/ares-pi-tower-copy-final-clippy.log`. The three new grouping nesting
  warnings were removed; the disconnected-kernel warning debt still fails
  the zero-new-warning gate.
- `ares-core` and `ares-wasm` build for `wasm32-unknown-unknown` passed:
  `/tmp/ares-pi-tower-copy-wasm.log`.
- Existing BBL/Adaptive PA controls pass all five test functions separately
  against official and seeded references:
  `/tmp/ares-pi-tower-copy-controls-{official,seeded}.log`.
- No production copy emission, generator integration, final review request or
  main integration is claimed by these data-model and fixture changes.
