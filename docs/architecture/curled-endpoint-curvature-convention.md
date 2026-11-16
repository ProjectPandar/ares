# Curled-perimeter curvature window sampling

Status: revised specification for review. No test or production fix authorized
by this document alone.

## Source-owned rewrite boundary

Port the window-sample position update semantics of OrcaSlicer 2.5.0
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`,
`src/libslic3r/GCode/ExtrusionProcessor.hpp::estimate_points_properties`:

- `:65–90`: previous/next index handling, including closed-input wrapping.
- `:372–381`: predecessor segment distances and accumulated length.
- `:383–435`: 3/9/16 mm curvature windows and greatest-magnitude selection.
- **`:387–405`**: backward sampling position starts at `current.position`;
  only the partial-segment interpolation branch changes `back_position`.
  Whole-segment progress changes the index and accumulated distance only.
- **`:407–425`**: the forward walk has the same rule for `front_position`.
- `:428–433`: `angle(current-back, front-current) / window_size`, retaining
  the greatest magnitude.

`src/libslic3r/Point.hpp:136–143::angle` casts its vectors to double and
returns `atan2(cross2(v1,v2), v1.dot(v2))`. There is **no zero-length guard or
special return** there or in the curvature window loop.

The Rust destination is
`crates/ares-core/src/project_slice/gcode_emit/motion/overhang/curl/geometry.rs`
`::walk` (`:250–294` before implementation). Its current whole-segment branch
at `:291–293` advances the index **and** assigns `position=points[index]`.
That assignment is not present upstream. `curvatures` at `:200–240` consumes
these samples and computes the angle at `:230–237`.

This slice replaces that divergent sampling behavior with the upstream rule.
It is a bounded `libslic3r` rewrite, not an independently designed endpoint
curvature policy.

## Captured arithmetic and required behavior

The target is zero-based physical layer2, line9, whose first segment is
intentionally degenerate in `SupportSpotsGenerator.cpp:158–161`:
`a = i>0 ? points[i-1] : points[i]`, `b = points[i]`.

Both sides receive this four-point, **non-looped** entity:

```text
(4.896053,4.853553)
(4.874803,4.890359)
(4.811053,4.853553)
(4.874803,4.816747)
```

The sequence is shorter than every half-window (1.5/4.5/8 mm). At point0,
backward traversal cannot move; forward traversal exhausts the sequence
without entering the partial-segment interpolation branch.

Native GDB at the actual `atan2` call `0x2d77fa6` captures, for windows3/9/16:

```text
current = back = front = (4.896053,4.853553)
cross = +0, bits 0x0000000000000000
dot   = +0, bits 0x0000000000000000
atan2(+0,+0) = 0
```

The compiled operands are constructed at `0x2d77f41–0x2d77fa2`, immediately
before `atan2`, not inferred from the final curvature. Both vectors are zero
because both sample positions remain current.

Rust GDB at `geometry.rs:235` instead captures:

```text
current = back = (4.896053,4.853553)
front = (4.874803,4.816747)
cross = +0
dot   = -0
atan2(+0,-0) = pi
```

The Rust whole-step position assignment moved the forward sample to the last
point. Its negative displacement multiplied by the zero backward vector
produces the negative-zero dot. The 3 mm window then retains
`1.0471975803375244` rather than Native0.

Required implementation behavior:

1. Keep the sample position initialized to the starting point.
2. On whole-segment progress, update index and accumulated distance, **not
   the sample position**.
3. On partial-segment interpolation, construct the sample from the current
   traversal point `points[index]`, as upstream does, rather than the still
   initial sample position. This is necessary to retain upstream behavior
   when the walk crosses whole segments before finding a partial segment.
4. If traversal exhausts the polyline before reaching half a window, return
   the unchanged starting sample, naturally yielding the upstream vectors
   and signed-zero operands.

Do **not** add an `angle=0` zero-vector branch, clamp signed zeros, force
endpoint curvature, change loop wrapping, or change the angle formula. The
result must follow from the upstream sample-vector construction.

## Zero-length partial interpolation

The partial branch in `ExtrusionProcessor.hpp:395–400,415–420` assigns its
sample even when the traversed direction has zero length. The source-pinned
Eigen5.0.1 `Eigen/src/Core/Dot.h:83–102`, `MatrixBase::normalized()`, returns
a copy of the original vector when its squared norm is zero. This is defined
behavior, not division by zero or uninitialized memory.

The first implementation draft removed whole-step sample writes but retained
an Ares `if norm > 0` around the partial assignment. The old whole-step writes
had masked that guard's mismatch: after whole-step progress, a zero-direction
partial must assign the current anchor, not retain the initial sample.

A whole-output-qualified Native MK4 capture demonstrates the branch:
layer0, entity0, point1, window3 starts at `(-4.760001,4.760001)`;
a zero-distance whole step advances backward to point0 at
`(4.760001,4.760001)`. The following partial has zero direction. Native
normalization preserves that direction (the compiled divisor is `(1,1)`),
then writes `(4.760001,4.760001)` to the back sample. Its resulting curvature
is `0.5235987901687622`. The initial draft's guard instead keeps the initial
back sample. This also affects the Sermoon witnesses; the complete first-draft
inventory rejects four P→F identities in each mode.

Both directions must therefore implement Eigen's zero-vector preservation
and assign the partial sample unconditionally. A unit divisor for a zero
normalization length preserves the vector and the existing positive-length
arithmetic order. Do not change precision, angle guards, signed-zero handling,
whole-step behavior, or any consumer to compensate. Evidence is retained in
`tiq2-curvature-regression-firstdraft-64a5e4ab9-r1/native-gdb` and
`native-first-zero-branch.json`; its Native whole output matches the public
MK4 strict-test artifact.

## Included and deferred behavior

Included: backward and forward window sample position updates and the
interpolation anchor and unconditional zero-length partial assignment required
by that update rule, for the shared curl
`curvatures` helper. The existing module is the temporary compatibility shell
around the upstream annotation concept; replace only this mismatched rule.

Deferred: the earlier one-ULP non-degenerate curvature difference at layer1
line3, f32/f64 distance accumulation and angle narrowing, closure-detection
policy, general point annotation, other geometry changes, tolerance/height
formula, overlap/fan/cooling consumers, Type1 retraction and unrelated work.
No general precision change or unconditional direction reset is included.

## Causal witness and review/verification gate

The upstream curvature consumer is
`SupportSpotsGenerator.cpp:173–174`, the average of endpoint curvatures.
The height formula at `:94–130` uses the tension term only inside the
malformation band. At layer2 line9, distance/width/layer-height/previous-height
match; the band is active. Native curvature0 leaves only swelling height
`0.04159177094697952`; Rust curvature1.04719758 activates tension and yields
`0.40586769580841064`. This seeds the later previous-curled input difference,
overlap difference and fan START position. No output compensation is allowed.

After coordinator review and implementation GO:

1. Add the minimal public synthetic E2E witness for a non-looped entity
   shorter than the half-window, retaining a repeatable Native/Rust artifact;
   preserve its genuinely failing unchanged RED before production changes.
2. Use the existing public TiQ2 retained-role-fan sequence witness as the
   fan-order F→P target. Preserve the original full-file witness and report
   any later remaining difference honestly rather than relaxing comparison.
3. Port only the sample-position rule and interpolation anchor above.
4. Run the identical ordered inventory in both official/seeded modes,
   including **all overhang-related cases** (speed and fan), require P→F=0,
   and preserve fixture bytes, complete ordered comparisons, line counts,
   config/footer/marker assertions and generated-by-only normalization.
5. Run nextest via the project harness, rustfmt, workspace all-target Clippy
   with warnings denied, and core wasm32 checks. No test/fix is part of this
   spec-only handoff.

Evidence:
`/home/indexyz/ares-evidence/worker2/tiq2-curvature-source-acee1a297-r1/`
contains Native atan2 operand bits, Rust sample positions/angle terms and
source-disassembly mapping. Batch CSV witnesses are under
`tiq2-curl-batch-acee1a297-r1`. Failed probes are excluded from conclusions.
