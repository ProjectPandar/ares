# ARD-0080: KSR island-order investigation

Investigation began with the Z52 movement mismatch through the candidate
`libslic3r/GCode.cpp:4982-5068,5432-5435` island grouping/enumeration,
`Layer.cpp:38-66` slice ordering and
`ShortestPath.cpp:1096-1120` polygon/point chaining. Rust destinations are
the existing `project_slice::extrusion_islands`, `slice_ordering`, and
`geometry::chain_points` modules. These are existing upstream compatibility
shells, not a new Ares pipeline. Capture now identifies the owning repair
boundary as `PerimeterGenerator.cpp:238-270` entity traversal and
`ShortestPath.cpp:1026-1042` loop chaining. Rust destinations are
`project_slice::perimeters::classic::shortest_path::chain::chain_extrusion_loops`
and its `entity_collections::traverse` consumer. Include only removal of
the extra coordinate canonicalization before chaining; preserve current
endpoint, KD-tree, heap and reversal semantics. Defer generic chaining,
top-split changes, seam behavior and reference scheduler independence.

Non-negotiable acceptance remains complete ordered G-code, with the existing
generator/timestamp-only normalization. Movement-only diagnostics cannot
establish acceptance. Retain all complete outputs, including unstable native
runs and failed experiments. Do not choose an arbitrary native path order,
sort output, tolerate coordinates, or pin private structures in permanent
tests. Preserve the original KSR fixture and separate initialized reference
identity from untouched official evidence. Global printer/default/option
acceptance remains open.

## Reproduction and reference stability

Baseline is pushed commit `e676f3e23dff0d47053ed66088b8770ac351b055`.
Original strict KSR replay16280 exits100/18.852s; complete pair
`${TMP_ROOT}/ares-ard80-ksr-baseline/case-cim6Iv` first differs at M73 line9946.

External grid probes produce apparent REDs in under one second. A three-column,
two-layer, two-wall fixture appears minimal under unrestricted native
scheduling: removing any column, one layer, or one wall yields a passing
full output in those exploratory runs. Strict cached replay of the three
columns exits100, pair `${TMP_ROOT}/ares-ard80-minimal-strict-red/case-ZdT4q5`,
first difference line57 (`X165.169` versus `X135.169`). These are not accepted
regressions: subsequent native repeats contradict deterministic RED.

Five normal-affinity native runs at fixed seed0 include one different path
order, retained in `${TMP_ROOT}/ares-ard80-native-normal-repeat`. Five CPU0 runs are
identical and match complete Ares output, retained in
`${TMP_ROOT}/ares-ard80-native-single-repeat`. The nine-column-grid CPU0 repeats
also match complete Ares output. Thus no production sorting change can be
justified by these grid probes. Debugger captures of all point-chain inputs
and results, and final island contours, agree for that model. The failed
native islands capture31146 attempted to read the `id` method as a field;
exclude it despite GDB exit0. Correct capture11588 exits0 with normal
inferior exit; Rust captures also exit normally. All scripts live in /tmp.

Five unrestricted full KSR native runs vary in estimated time and movements,
including the same Z52 alternative selected by Ares. Five CPU0 full KSR runs
are identical, retained at `${TMP_ROOT}/ares-ard80-ksr-native-single-repeat`.
This establishes stability for this input/build, not general scheduler
independence. Strict replay28572 against the fresh single-CPU native output
still exits100, pair `${TMP_ROOT}/ares-ard80-ksr-single-strict-red/case-oqh90Y`,
with the same first M73 difference. Movement-only localization still finds
native travel `(150.165,101.366)` versus Ares `(135.165,86.366)` at Z52.
Native normal run1 shares Ares's Z52 choice but first differs from Ares at
Z52.2 instead; accepting any native ordering is not strict acceptance.

Native wrapper SHA256:
`3d05d4ccab740ca5e13aa94d0d00867a75f867ba2159b9f5b024dc0fcaa3bfaa`;
wrapped ELF SHA256:
`8f0cf69dd9494ccfe53bf6749d6b685ccb5a0dd0df206694ce50d421007cc03d`.
These are the separately approved seeded/initialized build, not untouched
official OrcaSlicer. CPU affinity is applied only to the external reference.
Confirmed random seeds alone do not make multithreaded reference output
deterministic.

## Proven loop-chaining boundary

KSR layer259 final outer contours and holes are identical before grouping;
both contain one large contour with no holes. The small inner loops are
produced by top-surface wall splitting, not disconnected input islands.
Normal-exit captures78896/79255 and67573/59590 retain complete outputs.

A closed two-layer base plate with three raised square posts and active
`only_one_wall_top=1` gives a stable CPU0 RED. Disabling cooling exposes
the movement difference directly instead of its derived minimum-layer
speed difference. Five independent native runs are identical at
`${TMP_ROOT}/ares-ard80-base-grid3-stability`; nine-post stability is retained at
`${TMP_ROOT}/ares-ard80-base-grid9-stability`. Strict cached RED exits100/0.6s,
pair `${TMP_ROOT}/ares-ard80-base-minimal-strict-red/case-yWPKqK`, first differs at
line75: native `G1 X135.149 Y116.383 Z.6`, Rust
`G1 X165.149 Y116.383 Z.6`. Removing the middle post loses this witness;
removing either end reveals later seam differences instead, so do not claim
an absolute smallest strict-divergence model.

Native86299 captures incoming loop first points in order
`(15390542,390542)`, `(390542,390542)`, `(-14609458,390542)` and returns
entity order `[1,2,0]`. Rust46749 captures the identical incoming points,
but its additional `(y,x,index)` sort canonicalizes the candidate order
before the index-sensitive KD-tree/heap traversal. Both captures exit
normally and retain complete output. Failed probes17465/93527/97289 are
excluded: a breakpoint matched an optimized lambda, or Rust pointer-field
inspection failed. No private ordering capture becomes a permanent pin.

A permanent live public project-byte regression exports the closed STL,
verifies effective settings and two independent native replays, and uses the
unchanged complete ordered comparator. RED60350 exits100/2.065s after
setup/stability assertions pass. Removing only the extra pre-chaining sort
produces public GREEN56763, exit0/2.184s, complete matches against both
independent native outputs. The comparator and tolerances are unchanged.
Delete the directly obsolete private symmetric-index snapshot; do not replace
it with another ordering pin. This removes an intermediate allocation and
sort without changing the actual chaining algorithm.

Preliminary independent review finds no required production fix and confirms
both normalized GREEN pairs contain identical 24,795-byte outputs. Its
explicit classic-generator activation suggestion is adopted. Also verify
ordinary XY placement via the public project loader and retain both paired
artifacts before asserting comparison outcomes. Final approval remains
pending; docs ship only with code after review.

## Complete-output revalidation

Original unchanged KSR input SHA256 remains
`698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9`.
After the sort removal, strict replay80223 exits0/46.307s against the retained
initialized reference; complete pair is
`${TMP_ROOT}/ares-ard80-ksr-after/case-V1WiUt`. Strict replay52146 against this
turn's stable CPU0 reference also exits0/28.569s, retained under
`${TMP_ROOT}/ares-ard80-ksr-single-green`. Both compare every ordered byte except
the existing generator-line normalization, including M73 and all movements.
Ares raw output is 6,321,724 bytes, SHA256
`3869281da4157938bd0cd0752a068087145cc7d293b3cf725c91fa9987c77016`.

The live initialized-object-ID self-replay retains two newly produced KSR
outputs at `${TMP_ROOT}/ares-ard80-consumers/initialized-object-id-EKDM9d`.
Its self-replay test does not establish Ares parity by itself: separately
feed both complete outputs into the unchanged strict comparator. Replay26137
exits0/25.355s, two PASS pairs retained in
`${TMP_ROOT}/ares-ard80-fresh-ksr-green`. Thus the earlier M73/Z52 mismatch is
resolved for this exact input and identified reference, not all printers or
untouched official multithreaded output.

Ten initialized consumer tests exit0/79.558s: nine live Ares full-output
tests and one reference-only KSR self-replay. Twenty-four cached stepped
fixtures pass, replay37284 exit0/0.501s. Initial workspace run26994 exits0,
7460 passed/9 skipped; full all-target Clippy46907 exits0 with existing
warnings. An extra -Dwarnings attempt fails on252 warnings and is not a
passing gate. The newly added model nesting warning is fixed by flattening
tile iteration without changing fixture geometry; focused Clippy39501
exits0 with no warning in the new files. Public final strengthened regression
79630 exits0/2.216s after the lint fix. Final exact-code workspace47455
exits0/260.231s, 7460/7460 passed with9 skipped. Fmt and diff checks pass.
All five real builds now pass: core WASM, Windows GNU,
macOS, Linux and WASM adapter. The first Windows build failed because the
host lacked dlltool; retry42692 exits0 after obtaining pinned Nix
`pkgsCross.mingwW64.buildPackages.binutils`, path
`/nix/store/17mzbf0li71ssrckz4y1s1s3a6k9rx3r-x86_64-w64-mingw32-binutils-wrapper-2.46`.
These are actual cargo build runs, not the previous slice's cargo checks.

Focused residual option scan32737 exits100/24.085s. `enable_support=false`
now strictly passes; true still diverges due to deferred
`PrintObjectSupportMaterial` / `TreeSupport` behavior, leaving that domain
INCOMPLETE. `printer_technology=FFF` passes; SLA is still converted to FFF
by the external CLI application boundary and remains FAIL, not a tested SLA
implementation. Full inputs/outputs and domain summary remain at
`${TMP_ROOT}/ares-ard80-option-residuals`. No global domain completion is claimed.

## Independent review outcome

The independent six-axis reviewer approves the immutable code/docs patch
SHA256 `1a5f1ed8963dd057c0470262c3e2625811e41d4bc5c038475fc76449e43942fc`,
tree `2749c82a464ed2e442403ca177e160af5f895d5d`, with no required fixes.
This approval covers this finite rewrite slice and exact-input KSR GREEN,
not global printer/default/all-option completion. The preceding pending
checkpoints are historical; confirm this documentation-only review-status
addition before submitting code/docs together.
