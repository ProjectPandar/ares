# Seeded slicer Context implementation plan

Base: c9868321. Decisions: ARD-0068.

1. Trace the public project slicing interface to fuzzy polygon materialization
   and identify the smallest explicit Context propagation seam.
2. Confirm the original mt19937/distribution sequence and stable parallel
   consumption rule; review the interface before implementation.
3. Write public output regressions, observe RED, then integrate the Context
   and upstream random rewrite into active fuzzy geometry.
4. Build the isolated, visibly labelled reproducible original. Validate any
   initialization patch separately; retain untouched official controls.
5. Seed repeatability and first-reference full-output parity, selected fuzzy
   cases, full checks, independent six-axis review/repair; commit/push together.

Status: Context interface and active classic consumer connected, callers
migrated explicitly. Public self-replay and independent-job concurrency test
passed on a real official-exported project, with input and six complete outputs
retained in `/tmp/ares-seeded-context-direct.7x6WrP`. Fuzzy regressions passed
78/78. A workspace run passed 7447 tests with nine skips, but precedes the final
continuous-stream refinement and therefore is not the final gate. WASM,
Windows and macOS core checks passed. Rerun final gates after parity fixes.

The separate seed-only original has built and launches. Four seeds times
1000 double bit patterns match the actual Rust engine, including multiple
state updates. A single-CPU fixture self-replays except for generator time;
multi-CPU runs do not. The path-reseed prototype has been removed in favor of
one continuously consumed per-task RNG, without modifying original stream
partitioning. Thread-count-independent original replay remains future work,
not a claim of this single-CPU diagnostic contract.

The first actual full comparison failed and is retained at
`/tmp/ares-seeded-continuous-parity.HG2UgV/case-IPyzMi`: 267581 original bytes
vs 269868 Ares bytes, including Arachne fill survivors and fuzzy differences.
An unfilled-wall fixture is being used to isolate fuzzy consumption; this
does not close the original filled-case failure. The unfilled classic fixture
has now passed full generator-only comparisons for seeds 0 and 1, with seed-1
paired bytes retained in `/tmp/ares-seeded-continuous-seed1.LwMDdp`.
The maximum u32 seed also passed, with paired bytes in
`/tmp/ares-seeded-continuous-max.BiA3fF`; the final seed-0 run is retained at
`/tmp/ares-seeded-continuous-zero-final.vBRFw2`. Clippy and rustfmt pass;
the final workspace run passes 7447 tests (nine skipped), 257.272s. The
0.8 mm cube / 2 mm point-distance short-contour fixture passes the full
reference comparison. Reinstating the incorrect descending padding index
makes this public test fail (26901 reference vs 27080 Ares bytes), retained
under `/tmp/ares-seeded-short-counterfactual.*`; the source behavior is restored
and both restored fixtures pass seed 0/1/UINT_MAX. Each seeded run uses a fresh dedicated artifact
root; runner caches are not valid across reference builds or seeds.

Independent review requested coherent-spacing evidence and patch-whitespace
repair. Both are addressed: Perlin/Billow/RidgedMulti/Voronoi pass complete
seed-0 reference comparisons and repeat/interleave/concurrent-job checks,
`/tmp/ares-seeded-coherent.log` (three tests, six fixture comparisons including
both classic models). The two classic fixtures also pass seed-1 and UINT_MAX
comparisons (`/tmp/ares-seeded-short-seed1.log`, `...-max.log`). Assertions lock
the exported classic wall generator, noise kind and point distance.
The retained source patch uses zero context; forward checking on untouched
8500fcdcc and reverse checking on the built source both pass. The build recipe
is unchanged and still represents the same source substitution. Final
workspace passes 7449 tests (nine skipped), 253.194s;
`/tmp/ares-seeded-context-workspace-approved.log`. Final all-target Clippy,
rustfmt and staged whitespace checks pass. Independent six-axis re-review
approves this finite Context/classic fuzzy slice after the two requested
repairs. Filled-case differences and general parallel reference replay remain
uncompleted; no global parity completion is claimed.

Independent pre-implementation review requires the lifecycle, stable parallel
consumption and exact standard-library distribution contracts before code.
Classic materialization is the narrow state-passing seam; unrelated stages
need not hold Context. A fixed per-worker seed alone is explicitly rejected.
