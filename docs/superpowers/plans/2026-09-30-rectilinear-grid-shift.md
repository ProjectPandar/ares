# Rectilinear grid-shift implementation plan

Source boundary and non-negotiable arithmetic: ARD-0038 and the matching spec.
Existing Ares fill code is an upstream compatibility shell, not a new pipeline.

1. Add a small wide-slab complete-output oracle regression at the agreed
   public slicing seam. Run against the unmodified official 2.4.2 reference
   and retain a real red result, not the offline runner's no-op pass.
2. Preserve f32 horizontal movement through rectilinear, locked-zag, raft,
   ironing and sparse-anchoring callers; port both upstream shift branches.
   Verify the tracer turns green before expanding signed-shift coverage.
3. Re-run fill callers, complete-output bridge/timelapse/KSR diagnostics,
   workspace nextest, fmt/clippy and WASM/Windows/macOS compilation.
4. Request independent six-axis review, resolve findings, then commit/push
   verified code and documentation together. Do not claim global completion.

Diagnosis evidence and the corrected arithmetic audit are recorded in the
bridge-direction plan's KSR section. The initial test invocation omitted the
artifact-root environment and returned an offline pass; it is not red/green
evidence. The live invocation uses a fresh explicit artifact root.

Self-contained cause: KSR's Rust grid x0=-48329210 differs from official
x0=-48329208 with identical spacing 457622. Upstream's float compound
addition converts the former to f32 even when horiz_move=0; official
disassembly confirms cvtsi2ss/addss/cvttss2si. Consequently line 19 is
-39634392 in Rust versus -39634390 officially. Changing only that grid
origin in the Rust process changes emitted X156.234 to official X156.233.
The minimal wide-slab RED/GREEN below independently locks down actual output.

## Execution

The wide 73.07148 × 67.07148 × 0.4 mm slab is a real red tracer:
`/tmp/ares-fill-grid-red-live.log` exits 100, official E1.59475 versus
Rust E1.59474 at line 262. Artifacts are under
`/tmp/ares-fill-grid-red.QaAVrW/case-yHrv5v`.
After preserving f32 through the caller chain and porting the unconditional
grid shift, `/tmp/ares-fill-grid-green.log` exits 0 with complete-output
parity in 0.760 s. No comparator or reference change was needed.

Clippy passes with the existing 227 warnings. At this intermediate stage,
workspace nextest and three cross-target compilations were still running.
Independent static review accepts the grid arithmetic, but identifies a
related pre-existing early-f32 narrowing in
`group_fills/params/projection/sticky.rs`. Multi-layer cross-zag cases now
exercise fractional step values and alternating signed shifts before deciding
the next repair. The original all-printer/all-option goal remains open.

The first fractional-step request .73127153 was normalized by official
export to .731272; application verification rejected the request rather than
silently testing a different value. This is not a slicing regression.
Using exactly retained .731272 and 2.30772 steps, the two test functions
(three complete-output cases) pass in 1.808 s:
`/tmp/ares-fill-grid-signed-v2.log`. The 2.4 mm model retains actual sparse
infill, with one bottom/top shell layer and zero shell-thickness overrides.
This covers signed shifts but does not prove precision of the separately
identified sticky phase-construction narrowing.

WASM, Windows and macOS checks all exit 0 (logs
`/tmp/ares-fill-grid-{wasm,windows,macos}.log`). Existing five bridge cases
pass. Timelapse retains the same three overhang failures and thirteen passing
cases (`/tmp/ares-fill-grid-existing.log`), not a new tolerance or waiver.
The default workspace invocation ends after the existing KSR layer-26 bridge
golden mismatch: 5623 pass, one fail, nine skip, 1764 unrun due to fail-fast.
A full `cargo nextest run --workspace --no-fail-fast` was then started; the
partial run is not a full-workspace pass.

Fresh production KSR output `/tmp/ares-ksr-grid-fixed.gcode` removes the
diagnosed first-layer grid-origin discrepancy without debugger intervention.
Complete parity remains red: config bookkeeping, object labels, M73/time
and further motion differences still require source-boundary work. The
original reference remains unchanged.

Locked-zag caller coverage also passes: final live run
`/tmp/ares-fill-grid-locked-v2.log` has three tests / four complete-output
cases green in 3.652 s. The sparse-layer assertion tracks persistent TYPE
state and actual XY extrusion moves, requiring at least four sparse layers;
the initial comment-per-layer counter was incorrect because upstream omits
redundant TYPE comments. Its failure was in the new assertion, not slicing.
All changed source/test files remain below 400 lines; fmt/diff checks pass.

Final non-fail-fast workspace run completes in 257.325 s:
7388 run, 7387 pass, one existing KSR layer-26 reference failure, nine skip
(`/tmp/ares-fill-grid-workspace-full.log`, exit 100). This is not a green
workspace claim; no additional failure is observed against the prior baseline.
The subsequently added locked-zag test is covered by the separate live run.
Final reviewed live run `/tmp/ares-fill-grid-reviewed.log` exits 0:
three tests/four cases pass in 1.397 s. The multi-layer assertion now parses
positive E and requires tracked relative-extrusion mode, excluding negative-E
wipe moves. Independent review confirmed real locked-zag skeleton and skin
paths, not merely active configuration.

Independent review required an isolated HEAD-plus-grid-patch run before a
standalone commit, because the main working tree also contains unfinished
bridge, affine and timelapse slices. The isolated baseline must judge KSR
afresh rather than inherit the mixed-tree failure classification.
Isolation ran at `/tmp/ares-grid-isolated.m1r5o0` from HEAD accb8f78
plus only this slice's nine core files, public test/registration and docs.
The upstream tree is a read-only input symlink. Live-oracle, full workspace
and clippy/fmt logs use `/tmp/ares-grid-isolated-{live,workspace,clippy,fmt}.log`.
The standalone commit was withheld until these checks completed.

Isolated live output passes all four cases in 1.420 s, independently of the
other pending slices (`/tmp/ares-grid-isolated-live.log`, exit 0). Isolated
clippy finishes successfully with 249 warnings; fmt check also exits 0.
Importantly, the isolated KSR semantic test passes in 47.514 s: the mixed
tree's KSR reference failure is not inherited as a waiver for this slice.
At that intermediate observation, the rest of the isolated run was ongoing.

Final isolated full-workspace result: 7372 tests run, all 7372 pass, nine
skip, 255.507 s (`/tmp/ares-grid-isolated-workspace.log`, exit 0). The
isolated baseline includes neither pending bridge/affine/timelapse work nor
user geometry changes. All four live cases, KSR semantic regression,
workspace tests, clippy and fmt have therefore passed for HEAD plus this
slice alone. The stage scope is exactly the nine core files, public test and
registration, ARD/spec/plan and the roadmap's grid-only paragraph.
This closes the isolation gate, not the global printer/option parity goal.
Final independent six-axis review approves this limited grid-consumption
slice for an independent commit with no remaining code-fix requests. Sticky
phase generation and all other pending slices remain outside that approval.
