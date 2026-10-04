# Plan: Anycubic classic extra-overhang perimeters

[Spec](../specs/2026-10-04-extra-overhang-perimeters.md),
[ARD](../../architecture/ard-extra-overhang-perimeters.md).
Branch `pi/extra-overhang-perimeters`, initially stacked on approvedbd49f5fd.
After explicit main thaw, Qidi rebased/validated/integrated ase865eb2a; this branch
was rebased onto it before implementation. Independent /tmp worktree/target retained.

1. Preserve two stock REDs and create active step-overhang on/off full-output
   original-Orca controls before edits. Frozen stock source paths in
   `/tmp/ares-extra-overhang-replay/inventory.json`. Step geometry is an extruded
   eight-vertex stair cross-section; generator `/tmp/ares-extra-overhang-step.py`
   changes only model mesh and the feature flag in copied3MF. Original reference
   pairs under `/tmp/ares-extra-overhang-step-replay`.
2. Port `PerimeterGenerator.cpp:677-1084` generation, reconnection and sorting
   using existing geometry helpers. Locally adapt BridgeDetector.hpp:122-126 to
   existing geometry::detect_bridging_direction (authorized; shared algorithm
   edits forbidden without another coordinator check).
3. Port `apply_extra_perimeters` at classic infill-boundary surface stage:
   preserve activation operands; prepend paths; subtract covered fill; retain
   no-overlap semantics. Only then replace the existing unsupported guard.
4. Run strict target and active fixture comparisons, then68+823controls and four
   gates. Investigate remaining failures without weakening acceptance; keep all
   previous REDs and diagnostics. No unit tests written after implementation.
5. Push branch and REVIEW-REQUEST, then wait for approval before
   rebase/revalidation/ff-only integration, respecting any renewed main freeze.
   Qidi predecessor is already integrated.

Pre-implementation RED: stock2 both ARES_ERROR extra_perimeters_on_overhangs,
`/tmp/ares-extra-overhang-red/replay-summary.json`. Initial step artifact-generation
attempt failed because writing ZipInfo mutated input header offsets; fixed the
external generator with copy.copy before producing valid reference pairs. The
initial `/tmp/ares-extra-overhang-step-red` is NOT parity evidence. A second XML
serialization attempt lost extension namespace prefixes and is also invalid
(`/tmp/ares-extra-overhang-step-red-actual`). Surgical mesh-text replacement now
preserves those declarations. Valid evidence is `/tmp/ares-extra-overhang-step-valid-red`:
off full-bytePASS, on ARES_ERROR at the existing guard. Original Orca output
adds actual extra-wall paths (3563→3625G-code lines, Overhang wall role2→3).

## Implementation and first comparison

Ports are split across `classic/extra_perimeters/{mod,ordering,apply}.rs`, each
below400LOC. Existing stages remain the compatibility shell. Native `paths_touch`,
reconnection, anchored dependency reversal, path order and covered-area subtraction
are retained, including the source bool-assignment closest-point expression.
Reserved contiguous identity-hash keys permit deterministic reverse-insertion
iteration matching the source GCC unordered_map without a generic map emulator.
`has_regions` retains the distinction between an empty outer vector and nonempty
regions whose path lists are empty. The latter still triggers fill subtraction.
The existing `Geometry/ConvexHull.cpp:11-43` port moved unchanged from private
G-code skirt geometry into `geometry/convex_hull.rs`; all existing consumers now
use that owner, so perimeter generation reuses it without a G-code dependency.
Existing hull assertions remain unchanged.

First candidate restored stock2/2, but active step remained divergent; complete
failed artifact `/tmp/ares-extra-overhang-step-green/replay-summary.json` retained.
External debugger captures `/tmp/ares-extra-{native,rust}.jsonl` identified the
anchor-boundary difference: native(-1928119,3530625), Rust(-2140251,3318493),
exactly the half-nozzle displacement. `PerimeterGenerator.cpp:1177` caches expanded
`m_lower_slices_polygons`; its accessor (hpp:151), not raw lower slices, is passed
at cpp:1092. Reusing the existing classic-prelude cache corrects that input.
`Polyline.cpp:383-405` append drops a shared endpoint; raw reversed insertion and
final reconnect insertion intentionally do not. Corrected those distinct operations.

Corrected comparisons: stock2/2 and step2/2 full ordered-byte PASS at
`/tmp/ares-extra-overhang-{stock,step}-supported/replay-summary.json`.
No generator-external normalization or comparator changes. Shared bridge direction
untouched. No existing assertion removed; the existing stage unit-test call only
receives the new `None` context argument. First gate attempt failed compilation
because the new error wrapper assumed Display on existing Debug-only geometry
errors. All exit101 receipts remain `/tmp/ares-extra-overhang-exits.txt` and logs.
The wrapper now preserves typed Clipper/medial errors with Debug formatting.
The `verified` attempt was interrupted after Clippy exposed seven new nesting
warnings (228 total); no incomplete attempt is accepted. Flattened source loops
without suppressions or speculative behavior changes. Original logs and
intermediate Clippy diagnostics remain retained.

The `accepted` candidate then passed all gates (7594/7594 workspace,221 existing
Clippy warnings, fmt and WASM) and all replays (stock2, step2, full68, controls823,
official8, fresh4). Full receipts remain `/tmp/ares-extra-overhang-accepted-*`.
Final source audit found the gap conversion must distinguish source
`offset_ex` (ClipperUtils.cpp:409-410, PolyTree output) from raw `to_expolygons`
(ExPolygon.hpp:366-382, no union/hole regrouping). Both now use their exact source
operations. The previous passing receipts are not reused as final-code validation;
final-code gate/replay artifacts use `/tmp/ares-extra-overhang-source-final-*`.
Fresh CPU0 original-reference input/output checksums:
`/tmp/ares-extra-overhang-official-sha256.txt`.

## Final-code validation and handoff

`/tmp/ares-extra-overhang-source-final-exits.txt`: every exit0. Workspace
7594/7594 PASS (9 existing skips); fmt PASS; default Clippy PASS with unchanged
221 pre-existing warnings; actual wasm32-unknown-unknown ares-core+ares-wasm build PASS.
All replay inventories fully compared with `ordered_bytes_generator_only`:
stock2/2, active step on/off2/2, full68/68, controls823/823, original official8/8,
fresh official4/4. No skipped/dropped cases. Corresponding replay-summary.json
files are under `/tmp/ares-extra-overhang-source-final-{stock2,step2,full68,
controls823,official8,fresh4}`; command logs share that prefix.

Coordinator reports main advanced to16ac907b (support-interface fan lifecycle,
cooling/role_fans and motion/fan). This candidate is validated on prerequisite
e865eb2a. After coordinator approval, rebase onto latest origin/main and revalidate
before fast-forward-only integration; these results do not waive that requirement.
