# Stable KSR island-order plan

Source and destination are defined by the companion spec and ARD-0080.
Stage inputs establish the owning upstream boundary before selecting the
production repair. Do not build an independent Ares pipeline.

1. Re-run complete strict KSR and bind native source/build/seed identity.
   Verify: retained complete paired bytes and terminal result.
2. Minimize, checking reference self-replay before declaring stable RED.
   Verify: same input/reference repeatedly differs from public Ares output;
   discard models whose failure disappears under the selected affinity.
3. Capture candidate inputs and resulting order without production edits.
   Verify: normal debugger exits and complete outputs; revise cited source
   boundary if an earlier producer is responsible.
4. Add one public project-bytes to complete-G-code failing test, then port
   only the demonstrated upstream behavior. Delete affected obsolete pins,
   if any; do not replace them with new private-coordinate assertions.
5. Verify public GREEN, original KSR residuals, all affected consumers,
   workspace Nextest, fmt/clippy and Tier-1 builds.
6. Submit an immutable code/docs patch to the independent six-axis reviewer;
   repair/re-review findings before committing and pushing together.

## Evidence and current checkpoint

Pushed baseline `e676f3e2` is clean in the isolated worktree. Strict KSR
16280 exits100/18.852s at M73 line9946, with complete artifacts in
`/tmp/ares-ard80-ksr-baseline`. Grid exploration yields cached strict RED in
0.3s, but five native self-replays expose a normal-affinity path-order variant.
Five CPU0 grid replays match both each other and complete Ares output;
these grids are not accepted stable RED regressions.

All five CPU0 full KSR outputs are identical, whereas unrestricted full KSR
varies in time and movement order. Fresh CPU0 KSR cached strict replay28572
exits100, retaining complete pair
`/tmp/ares-ard80-ksr-single-strict-red/case-oqh90Y`. Its diagnostic first
movement mismatch remains Z52. Native/Rust layer259 final contours and holes
agree. Captures of the three-post base-plate top-split loop group have
identical incoming points; upstream preserves this order while Rust
additionally canonicalizes it. Repair boundary is now
`PerimeterGenerator.cpp:238-270` and `ShortestPath.cpp:1026-1042`, with Rust
destination `perimeters::classic::shortest_path::chain::chain_extrusion_loops`
and `entity_collections::traverse`. Stable CPU0 base-plate strict RED retains
pair `/tmp/ares-ard80-base-minimal-strict-red/case-yWPKqK`. Permanent public
RED60350 exits100/2.065s with first travel difference at line75. Removing
only coordinate canonicalization produces live GREEN56763, exit0/2.184s;
both fresh independent reference outputs match complete Ares output.
The obsolete symmetric private index pin is deleted rather than updated.
Independent preliminary six-axis review finds no required production repair.
Its suggested explicit classic activation assertion is implemented, alongside
ordinary input placement checks and retaining both paired artifacts before
asserting outcomes. Final rerun, other gates and immutable review are pending.
Original KSR strict replay80223 now exits0/46.307s; the stable CPU0 reference
replay52146 also exits0/28.569s. Both fresh KSR outputs from the live native
self-replay separately compare PASS in replay26137 (0/25.355s).
Ten initialized tests pass (nine Ares comparisons plus native KSR self-replay),
24 stepped cache fixtures pass, initial workspace7460/7460 passes with9 skips.
All-target Clippy passes with existing warnings; its extra strict-warning
attempt is not green. Fix the one new model nesting warning, re-run focused
Clippy and public full-output test: both pass. Final exact-code workspace47455
exits0/260.231s, 7460 passed/9 skipped. All five actual cargo builds pass;
Windows first fails for missing dlltool, then passes with pinned Nix cross
binutils. Fmt and diff checks pass. Submit the immutable code/docs patch
for final independent approval before committing. No commit exists yet.

Final independent six-axis review approves patch SHA256
`1a5f1ed8963dd057c0470262c3e2625811e41d4bc5c038475fc76449e43942fc`
with no required fixes. Confirm the final review-status documentation addition,
then commit/push the approved code/docs together. The global goal stays open.
