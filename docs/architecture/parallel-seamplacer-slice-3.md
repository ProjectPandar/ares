# Slice 3: parallelize the SeamPlacer stages

## Decision

Port the five upstream `tbb::parallel_for` boundaries in OrcaSlicer v2.5.0's
`SeamPlacer.cpp` into Ares seam placement. The pinned upstream source is
`OrcaSlicer` at `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`. This slice changes
only independent preparation and per-layer selection work. The cross-layer
alignment pass remains serial.

The implementation must preserve the existing Ares output bytes by
construction. Parallel execution changes scheduling, never the order in which
candidate records, choices, warnings, or final seam positions are committed.

## Measured target

A release KSR slice with `RAYON_NUM_THREADS=4`, a pinned 4-CPU set, and the
current instrumented boundary measured `emit_seam_placement` at **3.104 s**.
The four-worker ideal for the parallelizable portion is therefore
approximately a **2.3 s KSR wall-time saving** (`3.104 * 3/4`), before
accounting for serial mesh assembly, per-layer setup, synchronization, and the
serial alignment pass.
This is an expected target, not an acceptance promise: benchmark results must
report the actual saving and the residual seam-placement wall time.

The expected gain is concentrated in global visibility raycasts. Candidate
collection, candidate visibility, context values, and per-layer choice are
also parallelized, but their contribution is workload-dependent. The KSR baseline and raw run data are recorded with the release profiling
results for this slice. Temporary timers and profiling edits are not part of
this slice.

## Upstream source map

The upstream order is `compute_global_occlusion`, candidate gathering,
candidate visibility, overhang/layer embedding, per-layer seam selection, and
then serial `align_seam_points`. The relevant sites are:

| Upstream site | Ares destination | Parallel unit and owned output |
| --- | --- | --- |
| `OrcaSlicer/src/libslic3r/GCode/SeamPlacer.cpp:986-1050` prepares the occlusion model; `:184-225` is the `raycast_visibility` `tbb::parallel_for` | `crates/ares-core/src/project_slice/seam_placement/visibility.rs::GlobalVisibility::from_mesh`, with mesh/BVH construction in `mesh.rs` and `spatial/bvh.rs` | One visibility value per sampled mesh point. Mesh merge, decimation, transforms, sample generation, and tree construction remain ordered/serial as needed; raycast jobs write only `values[sample_index]`. |
| `SeamPlacer.cpp:1635-1665`, especially `:1641` | `seam_placement/alignment/preparation.rs::prepare` and `prepare_layer`, using `seam_candidates::generate_regions` | One layer. Each job owns a complete `LayerPlan`, candidate points, perimeter ranges, and that layer's point tree. Precise-seam warning mutations use per-layer warning slots and are merged in layer order. |
| `SeamPlacer.cpp:1668-1680`, especially `:1672` | Candidate score construction in `alignment/preparation.rs::prepare_layer`, calling `candidate_penalty` and `GlobalVisibility::at` | One layer. Each job fills only that layer's score vector; the visibility model and its spatial index are immutable. |
| `SeamPlacer.cpp:1683-1734`, especially `:1688` | `alignment/context.rs::populate`, including `populate_overhang`, `populate_embedded_distance`, and `alignment/context/unsupported.rs` | One layer. A job owns local distance trees and writes only `overhangs`, `embedded_distances`, and `unsupported_distances` for its layer. Previous-layer geometry is read-only input. |
| `SeamPlacer.cpp:2181-2194` | `alignment/preparation.rs::best_candidate` and `alignment/random.rs::apply`, reached from `seam_placement.rs::apply_objects` | One layer. Each job selects each perimeter's initial seam into that layer's preallocated `choices` slots. The within-perimeter scan remains in source order. Random mode uses a deterministic perimeter seed. |
| `SeamPlacer.cpp:1883-2140` | `alignment.rs::align`, including contour continuation and `string_fit` | **Serial.** It observes and finalizes choices across layers; do not parallelize it in this slice. |

The upstream `compute_global_occlusion` function itself contains serial model
assembly and cancellation boundaries. The cited line 184 parallelizes the
expensive raycast loop, not the `its_merge`/decimation sequence. The port must
retain that distinction.

## Execution design

### Phase boundaries

For each print object, preserve this phase order:

1. Build the immutable visibility model, parallelizing only indexed raycast
   values after the mesh samples and read-only acceleration structures exist.
2. Gather all layer candidates into an indexed `Vec<LayerPlan>`.
3. Calculate candidate visibility scores into each layer's score vector.
4. Calculate overhang, embedding, and unsupported distances into each layer's
   preallocated context vectors.
5. Select the initial seam for every perimeter in each layer.
6. Run `align_seam_points`/`alignment::align` serially when the seam mode
   requires alignment.
7. Continue with the existing placement and emission code.

A phase may begin only after all writes from its predecessor are complete. Do
not pipeline phases with partially initialized `LayerPlan` values: the phase
barriers are part of the deterministic contract.

### Indexed parallel results

Use Rayon indexed traversal or an equivalent fixed-slot dispatcher. Every
parallel job receives an explicit layer or sample index and writes to a slot
allocated before dispatch. The result vectors must have the same lengths and
initialization semantics as the serial path.

Do not use a shared accumulator, concurrent append, completion-order map, or
mutex-protected vector as the result representation. In particular:

- candidate vectors and perimeter ranges are constructed locally, then stored
  at `plans[layer_index]`;
- warning events are accumulated per layer and reduced in ascending layer
  order;
- visibility values are stored at the sample index;
- score, overhang, embedding, unsupported-distance, and choice fields are
  written only by their owning layer job;
- any list whose order is externally visible is concatenated by index after
  the parallel phase.

If the current API requires a mutable whole slice, split it into disjoint
sub-slices before invoking the worker rather than sharing mutable state.

### Global visibility and raycasts

Keep `TriangleMesh`, the sample positions/normals, precomputed hemisphere
directions, and the BVH immutable during raycasting. A worker must own its
scratch hit storage or use a worker-local scratch object; no worker may reuse
another worker's hit vector. The negative-volume path must follow the same
rule.

The loop must preserve the upstream sample-index algorithm, including the
front adjustment for `spAlignedBack`, the number and order of hemisphere
directions, the negative-volume sign handling, and the floating-point
operation order already represented by `visibility.rs`. Parallelism is only
across independent sample indices. Do not reduce ray hits, visibility values,
or sample statistics in completion order.

### Layer candidate preparation

`prepare_layer` remains the unit of ownership. It may call candidate generation,
paint classification, score calculation, position construction, KD-tree
construction, and perimeter association without observing another layer's
mutable state. The layer index must be passed explicitly rather than inferred
from worker order.

The precise-seam warning state currently represented by a shared warning
reference needs a deterministic replacement: collect warning facts in a
layer-indexed local result and merge them from layer zero upward after all
candidate jobs join. Duplicate suppression and modifier usage registration
must likewise occur in that ordered merge, not in worker completion order.

### Candidate visibility

`GlobalVisibility::at` is a read-only query. Parallelize the outer layer loop,
not a shared score append. Each layer computes scores in candidate index order
and stores them in its own vector. The serial comparator and all tie behavior
remain unchanged.

### Overhangs and embedding

Match upstream's range behavior at `SeamPlacer.cpp:1688`: a worker may create a
local distance structure for the previous layer and a local structure for the
current layer. The first layer has no previous-layer query. Current-layer
embedding is enabled from the same region-count predicate as the serial path.

All geometry inputs, layer slices, and planned-layer metadata are immutable
for this phase. Only the destination layer's context arrays are mutable. Do
not share a mutable distance-tree scratch buffer or update a cross-layer
summary while workers run.

### Initial seam selection and RNG

For aligned, rear, and aligned-back modes, parallelize the per-layer initial
choice that corresponds to upstream `pick_seam_point`. For random mode,
parallelize the equivalent `random::apply` layer jobs. Keep each perimeter's
candidate scan in ascending source index order and write its choice to its
preallocated perimeter slot.

Before implementation, verify `pick_random_seam_point` against
`SeamPlacer.cpp:1367-1435`, including all of the following:

- the seed is the first candidate position of the perimeter;
- the seed dot-product constants are `12.9898f`, `78.233f`, and `133.3333f`;
- the multiplier and fractional-part operation use the upstream literal and
  single-precision rounding (`43758.5453f` in the pinned source);
- viable-edge filtering, enforced/blocked edge zeroing, and the final walk
  remain in source order;
- no thread-local stream, time seed, worker index, or completion order enters
  the result.

The current Rust literal and arithmetic must be compared at the compiled
`f32` bit level, not judged by decimal spelling alone. Record this comparison
in the implementation review against the pinned source; do not add a new
unit or internal test for it. Confirm the result through the existing public
API random-seam parity witnesses on both Native lanes, and repeat those
witnesses with the required thread-invariance settings. A parallel random job
is valid only because each perimeter has a geometry-derived seed and no RNG
state is shared between jobs.

## Cancellation and platform behavior

Use the existing cooperative cancellation mechanism. Check cancellation:

- before constructing or dispatching each phase;
- between global-visibility raycast blocks;
- between layer blocks in candidate gathering, scoring, context, and choice;
- before the ordered warning/result merge; and
- before entering serial alignment and before publishing final seam data.

A worker must stop at a block boundary and return the existing cancellation
error. Do not publish partially merged warning state or partially finalized
choices. Cancellation must not turn a failed slice into a successful slice
with incomplete seam data.

On `wasm32-unknown-unknown`, and when the parallel feature is unavailable, use
the same functions and the same indexed phase order through a serial fallback.
Do not require a WASM thread pool, atomics, shared memory, or browser-specific
scheduling. The serial fallback is the reference execution for byte identity.

## Scope exclusions

This slice does not parallelize:

- `align_seam_points`/`alignment::align` or contour continuation;
- mesh merging, decimation, transforms, or any order-sensitive model setup;
- nearest-mode emit-time placement;
- G-code layer emission or cooling;
- Arachne materialization, infill, island ordering, or tool ordering; or
- any seeded fuzzy-skin or other stateful random traversal outside seam choice.

No output formatting, comparator policy, seam preference, geometry formula, or
floating-point normalization may change as part of the port.

## Delivery order

If a public random-seam witness is missing, add it in a separate test commit
before implementation. The required order is: test commit when needed, a
RED/witness note, the implementation commit, the exact-tip gate, and the final
review.

## Verification contract

Implementation acceptance requires all of the following:

1. Capture raw G-code bytes and SHA-256 values for the fixed KSR, Arachne, and
   toolchange fixtures before and after the change. Compare bytes without
   normalization.
2. Run the seam-placement fixtures with `RAYON_NUM_THREADS=1,2,4,7,16`.
   The outputs at levels 1, 2, 7, and 16 must be byte-identical to the serial
   reference and to one another. Level 4 is the performance target and must
   also remain byte-identical.
3. Run the seam-focused parity suite (`test(seam)`) on both Native lanes,
   including the existing public random-seam witnesses, and repeat those
   witnesses for the thread-invariance settings. Repeat aligned, aligned-back,
   back, random, nearest, painted, precise-seam, multi-object, and
   mirrored/negative-volume cases.
4. Run the broad `orca_parity` base-versus-tip comparison on both Native lanes,
   with zero P->F changes and nonparity counts of 1 and 8 as required by the
   parity gate.
5. Use the existing public cancellation E2E from slices 1/2: cancel through
   the public token, require `Cancelled`, and require no output. Do not add
   internal phase hooks, mocks, or per-phase cancellation tests.
6. Run release benchmarks with a pinned 4-CPU set and
   `RAYON_NUM_THREADS=4`, recording repeated KSR wall times, seam-placement
   time, total wall time, and output hashes. Report the actual result against
   the approximately 2.3 s KSR t4 target rather than claiming the ideal bound.
7. Run formatting, workspace tests, workspace Clippy, the
   `wasm32-unknown-unknown` checks for `ares-core` and `ares-wasm`, and the
   repository pre-push checks. No temporary profiling timer or native output
   is committed.
