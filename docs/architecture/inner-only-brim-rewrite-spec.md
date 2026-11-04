# SPEC-REQUEST: Brim.cpp inner-only hole brim

## Roadmap selection

See [RED evidence](inner-only-brim-red-evidence.md) for the test-first record.

Selected from roadmap **Current acceptance priorities: full acceptance (open)**:
source-owned active dependencies and strict complete-output witnesses remain
open even when PrintConfig registry milestones are DONE. `inner_only` needs
an actual first-layer hole, not an empty selector/config-echo probe.
M44's registry completion does not establish this runtime boundary's parity.

## Upstream ownership and exact Rust boundary

Source snapshot: repository `OrcaSlicer` link to the read-only v2.4.2
seed-replay source, not independently designed adhesion logic.
`Brim.cpp` SHA256:
`807c3a449dfc07a887959f3f7badb86eab9dd75e017ff136e03d04f96979dbe3`.

All upstream paths below are under `OrcaSlicer/src/libslic3r`.

| Upstream owner | Rust destination / treatment |
|---|---|
| `PrintConfig.hpp:271-278` BrimType; `PrintConfig.cpp:1680-1721` width/type/gap | Reuse existing ProcessBrimType::InnerOnly and typed options; no new option/API |
| `Brim.cpp:421-425,448-458` outer_inner_brim_area flow, quantization and selector predicates | Existing `ares-core/src/project_slice/gcode_emit/brim.rs::BrimPlan::generate` admits InnerOnly in the bounded project context |
| `Brim.cpp:529-530,547-558` reversed holes, negative offsets and intersection | New private `ares-core/src/brims/inner.rs` area producer, registered in `brims.rs`; no rectangular bounds approximation |
| `ClipperUtils.hpp:19-27,344-346` default jtMiter / limit3 | Pass the source join/limit to existing geometry offset services; no global default change |
| `Brim.cpp:564-570,678-711` no-outer exclusion, object footprint subtraction and contact retention | Inner-only private area/exclusion logic plus existing project `brim/bed_area.rs`; retain the single-object own-island/own-brim contact condition |
| `Brim.cpp:813-875` tryExPolygonOffset / makeBrimInfillImpl | Reuse `brims/ears/paths.rs::from_area` and existing `brim/emit.rs`; this is a shared upstream area-to-path port despite its current directory name |

Rust paths above are relative to `crates/`. No new crate or public entry point.
See [runtime ARD](ard-inner-only-brim-runtime-boundary.md).

## Current scaffold and migration

Static evidence: `BrimPlan::generate` in
`project_slice/gcode_emit/brim.rs:29-36` accepts only OuterOnly/BrimEars,
so InnerOnly returns None. This is a candidate runtime gap, not a measured
Native divergence; the spec precedes admission and RED capture.

Add the source-owned InnerOnly dispatch and area calculation to the existing
project adapter. BrimPlan remains a temporary compatibility shell around
outer_inner_brim_area/makeBrimInfillImpl, not a new Ares pipeline stage.
Reuse its flow, placement, printable-area clipping and emission boundaries.
Do not silently fall back to outer-only or no-brim when the new path fails.

The separate public `generate_brims` adapter currently approximates inner
paths with `brims.rs::inner_brim_paths` bounding rectangles. That adapter and
its square-coordinate assertions are explicitly deferred. Its replacement
needs its own admitted boundary and test disposition; this SPEC permits
no assertion retirement and creates no runtime fallback to that scaffold.

## Included upstream behavior

- One object, one instance, one model-part volume/group, one filament/nozzle;
  explicit InnerOnly, ordinary first-layer slices, EFC outline disabled.
- All holes in the admitted first-layer ExPolygons, including concave holes,
  two holes and a narrow-neck hole whose negative offset can split/collapse.
- Width quantization in pairs of scaled brim-flow spacing; scaled gap and
  upstream float/integer conversion order. Width0 or quantized width0 stays
  inactive; a model without holes produces no inner brim.
- Reverse hole winding; inset at `-width-gap`, outer boundary at `-gap`,
  difference then intersection with the original reversed holes. These
  offsets use upstream default Miter/limit3, not the outer branch's Round.
- No-outer exclusion uses `no_brim_offset = flow.scaled_spacing()` from
  `make_brim:903-904`; objectIsland uses the source Round gap offset.
  Preserve the area subtraction and two-spacing contact test for this one
  object. The other-object contact set is empty in the admitted context.
- Existing whole path ordering, loop connection, erBrim extrusion and footer
  calculation; actual positive XY/E brim motion must occur inside holes.

## Deferred adjacent behavior

OuterAndInner runtime dispatch, automatic-width analysis, painted ears and
model transport, support brim, EFC snapping/compensated footprints, combine
brims, multiple objects/instances/groups/tools, shared objects, plate offset
redesign and per-extruder bed-area redesign remain deferred. Existing outer
and automatic-ear behavior is a preservation control, not expanded scope.
Public LayerContours adapter replacement is deferred as stated above.
No APA, G4, profile-loading, fill/anchoring, Clipper kernel, CLI policy or web
changes. Any RED whose first cause lies there must be reported to Claude;
do not remove its guard or fix an owned neighboring slice.

## Failure inventory before writing tests or code

1. Fixture has no real first-layer hole or Native never emits an inner brim.
2. Public project admission rejects the input before reaching this owner.
3. Width/gap spacing units, quantization or implicit offset join differ.
4. Hole winding or boolean masks emit outside the hole or on solid material.
5. Narrow-hole collapse/splitting loses islands or changes path ordering.
6. Forbidden-area/contact handling erases a valid ring or keeps an orphan.
7. Area is right but connection, travel, extrusion, time or full footer differs.
8. No-hole/zero-width controls or existing outer/ear streams change.

## Test-first plan and acceptance

1. Author input-only 3MF fixtures and Native-backed
   E2E tests before production code. Proposed finite cases: rectangular
   hole, concave hole, two holes, narrow-neck hole, positive gap0.1, width
   large enough to collapse a hole inset, width0, sub-pair width, and no-hole
   solid control. Reuse the original six N/O/E brim controls unchanged.
   Geometry, effective settings and archive members must be recorded.
2. Execute seeded and official with canonical control-set-v2,
   fresh Native cache and artifact root for every run/mode. Preserve saved
   Native settings, complete Native/public API/real CLI G-code, first byte
   difference, first-layer erBrim XY/E witnesses and case outcomes.
   No fixture goldens from Native output; no first-layer-only acceptance.
3. Commit the real RED separately. Admission errors and inactive controls
   are not production RED. If both references already match, report
   NO RED FOUND and do not manufacture a production change.
4. Port only the source-owned boundary above. Keep whole ordered-byte
   comparison with the existing generated-by normalization unchanged.
   No unit tests after implementation, deleted/weakened assertions,
   ignore/allow/expect, probes, tolerance changes or legacy fallback.
   Every changed/new Rust file stays at most400 lines.
5. Require both-mode directed whole option_coverage plus the finite witnesses,
   strict Clippy, rustfmt, core/WASM wasm32 checks and a fresh clean-env
   workspace Nextest run. No filesystem/process/UI behavior in core; the
   same byte API remains usable on browser WASM, Windows, macOS and Linux.

Endpoint completeness beyond these finite witnesses remains
open; this slice does not close the seven-value brim_type domain globally.
