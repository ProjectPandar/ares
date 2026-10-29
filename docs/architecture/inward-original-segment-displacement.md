# Loop-end inward move: preserve the original displacement segment

## Source boundary and destination

Port the distinction inside `OrcaSlicer/src/libslic3r/GCode.cpp`,
`GCode::extrude_loop` lines 5996–6023: orientation-dependent angle rays versus
orientation-independent first-segment displacement. Angle calculation consumes
`Point.cpp::Point::ccw_angle` (81–89). Rust destination is the existing private
`crates/ares-core/src/project_slice/gcode_emit/motion/loop_paths/inward.rs`,
`inward_point`, called by `append_inward_move`.

The existing loop/path carriers are temporary compatibility adapters for
`ExtrusionLoop`/`ExtrusionPath`, not a new Ares motion pipeline. No API shape,
configuration, file I/O, viewer or platform-specific core behavior is added.

## Non-negotiable algorithm boundary

Upstream copies the second point and the end-minus-three point into `a`/`b`.
When `is_hole == loop.is_counter_clockwise()`, it swaps those local angle
rays and negates one third of the resulting CCW angle. It then constructs
`p1` and `p2` afresh from the unchanged `paths.front().polyline.points[0..2]`.
The displacement vector `v = p2 - p1` therefore always follows the original
first segment, regardless of the angle-ray swap.

Rust currently swaps `first` and `second` in place and then uses the swapped
`first` to compute displacement. Preserve the original first-segment endpoint
before selecting angle rays, so the source distinction is represented without
changing the function's callers, orientation rule, threshold or nozzle cap.

Included: only that source-owned displacement choice and complete public
output coverage of its observed consequence. Deferred: truncating the
pre-rotation displacement destination to `coord_t`, rotation rounding,
clipped-tail vertex selection, angle mathematics, scarf/wipe-before-external
consumers, generator voxels, actual wipe-path clipping and cooling/feedrates.
The adjacent cast-order difference has no independently admitted public RED
and must not be folded into this GREEN. If this bounded fix leaves any target
stream different, report the residual; do not broaden silently or claim GREEN.

## Runtime discovery and falsifiable hypothesis

On the approved clean-generator main, the frozen orthogonal-grid input has
one first difference at byte 100825, line 5137. It is the writer-only inward
move before `;WIPE_START`, not a retracting wipe segment:

- Native: `G1 X106.655 Y106.098`.
- Rust: `G1 X106.718 Y106.107`.

Read-only debugger receipts show the same center, original angle rays and
`(is_hole, counter_clockwise) = (true, true)`. Native retains the original
first segment for displacement. Independent arithmetic with the captured
values reproduces Native `(106.655111, 106.097513)` and Rust
`(106.717887, 106.107467)` exactly at printed precision. Evidence is external:
`/home/indexyz/ares-evidence/w6-orthogonal-inward-readonly-finding.json`.
Hypothesis: separating displacement from swapped angle rays removes the
entire strict-output discrepancy, without touching voxel or wipe generation.
This remains a hypothesis until fresh public RED/GREEN replay.

## Native-first test seam and failure inventory

Use the already agreed public seams: Native CLI twice per input, validated
byte-in/options core API, and real Ares CLI. Freeze the unchanged input-only
`orthogonal-grid-beam1.3mf` from `worker6/interlocking-wip` with SHA
`afed0f4f4417916d2963363754bc45ef1300f6bbc2d6dfbcc9d52cf751a0c312`.
Use the original default beam1 and single-volume frozen inputs as PASS
controls, preserving their SHA and existing complete comparisons.

Failure modes: using the swapped angle ray for displacement; accidentally
changing the angle or orientation predicate; disturbing non-swapped default
controls; altering generator/path geometry to compensate; masking an inward
move difference in normalization. Each case requires stable Native double
replay before Ares comparisons, API/CLI raw equality, and both complete
ordered Native/API and Native/CLI comparisons with the existing generator-only
normalizer. Keep launch/input/output/comparison artifacts outside Git.
No ignored tests, Native output, probes, new allow/expect, unsupported guards,
fallbacks, comparator changes or hand-authored output expectations.

## Review sequence and acceptance

Branch `worker6/inward-displacement` is ordered after the approved
endpoint-contact chain `3b446a687`, itself based on
`a5eb0f0356057fe9c62ec3319834f82b40a7731a`. Its RED commit moves the endpoint
and inward Native/API/CLI harness into a shared runner without changing
assertions. Both suites reuse existing `interlocking-base` control inputs
instead of storing duplicate archives. No endpoint production change or old
WIP history is included in the inward commits.

1. Commit this spec and send SPEC REVIEW-REQUEST before tests/implementation.
2. After spec review, freeze the target and PASS controls; observe and commit
   genuine Native-first public runtime RED in official and seeded modes.
3. Production-only GREEN changes only displacement selection in `inward.rs`.
   Both target streams and controls must wholly match; partial improvement
   is not completion. Existing interlocking-base controls must remain PASS.
4. Changed files stay below 400 lines; fmt and
   `cargo clippy --workspace --all-targets -- -D warnings` must be clean.
5. Integration follows the coordinator queue. Rebase to then-current main and
   run both complete FULL modes at the exact final hash, each zero failures
   and zero skipped. Earlier-baseline results cannot waive that gate.
6. No stash, push or self-merge; every commit includes the required coauthor.

Other unresolved WIP cases remain outside this slice. No claim of complete
interlocking or general inward-move parity follows from this bounded repair.
