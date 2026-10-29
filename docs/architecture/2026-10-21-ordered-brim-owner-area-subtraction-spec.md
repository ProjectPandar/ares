# Ordered cross-owner brim area subtraction rewrite SPEC

Status: **SPEC REVIEW REQUEST — docs-only; RED awaits approval**.
Base and explicit dependency: integrated main
`0616138a348ea99d7bd964d527104b4902cf39d7`.
This is an independent repair of an already-supported uncombined brim path.
The per-object combined-brim contact-component gap remains suspended.

## Owning upstream boundary

Source checkout: `8500fcdccaa10b5099ac20d252af3a7c560046f1`, paths below
under `OrcaSlicer/src/libslic3r`.

| Source | Behavior being preserved or ported |
|---|---|
| `Brim.cpp:36-45::append_and_translate` | Translate candidate area by instance shift, then `diff_ex(srcShifted, dst)` before appending it to that owner's `brimAreaMap` entry. Earlier occupied brim area has priority. |
| `Brim.cpp:421-448::outer_inner_brim_area` | Visit print extruders and `objPrintVec` in upstream order, with written-state tracking; do not substitute arbitrary object-index, hash-map, tool-emission or nearest-nozzle order. |
| `Brim.cpp:573-581` | Assign instance area under the PrintObject owner, append assigned area to cumulative `brim_area`; later owners see that earlier area. |
| `Brim.cpp:662-683` | Subtract shared no-brim and per-extruder bed masks after ordered area assignment. Preserve this ordering; do not use already-infilled paths as the subtraction operand. |
| `Brim.cpp:685-713` | Retain brim fragments contacting object islands, same-owner fragments or other-owner brim areas. Preserve existing behavior; new contact-component combination is not this slice. |
| `Brim.cpp:836-875::makeBrimInfillImpl` | Generate paths only after source-owned area assignment/masking; preserve stepping, chaining and emission flow. |
| `Brim.cpp:964-974` | Uncombined mode generates each owner's infill separately from the assigned map, not from independent un-subtracted areas. |
| `GCode.cpp:4490-4537::generate_object_brim` | Consume stored owner collection once, in plate coordinates, before that owner's extrusion; keep ownership and ordinary emission behavior. |

The new rewrite slice is specifically the previously missing **ordered cumulative
area subtraction**, not another offset or brim-width formula and not a new Ares
pipeline. The remaining rows constrain where the port belongs in the existing
upstream sequence; they are not authorization to redesign adjacent behavior.

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
   `layers/object_order::chain_without_start` / `Print.cpp:2520-2530` mapping
   must be audited and reused where it is the matching upstream carrier.
3. Assign each candidate's difference against **earlier accumulated area** to
   its owner; extend the cumulative area as upstream does before later owners.
4. Keep subsequent masks/contact retention and per-owner path generation on
   assigned areas, in the cited order. Do not subtract paths, covered convex
   hulls, independently generated loops or already-clipped substitutes.
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

Deferred: `Print.cpp:2898-2990` per-skirt-group contact-connected combined brims,
new combine admission, repeated-copy/ByObject expansion, active support/tower,
multi-material/extruder expansion, AutoWidth/EFC/material/speed changes, new
mask/retention algorithms, loop chaining, travel/seam/fill/ironing changes.
No time-estimator or M73 correction is included: progress is a strict observable
of the repaired complete stream, not a value to tune independently.

## Fixed input provenance and Native observations

External input root:
`/home/indexyz/ares-coord/evidence/worker2-perobject-combined-discovery-20261021T103537/`.
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

Independent read-only diagnosis:
`/home/indexyz/ares-coord/evidence/worker2-near-uncombined-residual-20261021T104006/`.
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
   only in a future coordinator-allocated W2 window, exact tip after rebase.

All present work is docs-only, CPU13 single-core, no FULL/build/source changes.
Coordinator owns review/merge. Combined-brim SPEC remains suspended until this
supported uncombined control is repaired and independently reviewed.

## Approved revision: reuse one source-faithful instance chain

Coordinator authorized this revision after the attempted owner subtraction hit
strict residual; attempted `020062154` is **not** an accepted GREEN. Runtime
probe evidence is external at
`/home/indexyz/ares-coord/evidence/worker2-brim-runtime-order-probe-20261021T112517/`.
Native and Ares have the SAME ordered scaled reference points
`[(80250000,110000000),(88250000,110000000)]`, but unseeded outputs are
Native `[1,0]`, Ares `[0,1]`. Native `objPrintVec` confirms that owner priority.
The mismatch is in the duplicated instance-chain algorithm, not slot mapping.

Owning source: `ShortestPath.cpp:92-418,1007-1011,2015-2041`,
`KDTreeIndirect.hpp:62-69,114-165,229-243`, and
`MutablePriorityQueue.hpp:76-197`. Scaled f64 distances, KD visit/pruning and
heap mutation order on equal priorities are observable. Millimetre conversion,
index-based tie breaks, arbitrary reversal or a two-object special case are
not equivalent ports.

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
the same earlier-owner accumulator; extend it with the assigned owner area only
AFTER the instance loop (`Brim.cpp:578-581`). Previously approved minimal batch
orchestration call and mechanical empty-accumulator direct-test argument remain
in scope. Batch output remains keyed by original object index.

Final GREEN is one production commit covering the delegation/deletion, ordered
owner subtraction and approved wiring; superseded attempt is retained only as
evidence. Public RED tests, six frozen inputs and helper/signature remain frozen.
No new tests, KD implementation, guard, fallback or result constants.

Expanded directed gates in BOTH modes: the six fixed owner cases; neighboring
brim/skirt/Auto/Ears/per-object cases; multi-object and multi-instance ordering
cases exercising `layers/instances.rs` callers. Then fmt, workspace all-target
clippy with `-D warnings`, wasm32. Any strict residual requires STOP before
further edits. No FULL until another allocated W2 window. Combined components,
progress/time fixes and all other deferred behavior remain excluded.
