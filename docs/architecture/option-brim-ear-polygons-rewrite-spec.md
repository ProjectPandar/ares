# SPEC-REQUEST: source-owned brim ear polygon rewrite

## RED

The legacy brim scaffold is addressed here after seeded E0/E1 and ear sweep
failed. Native six-case admission proves bool0/1 changes only config echo.
N0/N1/O0/O1 whole API/CLI comparisons passed. E0/E1 have genuine full-output
RED: first difference byte453/line18, Native `M73 P0 R9`, Ares `R8`; lengths
116002 versus100730. This is the first difference, not a complete root cause.

Scope: **auto btEar only**. Painted make_brim_ears is entirely
deferred to a separate source-cited, test-first slice; no painted production
code or transport/API changes belong here.

## Upstream owners and Rust destinations

Runtime correction: validated project slicing is owned
by `project_slice/gcode_emit/brim.rs::BrimPlan::generate` (currently BrimEars
returns None), not the square adapter. GREEN adds its BrimEars branch calling
the shared `brims/ears` kernel; the old adapter calls the same kernel, with no
parallel square implementation or fallback.

| Upstream rewrite slice | Rust destination / scaffold disposition |
|---|---|
| Brim.cpp:310-352 `make_brim_ears_auto` | Replace `crates/ares-core/src/brims/ears.rs::brim_ear_paths`, including square_ear_path172; generate scaled circular ear polygons, not independent square loops |
| Polygon.cpp:275-317 `filter_convex_concave_points_by_angle_threshold`, `convex_points`, `concave_points` | Ear candidate selection in ears.rs or a private ear-candidate module; reuse existing geometry types without changing global unrelated polygon behavior |
| Brim.cpp:354-419 `make_brim_ears` | Entire painted sibling deferred; no implementation in this slice |
| Brim.cpp:448-460,529-573 outer_inner_brim_area ear branches | `crates/ares-core/src/brims.rs` ear dispatch and private ear area module: flow-spacing width quantization, object-gap offsets, outer/inner differences and intersections |
| Brim.cpp:813-856 `tryExPolygonOffset` and makeBrimInfillImpl polygon-loop setup | Private ear area-to-path module under brims; use existing clipper/offset boundaries, preserve source loop ordering rather than emitting each corner independently |

Existing `brims.rs`/`ears.rs` remain temporary compatibility shells around
these upstream owners. Replace the square-ear algorithm, not wrap it in a
fallback or choose it by fixture/feature guard. Core remains byte-oriented,
platform-neutral and free of Native/process/filesystem behavior. CLI harness
owns Native admission and verifiable artifacts.

## Included behavior

- Match upstream scaled-coordinate circle construction with POLY_SIDE_COUNT,
  cos/sin vertex order, Point.hpp:197 round-to-coordinate construction and
  per-corner translation (ear size assignment itself still truncates to coord_t).
- Follow angle threshold `(180-max_angle)*PI/180`, nextafter/cos comparison,
  signed convex/concave predicates, normalized adjacent-vector direction.
- Match closed-contour Douglas-Peucker preparation and the >4-point admission
  rule in Brim.cpp:321-329; do not invent a decimation threshold policy.
- Follow positive ear size and spacing-based width quantization, exclusion of
  object footprint, outer convex and hole-side concave geometry, clipping and
  simplification/inward offsets needed to turn ear areas into upstream paths.
- Preserve source path ordering and existing emission behavior. Full-output
  estimate/footer differences must disappear through matching geometry and
  paths, not by disabling M73 or changing estimate/comparator tolerances.
- Automatic ears only: do not implement painted radius/transforms or add model
  transport fields. Their source sibling is explicitly outside this slice.

## Deferred adjacent behavior

No new GUI painting workflow, new project format, automatic adhesion-width
heuristics, multi-extruder allocation, support brim rewrite, bed clipping
redesign, global path scheduler or general perimeter/polygon refactor.
Painted make_brim_ears, its radius/transforms/model transport and EFC island
snapping are entirely deferred. No painted production code without independent
admitted tests. Automatic volume-group composition beyond the admitted cases
needs source-cited admission; do not fabricate defaults.
Reuse existing flow/geometry services; do not repair other brim modes unless
the cited shared operation is necessary and its unchanged-output controls pass.

## Failure inventory before implementation

1. Circle side count/order or float-to-coordinate conversion differs.
2. Corner angle boundary includes right/reflex angles incorrectly; winding
   flips convex/concave choice or repeated points change vector normalization.
3. Detection-length decimation removes required corners or closes twice.
4. Size/gap/spacing quantization produces wrong radius, zero/negative ears,
   wrong minimum-width behavior or double application of offsets.
5. Overlapping ears are emitted twice; footprint/hole masks are reversed;
   offset simplification changes vertex count or loop nesting/order.
6. Auto implementation accidentally introduces untested painted radius,
   transforms or model transport, violating the explicit deferred boundary.
7. Native parity is restored only in motion but estimates/footer/API/CLI differ.
8. Non-ear modes/bool echo change through shared dispatch modifications.
9. Tier1 numeric differences, increased allocations, module size >=400LOC,
   or new core filesystem/process dependencies violate implementation bounds.

## Test split and acceptance

Preserve complete original six-case RED evidence.
Mechanically separate N0/N1/O0/O1 and non-ear authored sweep as independent
always-GREEN test-only commits. Run fresh two-mode directed + strict/fmt/wasm32
for those commits. Its catalog describes
only non-ear coverage, never labels the omitted ear behavior PASS/closed.

Keep E0/E1 and ear sweep in an explicit RED commit on the production chain.
That RED does not land independently. GREEN restores all six complete
comparisons and ear sweep without skips, guards, disabled assertions or
filtered acceptance. Existing E0/E1 first-layer witnesses (443 Native brim
extrusion commands) and bool-neutrality controls remain intact. Tests for any
new admitted angle/hole/painted slice are authored before production changes;
record Native input proofs and complete outputs, not hardcoded output goldens.

Verify both standard seeded and official live Native, complete validated API
and real CLI using unchanged compare_ordered_bytes_generator_only. Run strict
Clippy/fmt/wasm32, clean workspace --no-fail-fast and
fresh BASE/CAND seeded+official exclusive full four legs.
