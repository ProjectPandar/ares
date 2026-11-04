# Spec: Brim.cpp OuterAndInner project brim

## Roadmap selection

A separate admitted Native fixtures/tests RED precedes implementation; see
[RED evidence](outer-inner-brim-red-evidence.md).

Selected from roadmap **Current acceptance priorities: full acceptance (open)**:
active selector dependencies need source-owned, nonempty complete-output
witnesses. The inner-only slice deliberately deferred OuterAndInner; this
proposal closes only its bounded real-project runtime path, not the full
seven-value brim domain or printer/Option acceptance.

The proposed production paths are confined to brim modules. No other runtime
file or top-level `orca_parity.rs` / `case_helpers.rs` registration is changed.

## Upstream ownership and Rust destination

Source: the repository's read-only OrcaSlicer v2.4.2 seed-replay source link.
`src/libslic3r/Brim.cpp` SHA256:
`807c3a449dfc07a887959f3f7badb86eab9dd75e017ff136e03d04f96979dbe3`.
Upstream paths below are relative to `OrcaSlicer/src/libslic3r`;
Rust paths are relative to `crates/ares-core/src`.

| Upstream function/boundary | Rust destination and treatment |
|---|---|
| `Brim.cpp::outer_inner_brim_area:448-458` paired width and both selector predicates | Existing `project_slice/gcode_emit/brim.rs::BrimPlan::generate` admits existing `ProcessBrimType::OuterAndInner` in this context |
| `Brim.cpp:512-545` per-ExPolygon outer contour, outer width re-quantization, two Round offsets and difference | New private `brims/combined.rs` combined area producer; retain contour-only ownership and upstream append order |
| `Brim.cpp:529-530,547-558`; `ClipperUtils.hpp:19-27,344-346` | Reuse/extract hole-ring calculation from `brims/inner.rs`, with original paired width, reversed holes and default Miter limit3; never borrow the outer modified width |
| `Brim.cpp:560-570` mutually conditional masks and Round objectIsland | Combined producer excludes both no-inner/no-outer masks; only the shared object footprint is initially forbidden |
| `Brim.cpp:661-679,686-711` bed exclusion and contact retention | Reuse `project_slice/gcode_emit/brim/bed_area.rs` and `brims/inner.rs::retain_contact`; apply contact retention to the combined clipped area |
| `Brim.cpp::tryExPolygonOffset / makeBrimInfillImpl:813-875` | Reuse `brims/ears/paths.rs::from_area` and existing project emitter; no separate outer/inner emission streams |

Register the private producer in `brims.rs`. No public API, new option,
candidate crate, geometry default or native dependency enters core.
[Runtime ARD](ard-outer-inner-brim-runtime-boundary.md).

## Existing scaffold and migration

Before this slice, the actual project `BrimPlan::generate` accepts OuterOnly,
InnerOnly and BrimEars, while OuterAndInner returns None. This is a static
runtime-gap hypothesis; no Native divergence is claimed before admission.

The existing OuterOnly branch offsets complete ExPolygons, including holes;
the cited upstream outer branch offsets each **contour**. Its result cannot
simply be concatenated with inner-only output for this port. The inner-only
producer also adds the no-outer exclusion, which would erase a valid outer
ring if reused wholesale. Extract only the shared upstream hole calculation;
construct combined areas/masks at their owning source boundary.

BrimPlan remains a temporary compatibility shell around the named upstream
area/flow/path functions. Existing OuterOnly, InnerOnly and ear behavior stays
under its existing controls. This proposal does not repair OuterOnly's
deferred hole semantics or route through the public rectangular
`generate_brims` adapter. That adapter and assertion disposition remain
deferred; there is no legacy fallback.

## Included source behavior

- One object, one instance, one model-part volume/group, one filament/nozzle;
  ordinary first-layer slices, explicit OuterAndInner, support/prime tower,
  skirt and EFC outline disabled. Holes and multiple footprint islands within
  that single volume are included.
- The shared initial width is paired-quantized as at line450. For explicit
  OuterAndInner, `has_brim_auto` is false; preserve outer re-quantization at
  line527 without auto adhesion additions/clamping. Inner always uses the
  original width, not `brim_width_mod`.
- Per source ExPolygon, append outer area then hole area. Outer inner boundary
  is `offset(contour, gap, Round, SCALED_RESOLUTION)`; outer boundary offsets
  that result by the outer modified width. Subtract the two boundaries.
- Reverse hole winding, offset by `-width-gap` and `-gap` using default
  Miter3, subtract and intersect with the reversed holes. Do not apply Round
  or SCALED_RESOLUTION to these hole offsets.
- Both selector predicates are true: neither line562's no-inner hole mask
  nor line565's no-outer contour mask participates. The Round offset of the
  actual ExPolygons by gap supplies objectIsland and its exclusion.
- Reuse existing printable-area subtraction, then retain combined fragments
  contacting own objectIslands or other own brim fragments after the source
  2*spacing Round expansion. Other-object area is empty in this scope.
- Feed the resulting combined area once into the existing source-owned path
  producer and emitter. Preserve shared path/travel/E/progress/footer order.
  Width0/sub-pair collapse legitimately produces no brim.

## Deferred behavior and ownership exclusions

AutoBrim, automatic/painted inner ears, EFC-compensated outlines, supports,
towers, multi-object/multi-instance/multi-group/multi-tool composition and
cross-object contacts remain deferred. So do the rectangular public adapter
and any existing test retirement. No anchor/APA, layer-template, profile,
multiline/Clipper kernel, CLI policy, CI or web changes are included.
A RED whose first cause belongs there must be reported, not repaired here.

## Failure inventory before tests or code

1. Fixture lacks real holes or Native emits only one side despite the selector.
2. Native/API/CLI admission or single-volume grouping is not established.
3. Whole-ExPolygon outer offsets double-produce inner rings.
4. Inner width incorrectly uses the outer width modification/re-quantization.
5. Hole join/winding or source scaling differs from default Miter3.
6. Inner-only no-outer mask erases outer area, or no-inner mask erases holes.
7. Boolean subtraction/contact drops valid collapsed/split regions.
8. Multi-island append order or separate stream emission changes full bytes.
9. Existing InnerOnly/OuterOnly/ear streams or inactive controls regress.

## Test-first plan and exit conditions

1. Author input-only deterministic 3MF fixtures and
   Native-backed CLI integration tests. Proposed cases: rectangle hole,
   concave hole, two holes, narrow-neck hole, gap0.1, collapsed hole inset,
   two footprint islands in one volume, no-hole, width0, sub-pair width.
   The six hole cases plus multi-island require positive first-layer brim
   XY/E on both sides; no-hole requires outer brim, inactive widths require
   no brim. Use actual input polygons for witnesses, not bounding-box goldens.
2. Save input archive/member hashes, closed-mesh/volume provenance, effective
   Native settings, commands, full Native/API/real CLI output, first byte
   difference and regional witnesses. All three engines must admit the input
   before a runtime mismatch qualifies as RED. No Native output is a fixture.
3. Run both modes with unchanged canonical control-set-v2 and a fresh
   Native cache each run. Keep full generator-
   only comparisons; witnesses are additional evidence, never acceptance
   substitutes. Reuse all existing inner-only and N/O/E controls unchanged.
4. Commit fixtures/tests and real RED separately before writing production
   code. If a fixture is inactive or rejected,
   report/reshape it within the scope rather than removing an unrelated guard.
5. Implement only the cited boundary after that RED. Preserve every existing
   assertion/comparator; no probes, ignore, suppression, regions[0] or fallback.
   Changed Rust/new docs stay <=400 lines. No unit test is added after code.
6. Verify finite full-byte GREEN, both directed groups, strict/fmt/WASM and the
   default workspace.

Finite GREEN passes25/25 in both modes,
including all ten combined cases and fifteen unchanged controls. Each of the
nine InnerOnly controls has byte-identical API and CLI output against RED in
both modes (36 raw stream comparisons).
