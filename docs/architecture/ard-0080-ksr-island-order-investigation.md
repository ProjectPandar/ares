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

Baseline is commit `e676f3e23dff0d47053ed66088b8770ac351b055`.
Original strict KSR replay exits 100; the complete pair first differs at M73
line 9946.

External grid probes produce apparent REDs in under one second. A three-column,
two-layer, two-wall fixture appears minimal under unrestricted native
scheduling: removing any column, one layer, or one wall yields a passing
full output in those exploratory runs. Strict cached replay of the three
columns exits 100, first difference line 57 (`X165.169` versus `X135.169`).
These are not accepted regressions: subsequent native repeats contradict
deterministic RED.

Five unrestricted native runs at fixed seed 0 include one different path
order. Five single-core-affinity runs are identical and match complete Ares
output. The nine-column-grid single-core repeats also match complete Ares
output. Thus no production sorting change can be justified by these grid
probes. Debugger captures of all point-chain inputs and results, and final
island contours, agree for that model. One native islands capture attempted
to read the `id` method as a field and is excluded despite GDB exit 0; the
corrected capture exits with a normal inferior exit; Rust captures also exit
normally.

Five unrestricted full KSR native runs vary in estimated time and movements,
including the same Z52 alternative selected by Ares. Five single-core full KSR
runs are identical. This establishes stability for this input/build, not
general scheduler independence. Strict replay against the fresh single-core
native output still exits 100 with the same first M73 difference.
Movement-only localization still finds native travel `(150.165,101.366)`
versus Ares `(135.165,86.366)` at Z52. Native normal run 1 shares Ares's Z52
choice but first differs from Ares at Z52.2 instead; accepting any native
ordering is not strict acceptance.

Native wrapper SHA256:
`3d05d4ccab740ca5e13aa94d0d00867a75f867ba2159b9f5b024dc0fcaa3bfaa`;
wrapped ELF SHA256:
`8f0cf69dd9494ccfe53bf6749d6b685ccb5a0dd0df206694ce50d421007cc03d`.
These are the seeded/initialized build, not untouched official OrcaSlicer.
Single-core affinity is applied only to the external reference. Confirmed
random seeds alone do not make multithreaded reference output deterministic.

## Proven loop-chaining boundary

KSR layer 259 final outer contours and holes are identical before grouping;
both contain one large contour with no holes. The small inner loops are
produced by top-surface wall splitting, not disconnected input islands.
Normal-exit native and Rust captures retain complete outputs.

A closed two-layer base plate with three raised square posts and active
`only_one_wall_top=1` gives a stable single-core RED. Disabling cooling
exposes the movement difference directly instead of its derived minimum-layer
speed difference. Five independent native runs are identical, and nine-post
stability is also retained. Strict cached RED exits 100 and first differs at
line 75: native `G1 X135.149 Y116.383 Z.6`, Rust
`G1 X165.149 Y116.383 Z.6`. Removing the middle post loses this witness;
removing either end reveals later seam differences instead, so do not claim
an absolute smallest strict-divergence model.

The native capture records incoming loop first points in order
`(15390542,390542)`, `(390542,390542)`, `(-14609458,390542)` and returns
entity order `[1,2,0]`. The Rust capture records the identical incoming points,
but its additional `(y,x,index)` sort canonicalizes the candidate order
before the index-sensitive KD-tree/heap traversal. Both captures exit
normally and retain complete output. Failed probes are excluded: a breakpoint
matched an optimized lambda, or Rust pointer-field inspection failed. No
private ordering capture becomes a permanent pin.

A permanent live public project-byte regression exports the closed STL,
verifies effective settings and two independent native replays, and uses the
unchanged complete ordered comparator. RED exits 100 after setup/stability
assertions pass. Removing only the extra pre-chaining sort produces public
GREEN, exit 0, complete matches against both independent native outputs. The
comparator and tolerances are unchanged. Delete the directly obsolete private
symmetric-index snapshot; do not replace it with another ordering pin. This
removes an intermediate allocation and sort without changing the actual
chaining algorithm.

Both normalized GREEN pairs contain identical 24,795-byte outputs. The test
explicitly activates the classic generator, verifies ordinary XY placement via
the public project loader, and retains both paired artifacts before asserting
comparison outcomes.

## Complete-output revalidation

Original unchanged KSR input SHA256 remains
`698f40f13c9075b818abedd3d10f022fbb5d8200aed48fbdde651f6bfb21b8a9`.
After the sort removal, strict replay exits 0 against the retained
initialized reference and against a fresh stable single-core reference. Both
compare every ordered byte except the existing generator-line normalization,
including M73 and all movements. Ares raw output is 6,321,724 bytes, SHA256
`3869281da4157938bd0cd0752a068087145cc7d293b3cf725c91fa9987c77016`.

The live initialized-object-ID self-replay produces two new KSR outputs. Its
self-replay test does not establish Ares parity by itself: separately feed
both complete outputs into the unchanged strict comparator, which passes both
pairs. Thus the earlier M73/Z52 mismatch is resolved for this exact input and
identified reference, not all printers or untouched official multithreaded
output.

Ten initialized consumer tests pass: nine live Ares full-output tests and one
reference-only KSR self-replay. Twenty-four cached stepped fixtures pass.
Workspace tests pass, 7460 passed with 9 skipped; all-target Clippy exits 0
with existing warnings. The newly added model nesting warning is fixed by
flattening tile iteration without changing fixture geometry. Fmt and diff
checks pass. Core WASM, Windows GNU, macOS, Linux and WASM adapter builds
pass. The Windows GNU build requires pinned Nix
`pkgsCross.mingwW64.buildPackages.binutils` for dlltool. These are actual
cargo build runs, not cargo checks.

A focused residual option scan shows `enable_support=false` now strictly
passes; true still diverges due to deferred
`PrintObjectSupportMaterial` / `TreeSupport` behavior, leaving that domain
INCOMPLETE. `printer_technology=FFF` passes; SLA is still converted to FFF
by the external CLI application boundary and remains FAIL, not a tested SLA
implementation. No global domain completion is claimed.

## Status

Accepted for this finite rewrite slice and exact-input KSR GREEN, not global
printer/default/all-option completion.
