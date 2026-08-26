# Spec: KSR FDM Test V4 semantic G-code parity

## Observable contract

The `ares slice` project route accepts
`tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf` and emits complete,
production-usable G-code. The external golden seam compares semantic print
behavior rather than the captured order of independent islands. It validates:

1. exactly 460 layers with exact Z and layer-height metadata;
2. an exact multiset of deposited G0/G1/G2/G3 segments per layer, including
   feature, line width, endpoints, arc center, extrusion, acceleration, and fan
   state;
3. exact wipe paths, relative-E retract/unretract moves, and lift command/Z
   lifecycle;
4. exact configuration, start/end templates, object-label events, and
   non-motion control events;
5. exact filament volume, mass, and cost, with displayed filament length
   within 0.05 mm;
6. cooling-derived deposition feed rates within both 10 mm/min and 1%, and
   estimated times within five seconds.

The seam validates one complete generator line and normalizes only its slicer
name/date, normalizes the indeterminate Orca object-ID decimal field, and
ignores M73 placement and independent-island travel order. These fields are
derived from an upstream order that is not stable: OrcaSlicer 2.4.2
`TriangleMeshSlicer.cpp:521-529` appends TBB-produced intersections under a
mutex without defining append order. Two same-binary, same-input AppImage runs
first differed geometrically at the Z1.2 travel lift and varied cooling feed by
7 mm/min (0.517%). Reproducing one captured schedule would require forbidden
fixture-specific data.

All behavior is derived from the loaded 3MF model, effective typed project
options, and generated geometry. Production code must not inspect fixture
names, reference G-code, fixture digests, or known output constants. The CLI
integration test is the external seam; focused core tests may cover
option-driven motion invariants through `slice_project`.

## Upstream boundaries

Implementation proceeds as source-cited vertical slices of OrcaSlicer 2.4.2:

1. `src/libslic3r/GCode.cpp:6345-7047` and `src/libslic3r/GCodeWriter.cpp:587-617` — role speed selection, volumetric extrusion, travel, relative E formatting, acceleration, and processor tags.
2. `src/libslic3r/GCode.cpp:5744-6127` and `src/libslic3r/GCode/SeamPlacer.cpp` — loop seam placement, clipping, entity chaining, retraction, lift, and wipe paths.
3. `src/libslic3r/GCode.cpp:6990-7110` and the arc-fitting implementation consumed there — option-controlled G2/G3 emission.
4. `src/libslic3r/GCode.cpp:4539-6228` — object/layer prologues, fan and custom templates, ordered entity emission, and end sequence.
5. `src/libslic3r/GCode/GCodeProcessor.cpp:1100-1140` and its time processor — M73 progress and header time replacement.
6. `src/libslic3r/GCode.cpp:5348-5351`, `5471-5475`, and `8072-8099`, plus `src/libslic3r/Print.hpp:468-469,581-582` — object-comment ID normalization at the golden seam for the BBL path's uninitialized `PrintObject::m_id`; production Ares still emits deterministic project-derived IDs.
7. `src/libslic3r/TriangleMeshSlicer.cpp:511-531` and
   `src/libslic3r/ShortestPath.cpp` — nondeterministic parallel intersection
   collection is excluded from the observable ordering contract; deterministic
   Ares island ordering must preserve the same toolpath multiset.

## Incremental acceptance

Each slice adds a failing observable assertion before implementation, derives
values from existing typed options, keeps Rust files below 400 LOC, and
commits/pushes independently. Obsolete tests that pin Ares internals to encoded
Orca source-stage artifacts are removed; behavior and fixture-output tests
remain.

Completion requires the unignored semantic CLI golden test,
`cargo nextest run --workspace`,
`cargo clippy --workspace --all-targets -- -D warnings`,
`cargo fmt --all -- --check`, an independent six-axis review, fixes, and
reviewer re-verification.