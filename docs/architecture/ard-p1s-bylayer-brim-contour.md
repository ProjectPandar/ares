# ARD: P1S ByLayer cross-object no-brim area subtraction

Status: **revised SPEC / REVIEW-REQUEST / HOLD**. W5 owns brim.
Coordinator authorized this docs-only revision after materialization-boundary
STOP; GREEN awaits approval. Approved RED `61b15c6a8` is unchanged.
Base: frozen rib3 `a96cf4d033b44ed9418975f1f68ac8118710a97e`.
Upstream pin: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Corrected owning upstream boundary

`OrcaSlicer/src/libslic3r/Brim.cpp::outer_inner_brim_area` owns this slice,
not `tryExPolygonOffset` or `makeBrimInfillImpl`:

1. **448** scales each object's `brim_object_gap` into `brim_offset`.
2. **532–545** builds the object's OuterOnly ring: round-offset contour by
   gap, round-offset that result by quantized width, subtract inner from outer.
3. **569–570** computes `objectIsland = offset_ex(*brim_slices, brim_offset,
   jtRound, SCALED_RESOLUTION)` and appends it to `no_brim_area_object`.
4. **573–578**, especially **576**, translates every instance's no-brim
   polygons into the shared **print-wide `no_brim_area`**. The translation
   helper at **27–34** uses `shift_without_plate_offset()`.
5. Only after the object/extruder collection loops finish, **661–679** visits
   each object: copies the complete no-brim collection at **662**, adds its
   extruder-specific bed mask at **668–673**, then **679** executes
   `diff_ex(brimAreaMap[object->id()], extruder_no_brim_area)`.
   **This subtraction creates the witnessed eight notch vertices.**

The first brim's straight left edge intersects the other object's gap-expanded
island: two intersection points plus six rounded right-edge points produce
its staircase. It is not an inner-hole brim mode or an emission omission.
`make_brim` **890–904** calls the area builder; **963–969** consumes the
resulting maps through `makeBrimInfill`. Materialization **813–877** is a
consumer, not the missing operation. Ring ownership allocation at **36–45**
and contact retention at **688–715** remain existing adjacent behavior.

## Evidence correcting the original hypothesis

Evidence root: `/home/indexyz/ares-coord/evidence/w5-p1s-bylayer-brim-green`.
Read-only CPU11 GDB entry traces, no production instrumentation:

- `input-pretty-2.log`: Ares `from_area` first input = contour52 + hole12.
- `native-input.log`: seeded Native `tryExPolygonOffset` first input =
  contour60 + the identical hole12; its second area exactly equals Ares's.
- `boundary-comparison.json` / `compare-inputs.py`: all eight extra points
  belong to Native's first contour; no Rust-only points. Notch intersections
  are x112766710 at y135028552/114828552; right edge x115537500 at
  y134928552/114928552. These are diagnostic input geometry, not test goldens.
- Those right-edge coordinates are the neighboring rectangle's x115437500
  edge plus the configured 100000-unit gap; its top/bottom are likewise
  gap-expanded. The initial large rectangular ring therefore must lose this
  neighbor island before offset-loop materialization starts.

The trace establishes that the old **813–856 → paths.rs** hypothesis cannot
repair this witness faithfully: the notch is already missing at its input.
The precise preceding operation is identified by the source collection and
subtraction chain above. No claim is made that every compound timing residual
is explained or that every area-building mode has now been qualified.

## Rust comparison and destination

`crates/ares-core/src/brims/outer.rs::area` **24–41** already implements the
local ring and gap-expanded object island. It returns `area`, `forbidden`,
and `object_islands`; changing its local offset arithmetic would target the
wrong boundary. Its per-object result is a producer, not the global reducer.

The mismatch is at the current consumer boundary:

- `project_slice/gcode_emit/brim.rs::generate_all` **13–35** calls `generate`
  to completion for each owner; **290–303** passes only that owner's expanded
  forbidden areas immediately into `bed_area::clip_plate`.
- `brim/owner.rs::expand` **47–61** translates only that owner's masks.
  `accumulated` is previously assigned **brim area**, not print-wide no-brim
  geometry; it cannot substitute for the upstream `no_brim_area` collection.
- `brim/bed_area.rs::clip_plate` **41–96** adds tower/bed exclusions then
  subtracts the supplied forbidden polygons. It currently never receives
  the complete cross-object island masks in this uncombined-owner path.

**Revised destination:** existing private `BrimPlan::generate_all/generate`
collection/finalization boundary in `project_slice/gcode_emit/brim.rs`,
instance mask translation in `brim/owner.rs`, and existing subtraction consumer
`brim/bed_area.rs::clip_plate`. Reuse `brims/outer.rs` as the unchanged local
producer. These are temporary compatibility shells for upstream
`outer_inner_brim_area` maps and collection phases, not a new Ares pipeline.

## Proposed included rewrite and invariants (not yet authorized GREEN)

Collect the existing translated no-brim outputs across owners before any
owner's final forbidden-area subtraction/materialization. Preserve upstream
ordered brim ownership assignment independently; do not conflate assigned
brim area with no-brim area. Supply the complete no-brim collection to each
existing per-owner clip operation, retaining per-extruder bed and tower masks.
Reuse each existing producer's mask semantics; do not introduce fixture,
printer, coordinate, or print-sequence special cases.

No changes to local outer/inner/ear width heuristics, offset kernels, shared
Clipper, contact-retention rules, grouping/registration, G-code emission,
paths.rs materialization, Type1 rib3 conversion, public API or comparator.
If further localization requires one of those changes, STOP and amend scope.
Module splits only as needed to keep edited Rust modules ≤400 LOC. Core
remains filesystem/terminal-free and portable to browser WASM/Windows/macOS/Linux.

## Deferred behavior and exit gate

Deferred: new auto/ear/support algorithms, volume grouping, shared geometry
repair, combined-brim ownership changes, timing/M73 corrections and unrelated
tower/skirt behavior. Existing behavior is retained and covered by neighbors.
No legacy fallback, mask, normalizer extension or lint allowance.

Approved public RED remains unchanged: `native_stream_case::check` requires
Native, qualifies two Native replays, then compares complete Native/API/CLI
streams with the existing comparator and checks raw API==CLI. There are no
embedded Native motion-line goldens or exact motion callbacks. This explicitly
supersedes the original ARD requirement for Native coordinate assertions.

Baseline selector `test(brim) | test(p1s_bylayer_brim_)` lists **150** tests
(`selected-tests.txt`), covering target/control and existing brim neighbors.
Fresh official then seeded runs, CPU11/taskset/threads1: **149 PASS, target
FAIL** in each mode. These are unchanged-code baselines, not GREEN evidence.

After revised SPEC approval only: implement the bounded collection/subtraction
slice, rerun the same nonempty selector in both modes with fresh caches,
retain complete streams/commands/statuses/raw diffs/hashes. Full comparator
includes timing/M73. Any residual → STOP/report, never broaden scope or mask.
Then REVIEW-REQUEST/HOLD; FULL, merge, rebase and push require authorization.
