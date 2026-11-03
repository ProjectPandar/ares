# Dumbbell nearest seam: Gaussian distance normalization

Status: worker6 read-only source-cited spec for review. No production change,
new RED, fresh Native run or GREEN is claimed. Base: main
`b364a111de16597df7a06790721c4ea40159f86f`.

## Exact witness and evidence limits

Worker5 fixture, read-only:
`${TMP_ROOT}/ares-worker5/tests/parity/validation/tiq2-dumbbell-long-loop.3mf`.
SHA256 `90288cb1f8904fc245a792be9cc8d30e5ea2762f6faff4b573d362fec231bc23`.
Its bytes equal retained `input.3mf` in:
`${TMP_ROOT}/ares-tiq2-loop-red/corrected-transport/03-long_loop_fragments/artifacts/case-dqJg9L/`.

That directory's `orca.gcode` SHA256 is
`28d76e6b1a9c4bbe79c84782f9fc820fb63a3159fb823e9cdec6ece29af4b3df`;
`ares.gcode` SHA256 is
`8efeef41011b0c792970f7d8d9564efc96743c30ce6a706dc87c8cb37ac78d9d`.
Both raw line121s are travel to a loop seam at Y110.4:

```text
Native: G1 X107.957 Y110.4 Z.8
Ares:   G1 X112.043 Y110.4 Z.8
```

The X coordinates sum to220, symmetric about bed X110. Native starts at the
left neck endpoint; Ares starts at the right. The surrounding extrusion
sequence is cyclically reordered accordingly. This is a geometry/order
symptom, independent of where an M73 marker is inserted.

The coordinator's handoff calls this layer3. In these exact retained bytes,
line103 is the **second** `;LAYER_CHANGE`, followed by `;Z:0.4`; line148 begins
`;Z:0.6`. Use raw line121, XY and Z-header to identify this witness rather
than silently treating differing layer-number conventions as equivalent.
The preceding emitted loop ends at X107.69 Y109.59; its later wipe moves
are not a direct capture of the seam selector's internal last-pos input.

Full-file first difference is earlier: manifest reports byte1718, line81,
Native `M73 P67 R0` versus Ares `G1 X119.69 Y113.69 E.22677`.
Worker5 owns that timing investigation; its M73/wipe differences may be
downstream consequences of the same seam selection error. No independent
time-model root cause is established here. The manifest marks reference producer
identity unknown/cached; therefore these artifacts establish the observed
symptom, **not a fresh identified Native RED for this spec**.

Effective stored config: `seam_position=nearest`, `wall_generator=arachne`,
`wall_loops=1`, `layer_height=0.2`, `slow_down_layers=0`. This is not the
aligned seam-string/spline path, despite the apparent mirrored choice.

## Owning upstream boundary and tie rules

Pinned revision: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
Paths in this section are relative to `OrcaSlicer/src/libslic3r/`.

| File/function | Behavior to preserve |
|---|---|
| `GCode/SeamPlacer.cpp:44–48`, `SeamPlacerImpl::gauss` | float shifted distance, denominator `falloff * shifted * shifted + 1`, exponent `1 / denominator`, result `(exp(exponent)-1)/(exp(1)-1)` for mean0/value1. |
| `:742–795`, `SeamComparator::is_first_better` | Enforcer/type priority, then overhang, then embedded points; nearest adds distance Gaussian to visibility and angle score. Final comparison is strict `<`. |
| `:930–941`, `pick_nearest_seam_point_index` | Initialize to perimeter start; scan candidates in stored order; replace only when strictly better. Exact ties retain the earliest candidate. |
| `:1465–1489`, `SeamPlacer::init` | Nearest skips initial seam picking and alignment; selection is deferred until actual nozzle position is available. |
| `:1500–1562`, `SeamPlacer::place_seam` | Associate loop with candidate perimeter via two consecutive point matches; unfinalized nearest selection uses unscaled `last_pos`; scale the winning candidate. |
| `:1565–1627` | Inner-loop projection/staggering differs from external-loop direct split/snap. Preserve the existing role-specific branch. |
| `GCode.cpp:5769–5775`, `GCode::extrude_loop` | Pass actual `last_pos` or explicit start to SeamPlacer before later loop emission. |

The nearest comparator's type/enforcer handling matters for general painted
inputs, but is outside this unpainted witness's proposed numerical correction.
No new tie-breaker, X preference, symmetry rule or nearest-distance-only
shortcut is part of this rewrite slice.

## Rust comparison and proven source discrepancy

Rust destination boundary:
`crates/ares-core/src/project_slice/seam_placement.rs::gauss_penalty`.
It is the existing compatibility shell for the upstream distance Gaussian;
replace its mismatching arithmetic, not add an Ares seam policy.

Current line34 computes:

```text
(f32::consts::E.exp() - 1).recip() * (exponent.exp() - 1)
```

`E.exp()` is **exp(e)**; upstream uses **exp(1)**. This is a definite source
translation error, not a tie-policy difference or an upstream UB behavior.
At distance0, upstream Gaussian is1; current Rust is approximately0.121397.
The Gaussian's variation with distance is consequently about8.24 times
smaller. Since nearest adds `1 - gauss(distance)` to visibility/angle score,
this changes the relative weight of distance against those other terms.
The constant part cancels in candidate comparisons; the weakened distance
variation can change the winning candidate when visibility differs.

These scalar values are mathematical explanation, not E2E evidence that the
left/right candidates have specific runtime scores or are exactly tied.
The visibility, overhang, embedded values, candidate order, associated
perimeter and exact cursor have not been captured in this read-only task.
**The formula mismatch is proven; its sufficiency to explain the entire
Dumbbell mirror is not yet proven.**

Relevant existing Rust flow:

- `seam_placement.rs::apply_objects` stores nearest penalty data and skips
  aligned selection; `nearest_seam_data` computes visibility+angle score.
- `gcode_emit/motion/perimeter.rs:97–121` passes `local_cursor` into
  `place_nearest_penalized` when nearest data exists.
- `seam_placement/runtime/nearest.rs:34–86` associates the perimeter;
  `:93–121` uses overhang/embedded precedence, distance Gaussian and strict
  `<`, preserving the first candidate on a tie. No contrary tie rule was
  found in this path.
- `gcode_emit/motion.rs:220–237` takes `last_scaled_position` when present.
  Do not infer that cursor solely from the visible wipe endpoint.
- `gauss_penalty` has only the two nearest-distance call sites at
  `runtime/nearest.rs:110,112`, corresponding to upstream `:784–785`.
  Both upstream calls pass `(distance, 0.0f, 1.0f, 0.005f)`.
  Rust's omitted mean and mean-value parameters are fixed0 and1 at every
  caller. The omission itself does not lose configuration or behavior;
  no generic mean-value API is needed for this slice. Aligned
  `angle_penalty` is a separate existing implementation.

## Included, deferred and non-negotiable decisions

Included: port the exact Gaussian normalization denominator and arithmetic
order from `SeamPlacer.cpp:44–48` at the existing nearest-distance helper.
Preserve `mean_value * (expf(exponent) - 1) / (expf(1) - 1)`:
round the exp result to f32, then subtract, multiply by fixed1 and divide
in f32. Do not replace division by reciprocal multiplication or widen the
whole comparator. Follow the existing cross-platform exp strategy in
`seam_placement.rs:239–243`: evaluate exp on the widened f32 operand, narrow
the result once to f32 before subsequent arithmetic. Apply the same strategy
to exp(1); keep the existing angle implementation unchanged. Validate that
policy against the identified Native build and WASM controls rather than
assuming host and browser expf implementations always round identically.

Deferred: visibility sampling/occlusion, candidate extraction/order, KD-tree
ties, painted types, aligned seam strings, angle Gaussian, inner staggering,
cursor/wipe propagation, Arachne geometry, M73/time modeling and scarf/small-
loop speed work. If the numerical correction alone does not explain the
mirror, report that result before expanding the source boundary.

Non-negotiable: no coordinate special case, new tie-breaker, epsilon score
comparison, candidate sorting, legacy fallback, fixture repair, private test
emitter API, ignored assertion or comparator relaxation. Keep core neutral
to WASM/browser, Windows, macOS and Linux; no filesystem/terminal probes in
production. Do not alter worker1/worker5's files or implementation branches.

## Plan and verification after spec approval

1. Rebase latest main and recheck helper/call sites. Copy the exact fixture
   from worker5's branch into worker6's tree, verify its SHA256, and do not
   edit worker5's files. Coordinate the possibly related M73 observation.
   Capture a fresh identified Native/public-byte-API execution with
   exact input/producer/resources/output hashes and owned-process deadlines.
   Do not rerun until a preferred seam appears.
2. Before production edits, commit an independent real E2E RED comparing the
   complete output, normalizing only the generated-by line. Keep the original
   fixture unchanged. The travel XY witness supplements that full assertion;
   the old cached manifest and scalar checks do not substitute for RED.
3. Apply only the approved Gaussian correction. Check whether this same input
   now selects Native's left endpoint. Require complete bytes for GREEN; if
   M73 or other differences remain, retain DIVERGENT and coordinate ownership
   rather than promote a seam-only comparison to whole-file PASS.
   After the correction, notify worker5 to recheck his M73/wipe witnesses;
   identical causality must be established by output evidence, not assumed.
4. Reuse existing nearest/concave/rectangle and other seam-mode E2E controls.
   Preserve candidate-order exact ties and non-nearest behavior. No shallow
   unit test after implementation. If isolated arithmetic testing is needed,
   record its failure inventory and write it before the production edit.
5. Run fmt, strict workspace/all-target/all-feature Clippy, workspace nextest,
   wasm32 check and fresh same-round seeded/official candidate/base control
   sets with independent worker6 target/artifacts. Honor browser holds and
   one-heavy-task/16-thread rules; compare exact failure names and artifacts.
6. Request exact-hash review, await approval, then rebase/revalidate and push
   only fast-forward. This spec authorizes no implementation by itself.
