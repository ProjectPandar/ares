# Ordered cross-owner brim area subtraction rewrite SPEC

Status: **SPEC REVIEW REQUEST — docs-only; RED awaits approval**.
Rebased on the beta authority `345efec64c7b95a05ce5cf96df0dde66a158835c`
(OrcaSlicer 2.5.0-beta); every line number and semantic below was re-checked
against that tree. Differences vs the original 2.4.2 basis (source
`8500fcdccaa10b5099ac20d252af3a7c560046f1`) are recorded explicitly in the
"Rebase differences" section.
Base and explicit dependency: integrated main
`6bba2bdda` (branch `processor/b19-cross-owner-brim-subtraction`).
This is an independent repair of an already-supported uncombined brim path.
The per-object combined-brim contact-component gap remains suspended.

Slice order (coordinator decision): the **per-instance brim granularity**
slice lands first as the prerequisite — beta's `brimAreaMap` is
instance-keyed and the claimed-area loop (`Brim.cpp:953-958`) reads that
keying directly — then the subtraction ports as the verbatim map-level loop.
The subtraction is not landed on an intermediate owner-keyed shell.

## Owning upstream boundary (beta 345efec64)

Source checkout: `345efec64c7b95a05ce5cf96df0dde66a158835c`, paths below
under `OrcaSlicer/src/libslic3r`.

| Source | Behavior being preserved or ported |
|---|---|
| `Brim.cpp:69-96::append_and_translate` (three overloads) | The `brimAreaMap` overload (:78-84) translates the candidate by the instance shift and appends it whole — **no subtraction here in beta** — keyed `{print_object->id(), instance_idx}`. |
| `Brim.cpp:458-781::outer_inner_brim_area` | Per-extruder visitation over `objPrintVec` with `brimToWrite` written-state tracking (:485-527); candidate generation per object (:529-618); per-instance map append loop (:619-629); cumulative `brim_area` extension only after the instance loop (:631-633). |
| `Brim.cpp:695-722` masks | `extruder_no_brim_area` bed-exterior diffs (:701-705) and active bed-exclusion footprints (:713-717) applied per map entry (:718-722), after assignment. |
| `Brim.cpp:726-766` contact retention | Per map entry: keep fragments whose 2-spacing offset intersects `objectIslands`, another fragment of the same entry, or any other entry's area (cross-owner and cross-instance alike); survivors extend `brim_area`. |
| `Brim.cpp:875-925::makeBrimInfillImpl` (+`apply_plate_offset` wrappers :920-925) | Path generation from a single area after assignment/masking; stepping, outside-in chaining, reversal and `connect_brim_lines` unchanged in kind. |
| `Brim.cpp:941-948` | Belt printers return before any flat-plate brim work. |
| **`Brim.cpp:953-958`** | **The ordered cumulative subtraction, the subject of this slice**: when `!combine_brims`, iterate `brimAreaMap` in `std::map` key order (`ObjectInstanceID{ObjectID, instance_idx}` ascending), `areas = diff_ex(areas, claimed_area)`, then append the remainder to `claimed_area`. Earlier keys keep overlapping area. |
| `Brim.cpp:961-996` | `firstLayerObjectBrimBoundingBox` per object (:961-982); `islands_area` plate shift and `objectBrimAreasByInstanceOut` skirt-grouping footprints (:984-996). |
| `Brim.cpp:1004-1010` | Uncombined generation is per map entry (per instance): `brimMap[object_id].append(...)` plus `brimMapByInstance.emplace({object_id, instance_idx}, ...)` — the emission data is per-instance. |
| `Print.cpp:3812,3839-3848` | `print_object_instances_ordering = chain_print_object_instances(...)`; `objPrintVec` = first-seen dedup over that chain. |
| `Print.cpp:3849-3867` | `make_brim` call wiring (`m_brimMap`, `m_brimMapByInstance`, `objPrintVec`, `printExtruders`, `m_objectBrimAreasByInstance`); `islands_area` Douglas-Peucker and first-layer convex hull. |
| `GCode.cpp:5991-6082::generate_object_brim` | Per-instance emission: `ObjectInstanceID{object.id(), instance_id}` (:6065), belt path (:5998-6037), flat path consumes `SkirtBrimGroup` brims whose instance lists contain the key, dedup via `m_objsWithBrim`, owner-filament wait for split bed exclusions (:6039-6044). |
| `ShortestPath.cpp:2533-2576` (`chain_print_object_instances`), with `:64-175,418` chain machinery, `:1007-1011` KD batched updates, `KDTreeIndirect.hpp:34-165` and `MutablePriorityQueue.hpp:60-260` | The instance chain that orders `objPrintVec` and emission; scaled f64 distances, KD visit/pruning and heap mutation order are observable. |

The new rewrite slice is specifically the previously missing **ordered cumulative
area subtraction** at `Brim.cpp:953-958`, not another offset or brim-width
formula and not a new Ares pipeline. The remaining rows constrain where the
port belongs in the existing upstream sequence; they are not authorization to
redesign adjacent behavior.

## Rebase differences vs the 2.4.2 basis

Recorded explicitly, per the rebase instruction; every item below was verified
by reading the beta tree.

1. **The subtraction moved out of `append_and_translate`.** 2.4.2 diffed each
candidate against the accumulated area at append time inside the owner loop.
Beta appends whole per-instance areas into `brimAreaMap` (`Brim.cpp:78-84`)
and subtracts once afterwards, in `make_brim` (`:953-958`), gated on
`!combine_brims`.
2. **Subtraction priority is `std::map` key order, not `objPrintVec` order.**
`brimAreaMap` keys are `ObjectInstanceID{ObjectID, instance_idx}` and the
claimed-area loop iterates the map ascending. `objPrintVec` (built from the
chained `print_object_instances_ordering`, `Print.cpp:3812,3839-3848`) still
drives generation visitation and the written-state tracking; it no longer
encodes subtraction priority. The port must reproduce both orders as-is.
3. **Areas, infill and emission are per-instance in beta.** `brimAreaMap` keys
include `instance_idx`; uncombined generation fills `brimMap` AND
`brimMapByInstance` (`Brim.cpp:1004-1010`); `generate_object_brim` takes an
`instance_id` and keys emission on `ObjectInstanceID` (`GCode.cpp:5991-6082`).
The 2.4.2 basis described owner-level collections. The per-instance emission
envelope is shared with the separately routed per-instance granularity slice;
this SPEC consumes it only as the already-existing upstream shape.
4. **Masks and contact retention now run inside `outer_inner_brim_area`,
after assignment and before the subtraction** (`:695-722`, `:726-766`):
per-extruder bed-exterior diffs, active bed-exclusion footprints, then
per-entry contact retention against islands, same-entry fragments and other
entries' areas. Retention therefore sees pre-subtraction areas.
5. **`makeBrimInfillImpl` gained `apply_plate_offset`** wrappers (`:875-925`).
6. **Beta-only additions in the same function, all out of this slice's scope**
but present in the sequence: belt-printer early return (`:941-948`),
`firstLayerObjectBrimBoundingBox` (`:961-982`), `objectBrimAreasByInstanceOut`
skirt-grouping footprints (`:991-996`), islands Douglas-Peucker + plate shift
(`Print.cpp:3863-3866`).
7. **Chain citation updated.** `chain_print_object_instances` lives at
`ShortestPath.cpp:2533-2576`; the machinery ranges cited by the approved
revision (`92-418, 1007-1011, 2015-2041` on 2.4.2) sit at `64-175` and the
`:418` union-find merge in beta; `KDTreeIndirect.hpp` and
`MutablePriorityQueue.hpp` are whole-file ports whose 2.4.2 sub-ranges
(62-69, 114-165, 229-243 / 76-197) are subsumed by the existing Ares
`perimeters/classic/shortest_path` port — reuse, not re-implementation.
8. **Deferred combined-brim cite moves** from `Print.cpp:2898-2990` to the
`_make_skirt` skirt/brim-group construction (`Print.cpp:4231-4400+`, incl.
`make_brims_for_skirt_brim_group` `:4297+`). Scope unchanged: deferred.
9. The frozen Native observations below were recorded against the same pinned
merged oracle used today (numbers like 407/471 and first-divergence anchors
are historical observations, not assertions); RED re-derives them at runtime.

## Rust destination and non-negotiable decisions

Destination: `crates/ares-core/src/project_slice/gcode_emit/brim.rs` generation
sequence and `brim/owner.rs` area assignment. Existing per-object `BrimPlan`
preparation is a temporary compatibility shell around upstream's area maps.

Current `owner::expand` only translates/concatenates one owner's instances.
`BrimPlan::generate` subsequently clips and infills each owner independently;
no previous-owner area reaches it. The orchestration caller iterates independent
plans before `combine`, which is disabled for this supported per-object case.
This loses `append_and_translate`'s dependency on the cumulative area.

Required preparation semantics:
1. Retain candidate areas in the upstream instance/plate coordinate frame.
2. Follow the existing source-owned object/extruder order corresponding to
   `objPrintVec`, not an invented ordering. Existing
   `layers/object_order::chain_without_start` / `Print.cpp:3812,3839-3848`
   mapping must be audited and reused where it is the matching upstream
   carrier. In beta the chain orders generation visitation and emission;
   subtraction priority is the `brimAreaMap` key order (see rebase item2).
3. Subtract **earlier claimed area** in `brimAreaMap` key order from each
   entry; extend the claimed accumulator as upstream does before later keys.
4. Keep masks/contact retention before the subtraction and path generation on
   subtracted areas, in the cited order (`Brim.cpp:695-766` then `:953-958`
   then `:1004-1010`). Do not subtract paths, covered convex hulls,
   independently generated loops or already-clipped substitutes.
5. Preserve occupied-area consumers and owner identity; no global combined
   carrier or new skirt-group admission as a shortcut.

If minimal call-site wiring is needed to supply generation order, document it
for review before implementation; this SPEC does not authorize broad changes to
orchestration, time estimation, path routing or test assertions.
Core remains platform-neutral and byte/API-oriented across browser WASM,
Windows, macOS and Linux; no I/O or terminal behavior enters core.
No feature flags, fallback, new Unsupported guard, ignore/allow/expect or magic
result constants. Keep touched modules at most400LOC with only necessary splits.

## Included / deferred

Included witness envelope: ByLayer; two separately identified objects, one
instance each; one PLA filament; classic walls; explicit OuterOnly brim width5,
gap0; two per-object skirt loops, height2, minimum length0; no active support,
raft, tower, shield, local speed overrides or EFC. Existing GUI positions are
fixed at X80/88 near, X80/94 middle, X80/140 far, Y110.
The only added behavior is upstream ordered cross-owner area subtraction before
uncombined infill. Existing upstream owner/path consumers remain unchanged.

Deferred: `Print.cpp:4231-4400+` (`_make_skirt` skirt/brim-group construction,
incl. `make_brims_for_skirt_brim_group` `:4297+`) per-skirt-group contact-
connected combined brims, new combine admission, repeated-copy/ByObject
expansion, active support/tower, multi-material/extruder expansion,
AutoWidth/EFC/material/speed changes, new mask/retention algorithms, loop
chaining, travel/seam/fill/ironing changes, belt-printer brim and
bed-exclusion volume behavior beyond passing through existing config.
No time-estimator or M73 correction is included: progress is a strict observable
of the repaired complete stream, not a value to tune independently.

## Fixed input provenance and Native observations

External input evidence retained outside the repository
(directed fixture-discovery archive, 2026-10-21).
Inputs derive from tracked
`tests/parity/per-object-auto-brim-admission/{near,far}-perobject-auto.3mf`.
Source hashes respectively:
`33879016885adcc85533e345a52a46d6ef4fbe74dd63156305f07bc1981d2d6a`,
`339a797d71afc111b8857aae4db0ef405eb6a1534f1404a9a6762395651d4390`.
Generators retain mesh/object/material identities and set explicit brim width5,
OuterOnly/NoBrim, combine0 for brim witnesses; no-brim controls retain their
existing combine1 setting, which is inactive without brim area. Middle is the
already-generated GUI placement variant, not a new position search.
GUI route: choose brim type/width, use per-object skirts, move two objects.

Proposed frozen RED files (all referenced, input-only; no Native output):

| Role | Existing external archive | SHA256 |
|---|---|---|
| target | `inputs/near-uncombined.3mf` | `c73b006dee07894432046ef2b16724824e788bd2e75ab219338636870638ad56` |
| far control | `inputs/far-uncombined.3mf` | `c9f9c760caedd4fc2118e491b0a3df667346c6b4d2435ef4f752c8c87c4929a6` |
| middle control | `middle/inputs/middle-uncombined.3mf` | `c44fb386d4c0bff36e23a643c6e112182552497f4dc7f83605465f8af4e1dbf2` |
| near no-brim | `inputs/near-no-brim.3mf` | `6b2e46a061dee196d1a2a837e1cbd0c2502001bee06eca0ac65732f81dac382f` |
| far no-brim | `inputs/far-no-brim.3mf` | `71e2690d381d7698a94ce5c6d53eb2da5359fe00de18029c26c10a8b707a1695` |
| middle no-brim | `middle/inputs/middle-no-brim.3mf` | `be1176b4cb398beffd797898ebed6844be8e665cb52d63160767c4a743aafde4` |

Both official/seeded Native accept all6, actual CLI succeeds on all6.
Five controls match strict generator-line-only whole output in both modes.
Near target fails strict first byte2774 at the placement of M73P27R2.
First470 positive-XY Brim commands match. Second Brim block has407 Native vs471
Ares commands. First actual motion divergence is zero-based motion605/source682,
second brim entry: Native X83.759Y113.35 vs Ares X84.906Y110.074.
There are two independently emitted Brim blocks in both target streams: this
is not the suspended combine2→1 block-count behavior.

## Why no M73 repair or mask

Independent read-only diagnosis evidence retained outside the repository
(residual-mismatch archive, 2026-10-21).
Original instruction prefix through the early progress mismatch is identical.
Same existing C++ processor applied to both original G-code streams reports
anchor elapsed50.00978956s for each, but final totals185.065676/185.608442s;
`GCodeProcessor.cpp:1251-1257` uses final total in percent calculation, yielding
27/26 at that anchor. Future extra brim motion explains earlier progress shift.
Auxiliary processor metadata/resource limitations are recorded in its RR; it
is diagnostic evidence, not replacement official slicing or Ares runtime data.
All acceptance comparisons must retain M73 and every motion/metadata byte.

## SPEC → RED → GREEN review gates

1. Submit this docs-only SPEC RR; no test or production edits before approval.
2. RED reuses **`runner::native_stream_case::check` unchanged**, six frozen cases
   above, namespace/source references plus SHA. Single tool, so assert_two_tools
   is false. Its Native double replay must pass before public API and real CLI
   strict complete output comparisons and raw API/CLI equality.
   Do not copy a harness or modify helper/comparator. Native actual brim/skirt
   and407/470 observations are retained as external Native-first evidence, not
   hand-coded expected result constants or private-geometry assertions.
3. Expect target strict FAIL and5control PASS in each mode. Report exact RED,
   Native replay and API/CLI statuses before requesting GREEN authorization.
4. GREEN only production area preparation within the approved boundary; tests,
   fixture inputs, helper and assertions freeze. Strict residual STOP: report
   first full-output difference, do not widen into M73/time/travel/combined work.
5. Directed target/controls plus neighboring brim/skirt regression in both modes,
   fmt, workspace all-target clippy with -Dwarnings, wasm32. Canonical dual FULL
   only in a future coordinator-allocated window, exact tip after rebase.

All present work is docs-only; no FULL/build/source changes.
Coordinator owns review/merge. Combined-brim SPEC remains suspended until this
supported uncombined control is repaired and independently reviewed.

## Approved revision: reuse one source-faithful instance chain

Coordinator authorized this revision after the attempted owner subtraction hit
strict residual; attempted `020062154` is **not** an accepted GREEN. Runtime
probe evidence is external, retained outside the repository
(runtime order-probe archive, 2026-10-21).
Native and Ares have the SAME ordered scaled reference points
`[(80250000,110000000),(88250000,110000000)]`, but unseeded outputs are
Native `[1,0]`, Ares `[0,1]`. Native `objPrintVec` confirms that owner priority.
The mismatch is in the duplicated instance-chain algorithm, not slot mapping.

Owning source (beta): `ShortestPath.cpp:2533-2576::chain_print_object_instances`
with machinery at `:64-175` and the `:418` union-find chain merge, KD batched
update notes at `:1007-1011`; `KDTreeIndirect.hpp:34-165` and
`MutablePriorityQueue.hpp:60-260`. (2.4.2 sub-ranges `92-418,1007-1011,
2015-2041`/`62-69,114-165,229-243`/`76-197` are the same constructs.) Scaled f64
distances, KD visit/pruning and heap mutation order on equal priorities are
observable. Millimetre conversion, index-based tie breaks, arbitrary reversal
or a two-object special case are not equivalent ports. In beta the chain feeds
`objPrintVec` (`Print.cpp:3812,3839-3848`) and emission; the claimed-area
subtraction iterates `brimAreaMap` key order instead — both must be ported
as-is and neither substituted for the other.

**One implementation, all callers:** replace
`gcode_emit/layers/object_order.rs::chain_instance_order/chain_without_start`
with delegation to the existing
`perimeters/classic/shortest_path/chain.rs::chain_segments_constrained_with_optional_start`,
which uses `chain_multiple_constrained`, `chain/search.rs`, `kd_tree.rs` and
`priority_queue.rs`. Remove the private instance-chain KD/heap/union-find
implementation from `object_order.rs`; do not retain a legacy seeded path or
new/old policy switch. Existing source-owned perimeters implementation is reused,
not rewritten. Visibility/re-export may be widened only to
`pub(in crate::project_slice)` for this call.

Adapter semantics: each instance center is a degenerate segment with identical
scaled endpoints; every segment is reversible. Pass `Some(start_near)` unchanged
for seeded callers and `None` for unseeded callers, and discard reversal flags
only after chaining. Preserve caller-owned emission reversal in `instances.rs`;
do not reverse in the adapter. Existing optional-start port handles0/1 segments.
Read-only source arithmetic verified the captured unseeded result `[1,0]` and
seeded `[15,220]` result `[0,1]`, matching Native before emission reversal.

Also correct accumulation granularity: all of one owner's instances must read
the same earlier-owner accumulator; in beta this is carried by the map-level
claimed-area loop (`Brim.cpp:953-958`) after the instance loop and cumulative
`brim_area` extension (`:631-633`, 2.4.2 `:578-581`). Previously approved
minimal batch orchestration call and mechanical empty-accumulator
direct-test argument remain in scope. Batch output remains keyed by original
object index.

Final GREEN is one production commit covering the delegation/deletion, ordered
owner subtraction and approved wiring; superseded attempt is retained only as
evidence. Public RED tests, six frozen inputs and helper/signature remain frozen.
No new tests, KD implementation, guard, fallback or result constants.

Expanded directed gates in BOTH modes: the six fixed owner cases; neighboring
brim/skirt/Auto/Ears/per-object cases; multi-object and multi-instance ordering
cases exercising `layers/instances.rs` callers. Then fmt, workspace all-target
clippy with `-D warnings`, wasm32. Any strict residual requires STOP before
further edits. No FULL until another coordinator-allocated window. Combined
components,
progress/time fixes and all other deferred behavior remain excluded.
