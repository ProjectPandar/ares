# ARD: raft conditional scarf without a previous-layer distance

Status: historical undefined upstream boundary.

The former first-object-over-raft Native comparison is not an active parity
contract. The deterministic Rust behavior below records implementation
history, not defined C++ behavior or a numerical interpretation of its bits.

## Source boundary

Pinned OrcaSlicer: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

- `src/libslic3r/GCode.cpp:5777-5791`: `layer_id() > 0` admits the first
  object layer over a raft; conditional scarf compares `seam_overhang` with
  `scarf_overhang_threshold * 0.01f * line_width`.
- `src/libslic3r/GCode/SeamPlacer.hpp:59-70`: the `SeamCandidate` constructor
  does not initialize `unsupported_dist`.
- `src/libslic3r/GCode/SeamPlacer.cpp:1083-1092,1563`: a previous-layer
  distancer assigns the field; seam selection later reads it.

The first object layer has no previous object-layer distancer. Reading that
field is undefined in C++; the source does not specify a numerical value.
Repeated Native admission at 5% and 40% does **not** prove the field is zero.
It observes the comparison outcome for the pinned binary and fixtures only.

## Historical Rust boundary

The earlier implementation selected `0.0` for an absent support distance in
`crates/ares-core/src/project_slice/gcode_emit/motion/scarf/eligibility.rs`
and removed the corresponding rejection in
`crates/ares-core/src/project_slice/capabilities.rs`. This is a deterministic
choice at an undefined upstream boundary, not Native parity. This document
does not extend that choice or authorize a rewrite of undefined behavior.
Available distances continue through the upstream comparison.

The conditional-over-raft first-object-layer Native comparison is historical.
A property premise cannot make its complete comparison stable: Native can
change output while the input and Rust output remain unchanged. Active Native
fixtures must avoid the uninitialized read, not remove bytes or relax the
complete comparison. Seam selection, raft geometry and scarf construction
remain owned by their existing source boundaries.

## Observed Native run variation

At upstream `345efec64c7b95a05ce5cf96df0dde66a158835c`, the same conditional
raft input produced 64 slope moves on object layers 3–6 in one Native run and
48 on layers 4–6 in another; the latter had no layer-3 scarf, the first object
layer over the raft. Ares produced the 64 form. The 48 run also failed both
complete Native/API and Native/CLI comparisons: the first difference was at
byte 3208, line 103, where Native emitted `M73 P28 R2` and Ares emitted
`G1 X107.504 Y119.753 E.04556 ; support material`. These counts are
observations, not expectations. The original fixture and its comparison are
retained as history, not as a repeatability or parity assertion.

## Active defined-distance fixture boundary

`tests/parity/scarf-raft-defined/` uses the same taper geometry and two raft
layers, with zero walls on the first object layer through a height-range
region override. Later walls have nonempty prior object slices. The positive
uses external conditional scarf; the otherwise identical control uses scarf
type `none`. The original UB module comment in `scarf_conditional_raft.rs`
remains verbatim as the historical explanation.

The current source pin above calls `place_seam` at `GCode.cpp:8519` before
the conditional guards at 8534–8541. `SeamPlacer.cpp:2384` reads
`unsupported_dist` unconditionally; disabling conditional scarf alone is not
a source-level avoidance proof. The new first object layer has no
`place_seam` entries. Later distance construction uses nonempty prior slices
through `calculate_overhangs_and_layer_embedding:1683–1727`.

The height-range XML uses one-based ModelObject index 1, not package object
ID 2 (`bbs_3mf.cpp:2235–2238`), and supplies `layer_height` as well as
`wall_loops` (`Slicing.cpp:258`). The test checks activation and wall/role/layer
properties, then retains the complete Native/API/CLI comparisons. There is no
slope-count, first-motion, coordinate or extrusion-value golden, and no claim
about scarf on an undefined first-object-layer seam.
