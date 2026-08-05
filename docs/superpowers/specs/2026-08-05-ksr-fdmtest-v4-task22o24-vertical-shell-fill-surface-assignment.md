# Task 22O.24 — Single-region vertical-shell fill-surface assignment Spec

## Status

Drafted from Ares baseline `6dde113688f369c83bca139cd51f45ca9441bdf1` against pinned OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`. Production implementation must not begin until this spec and its implementation plan each receive literal `VERDICT: APPROVE` from both the required independent reviewer and a separate default-model OpenCode reviewer. Task 22O.23 local verification, final implementation reviews, commit, and push are complete; exact-SHA Tier-1 run `30982832344` is the predecessor release gate and must finish successfully before O24 ships.

## Upstream source boundary

This milestone rewrites only the final non-debug state transition in `PrintObject::discover_vertical_shells`:

- the already-wired caller at `OrcaSlicer/src/libslic3r/PrintObject.cpp:595-596`;
- the retained O21 `polygonsInternal` construction at `PrintObject.cpp:2334-2336`;
- the retained O23 filtered-shell empty gate at `PrintObject.cpp:2399-2400`;
- `new_internal_solid`, `new_internal`, and `new_internal_void`, followed by ordered fill-surface replacement, at `PrintObject.cpp:2402-2432`;
- the exact `intersection_ex(Polygons, ExPolygons)` and `diff_ex(SurfacesPtr, ExPolygons)` overloads, two-pass NonZero Paths-to-PolyTree execution, and PolyTree-to-ExPolygon traversal at `ClipperUtils.cpp:170-216,640-667,737-810` and `ClipperUtils.hpp:157-216,265-284,442-455,509-520`;
- collection-order filtering, stable `keep_types`, ordered append behavior, and the exact non-bridge classification of `InternalVoid` at `SurfaceCollection.cpp:45-60,84-95`, `SurfaceCollection.hpp:74-81`, and `Surface.hpp:9-32,42-79,108,120-156,247-280`.

The smallest coherent boundary is the complete block through line 2432. Stopping after any `new_*` temporary would not port an upstream state transition. The exact non-debug source order is:

1. `new_internal_solid = intersection_ex(polygonsInternal, regularized_shell)`;
2. `new_internal = diff_ex(filter_by_type(stInternal), regularized_shell)`;
3. `new_internal_void = diff_ex(filter_by_type(stInternalVoid), regularized_shell)`;
4. stably retain existing `Top`, `Bottom`, and `BottomBridge` surfaces in their original collection order;
5. append all `new_internal` results as `Internal`;
6. append all `new_internal_void` results as `InternalVoid`;
7. append all `new_internal_solid` results as `InternalSolid`.

All three booleans read the original, unmodified collection. The void difference is a real ordered call even when its subject is empty. All use default `ApplySafetyOffset::No`, NonZero fill, integer coordinates, first-pass flat Paths, second-pass NonZero Union-to-PolyTree, and existing Clipper output order. No safety offset, floating arithmetic, sorting, canonicalization, deduplication, or pre-union is introduced.

The Rust destination is a crate-private `prepare_infill::vertical_shell_assignment` successor after `PreparedPostVerticalShellFiltering`. It mutates only the retained `PreparedSurfaceTypeRecord::fill_surfaces` after whole-project staging, retains the exact O23 predecessor and sidecar graph, and adds no durable independent pipeline sidecar. O19-O24 state remains a temporary compatibility representation of `PrintObject::discover_vertical_shells`, not an Ares-owned slicing design.

The exact stop is after `PrintObject.cpp:2432`, before the loop/function close, cancellation/logging, all later horizontal-shell/bridge-over-infill work, and every fill/toolpath/G-code stage.

## Active envelope and provenance

O24 retains the reviewed O17-O23 envelope: global spiral is rejected before O17; each object has exactly one compatible region; `interface_shells = false`; active extra-bridge modes remain rejected; and only `ensure_vertical_shell_thickness = EnsureAll` can produce a nonempty filtered shell. Every option and scale remains derived from typed resolved 3MF state. O24 introduces no option or source constant.

For every populated aligned record:

- derive `polygonsInternal` only from the current pre-mutation `fill_surfaces`, in collection order, selecting `Internal`, `InternalVoid`, and `InternalSolid`, and flattening each ExPolygon contour before holes;
- derive the `Internal` and `InternalVoid` ExPolygon subjects only from current pre-mutation surfaces of exactly that kind, preserving surface order and each ExPolygon's contour/hole order;
- derive the clip only from the O23 `filtered_shell`, never from unfiltered O22 regularization;
- when `filtered_shell` is empty, preserve the complete record and every allocation unchanged and invoke no O24 event or geometry;
- never branch on fixture identity, filename, hash, dimensions, layer count, geometry identity, or reference G-code.

`RegionSurfaceKind` gains only the source-cited representational value `InternalVoid = 8`. Its exhaustive `is_bridge()` behavior must return `false`, matching upstream `Surface::is_bridge()` at `Surface.hpp:108`; no default arm may hide later vocabulary changes. The existing O18 `infill_only_where_needed = false` envelope means the real KSR input currently produces no void surfaces, but O24 must correctly consume, subtract, rebuild, order, and emit explicitly constructed void surfaces. A production `InternalVoid` producer and the adjacent upstream bridge kinds remain deferred to their owning source milestones. The shared O21/O23 `polygonsInternal` helper must include the new representable void kind, matching the retained upstream value used by O21 through O24.

## Included behavior

### Exact mixed boolean adapters

Add only the missing crate-private mixed adapter required by line 2402: flat `&[Polygon]` subject plus `&[ExPolygon]` clip to `Vec<ExPolygon>` intersection. It must add subject Paths directly as `PathRole::Subject`, add clip contours and holes as `PathRole::Clip`, and reuse the existing two-pass NonZero `_ex` execution. Do not reinterpret flat hole paths as standalone ExPolygons merely to call the existing ExPolygon/ExPolygon overload.

Use the existing ExPolygon/ExPolygon `difference_ex` for the two source collection differences after collecting subjects in exact collection order. The operation sequence is observable and cannot skip the empty-void call.

Every Clipper failure maps to exactly:

`SliceError::InvalidInput("vertical-shell fill-surface assignment geometry is outside the supported Clipper range")`.

Test-only failure injection is allowed at exactly the three real geometry sites: solid intersection, internal difference, and internal-void difference.

### Whole-project stage-before-move

Validate complete O23 alignment before the first O24 event: outer objects, O18 objects, O19 caches, O20 projections, O21 trims, O22 regularizations, O23 filters, traversal objects, record counts, plan layers, inputs, prelude records, `Some`/`None` slots, source/transform identity, planned layer/index IDs, current layer/region, region ID, one compatible region, and retained scale relations.

Borrow O23 and stage each active record's three fresh ExPolygon result vectors in stable object/record order. An empty O23 filter stages an explicit no-op. Do not modify any `fill_surfaces` until all objects and slots have completed all three fallible booleans. On any failure, expose no successor and iteratively dispose the exact O23 state.

After all staging succeeds, move the exact O23 graph and commit records in source order:

1. no-op records are not touched;
2. active records stably retain only existing `Top`, `Bottom`, and `BottomBridge` values, preserving their relative order, ExPolygon storage, and all metadata;
3. append staged `Internal`, `InternalVoid`, and `InternalSolid` groups in that exact category and Clipper-output order;
4. every appended surface receives source defaults: thickness `-1.0`, thickness layers `1`, bridge angle `-1.0`, and extra perimeters `0`;
5. all old non-external surfaces, including old `InternalSolid` and any future representable internal kind, are removed;
6. `slices`, perimeters, thin fills, `fill_expolygons`, `fill_no_overlap_expolygons`, predecessor state, and O19-O23 sidecars remain unchanged.

The outer objects/records and all O19-O23 sidecar allocations retain identity. An active record's `fill_surfaces` vector buffer may remain or reallocate, matching source `keep_types` plus repeated reserve semantics and not forming an identity guarantee. Retained external inner geometry remains nonaliasing and unchanged; all rebuilt internal geometry is fresh relative to predecessor fill geometry and O23 clip geometry.

Wire public slicing through O24 exactly once, dispose O24 iteratively, and continue returning `ProjectSlicingIncomplete`.

## Explicitly deferred

- debug SVG blocks at `PrintObject.cpp:2403-2414,2420-2426`;
- a production producer for `InternalVoid`, `InternalAfterExternalBridge`, `InternalBridge`, or `SecondInternalBridge`;
- cancellation, TBB scheduling, logging, profiling, and loop/function-close mechanics after line 2432;
- multi-region/material/interface-shell behavior outside the already reviewed constrained envelope;
- horizontal-shell discovery, bridge-over-infill, fill generation, seams, ordering, motion, G-code, and post-processing;
- reference-G-code reads/replay, fixture identity/name/hash/layer-count/geometry branches, Orca runtime/FFI, legacy fallback, or hard-coded fixture output.

## Tests and acceptance

### Direct topology and source-order witnesses

1. Freeze the new flat-Paths/ExPolygons intersection adapter for empty, disjoint, partial, fully covered, multi-component, holed, mixed-winding, and nested-island inputs. Freeze exact ExPolygon/contour/hole/point order and distinguish direct flat subject Paths from any guessed ExPolygon conversion that normalizes each standalone path to contour winding, plus an intersection-to-difference operation change, EvenOdd fill, flat output, safety offset, pre-union, sorting, and canonicalization. Role-only operand reversal under identical NonZero fill is an empirically equivalent commutative control and must not be claimed as a killed mutation. Mere wrapper types that feed byte-identical ordered Paths are likewise observationally equivalent; direct-provider use remains a manual implementation review requirement.
2. Freeze Internal and InternalVoid differences for empty/disjoint/partial/full cover, multiple surfaces, holes, and nested islands. Preserve original subject surface order and exact two-pass PolyTree result order.
3. Freeze the exact event sequence for every nonempty filter: solid intersection, internal difference, internal-void difference. Inject failure at each call and require exact prefixes, no later event, stable error, no successor, and iterative predecessor disposal.
4. Prove a nonempty filter executes all three calls when `polygonsInternal`, Internal, or InternalVoid subjects are empty. Prove an empty filter executes none and preserves the complete record and allocation snapshot.

### Assignment semantics

5. Use a pre-mutation collection interleaving Top, Internal, Bottom, InternalVoid, InternalSolid, BottomBridge, and further internal surfaces. Require the original stable external subsequence followed by all new Internal, then InternalVoid, then InternalSolid results. No old internal value survives.
6. Freeze retained external metadata and inner allocation identity. Freeze exact default metadata and fresh/nonaliasing geometry for every appended group. Do not assert active outer fill-vector pointer identity.
7. Freeze `InternalVoid as u8 == 8` and `InternalVoid.is_bridge() == false`, alongside the existing exhaustive discriminant/bridge vocabulary witness. Prove `polygonsInternal` includes Internal, InternalVoid, and InternalSolid in collection/contour/hole order, while each difference selects exactly one kind. Add shared O21/O23 regressions for the newly representable void kind.
8. Prove every unrelated record field, predecessor allocation, and O19-O23 sidecar allocation/content remains unchanged.

### Alignment, transactionality, cleanup, and lifecycle

9. Fail every inherited O23 outer/count/slot/identity/plan/input/region alignment relation before the first O24 event.
10. Use genuine later active-slot and later-object failures after earlier complete staging to prove no record is committed early. Preserve exact operation prefix, no partial successor, and predecessor drop probes.
11. Preserve iterative cleanup for both 10,000-node predecessor tree families on direct success, all three injected failures, and public-incomplete disposal using only the shared Unix/non-Windows 64 KiB and Windows 256 KiB constrained-stack baseline.
12. Public slicing reaches O24 exactly once after O23 and remains incomplete. Every earlier capability or O17/O19/O20/O21/O22/O23 failure invokes O24 zero times and preserves exact error precedence.

### Typed provenance, KSR, and metamorphic witnesses

13. Parse the real KSR archive independently twice. Reassert the frozen O19-O23 parent evidence, then freeze an O24 geometry+metadata checksum, kind/ExPolygon/contour/hole/point totals before and after assignment, unchanged-record count, active-record count, and ordered three-event totals. Tests never read reference G-code.
14. Record honestly that KSR has zero produced InternalVoid surfaces while direct synthetic tests prove exact void behavior. No production branch may depend on that zero.
15. ZIP entry order/compression/timestamps, non-slicing rename, inactive ensure modes, typed model-part option precedence, printable-area scale selection, and component-transform scaling preserve or change O24 only through normal typed predecessor geometry. Inactive modes preserve complete records exactly.

### Repository gates and review loop

16. Focused O24 tests, explicit O10-O24 regressions, workspace Nextest, native all-target check, strict all-target/all-feature Clippy, four Tier-1 WASM checks, optimized default/feature browser-WASM builds and export audit, both Playwright runs, formatting, diff, dependency, staging, rollback, and forbidden-pattern audits pass.
17. Every Rust file remains below 400 LOC and each new O24 shard is at most 300 LOC. New Rust contains no `unsafe`, `include!`, `include_bytes!`, broad lint allowance, binary oracle payload, reference-G-code access, fixture identity/hash/layer/geometry branch, Orca command/FFI, or fallback. Tests use real files and ordinary `mod`; tests must not read, parse, hash, grep, or line-pin Orca/Ares source text.
18. Required compiling behavioral mutations must be killed by intended witnesses: wrong `InternalVoid` discriminant or bridge classification, standalone subject-path conversion with forced contour-winding normalization, intersection changed to difference, skipped/reordered void call, safety offset/fill-rule change, keep-before-booleans, unstable/grouped external retention, append group reordering, metadata inheritance, empty-filter rebuilding, public-wiring bypass, alignment bypass, and truncated staging. Role-only operand reversal and byte-identical wrapper-only conversions are excluded as equivalent controls and remain covered by manual review. Restore final production byte-exactly before final GREEN.
19. Independent and default-model OpenCode spec and plan reviews must both return literal `VERDICT: APPROVE` before implementation. After implementation, an independent six-dimensional reviewer and separate default-model OpenCode reviewer evaluate requirements completeness, logic correctness, boundary cases, code quality, test coverage, and actual execution. The parent sole writer fixes every blocker and repeats both reviews until approval.

## Documentation and rollback

After final evidence is frozen, update `docs/architecture/option-parity-v4.md`, `docs/roadmap.md`, this spec, and the plan with exact checksums/totals/events/test counts and the next source boundary after completed `PrintObject::discover_vertical_shells`.

O24 adds no public API, persisted format, dependency, migration, fallback, or independently designed pipeline. Mechanical rollback restores O23 as the public terminal consumer and removes only the O24 module/state/wiring/tests/docs, the narrow mixed intersection adapter/export/tests, the private `InternalVoid` representation/exhaustive-match updates, and the O21 helper's void selection. It retains all O23 filtering and pre-existing geometry behavior unchanged.

## Final implementation evidence

The implemented successor preserves the exact three-call order and pre-mutation
subjects, performs whole-project staging before stable collection replacement,
checks the inherited selected scale against typed printable area before the
first geometry event, and iteratively owns the exact O23 graph. Added synthetic
witnesses freeze disjoint/full-cover/multiple/holed/nested Internal and
InternalVoid behavior and prove that O23 closing/protection consumes newly
representable InternalVoid Paths.

Repeated KSR capture freezes checksum
`-117597382518472843802490205604634875775`, pre/post kinds
`[113, 6, 48, 1127, 0, 0]` / `[113, 6, 48, 1281, 575, 0]`, pre/post geometry
`[1294, 168, 46011]` / `[2023, 270, 73848]`, 460 total records, 161 active,
299 no-op, 299 unchanged, and 299 unchanged no-op records. The digest stream
explicitly tags object/slot positions, record/surface boundaries, path counts,
contour/hole role and index, point counts, and end markers. The delimited record
sequence digest is `-65994586923856785425316699963519338136`; the exact event
sequence digest is `-110138798119262824097709645699717637653`, with operation
totals `[161, 161, 161]`. Real KSR InternalVoid counts remain `[0, 0]`.

Thirty-one focused tests pass: nine production-adjacent assignment/adapter
tests, three shared InternalVoid helper/filter tests, and 19 project lifecycle,
transaction, ownership, cleanup, provenance, metamorphic, and KSR tests.
Thirteen planned compiling mutations plus the reviewer-added retained-scale
mutation are killed by intended witnesses; role-only intersection reversal is
an explicitly equivalent commutative control, not a claimed RED. The release
gate includes 149 O21-O24 regressions and 5,827 workspace tests passed with 2
skipped, plus native,
strict Clippy, four WASM checks, optimized browser-WASM/export audit, two
Playwright runs, both implementation rereviews, and exact pushed-SHA Tier-1.

The next source boundary is `PrintObject::prepare_infill` line 618 and
`PrintObject::discover_horizontal_shells` at `PrintObject.cpp:3955-4161`.
