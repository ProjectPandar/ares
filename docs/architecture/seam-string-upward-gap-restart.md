# Seam-string reversal after an upward gap

Status: bounded RED qualification only; no production change. Target upstream is
OrcaSlicer beta `345efec64c7b95a05ce5cf96df0dde66a158835c`.

## Upstream and Rust boundary

`GCode/SeamPlacer.cpp:1824-1874::find_seam_string` reverses after an upward miss
at `:1865-1869` and immediately continues. Thus start_layer-1 is visited next.
`project_slice/seam_placement/alignment.rs::find_seam_string` resets to that layer
but reaches the unconditional next_layer += step, skipping it. Object-top reversal
already visits the immediate lower layer and must remain unchanged.

Include only the miss/reverse/continue control flow. Candidate comparison, KD
visitation, radius, alternative starts, spline fitting, continuous loop-normal
placement, contour resolver, paint, scoring and geometry are adjacent/deferred.
Existing Rust plans are compatibility data around upstream layer/candidate/string
concepts, not a new seam algorithm or authority to tune fitting thresholds.

## Preserved failed qualification and new input

The old af0177a5a two-input STOP remains unchanged. Fresh input1 on the combined
5b80490e2 base still had Native rear-interior placement but API opposite-corner
placement. It is explicitly NOT a restart RED. It is not relabelled as fixed by
C2 or by this new comparison.

The second/final input has one Classic boundary in Back mode, 23 lower layers and
a far upper continuation. Its octagon has a unique soft rear maximum. Lower-layer
translation is small; the penultimate notch translates parallel to the same rear
edge. This controls the prior equal-rear-Y competition and reduces overhang-priority
confounding while keeping fitted targets on the edge rather than invoking a new
normal-projection policy. It is a single fixed geometry, not a parameter sweep.
Input SHA-256: `be46fa7a81ee17aef5019dc3e341b1264e7261c551a60cbce91849842521c02e`.

Native exports ran before the cutover steer on 5b80490e2; their source identity is
preserved. Evaluation of those saved outputs establishes stable double replay
and an interior rear-edge start at (107.050,115.430), followed by the unique peak
at (107.029,115.431), with nine extrusion segments at Z4.4. No Native output is
committed. A new public API comparison is authorized on restacked b389b5301; no
claim turns the earlier Native run into an execution at this new commit.

## Decision boundary

The public validated API must succeed on identical bytes. If it chooses the
opposite corner, geometry or candidate choice is still confounded and cannot
establish restart. If it retains the same unique rear peak but lacks the Native
interior lead-in, inspect that missing fitting observation against the explicit
skip control flow. Preserve the complete original object block comparison,
including feed and every motion; no computed fitted expectation or private
string-count probe substitutes for runtime Native.

If the single authorized comparison does not discriminate on the owned decision,
park this slice with both input dispositions and request the next delta/new budget.
No further input, production intervention, forced corner, tolerance, contour
rotation/sorting, synthetic flag or projection correction. A qualified public RED
and review are required before any source-only GREEN. No broader/FULL or seeded run.
