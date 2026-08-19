# Spec: KSR FDM Test V4 complete G-code parity

## Observable contract

The `ares slice` project route accepts `tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf` and emits the complete `ksr_fdmtest_v4.gcode` byte stream. Permitted normalization is limited to the single generator metadata line (`OrcaSlicer` becomes `Ares`, and its date/time may differ) and the decimal ID field on `; printing object ... id:<n> copy ...` / `; stop printing object ... id:<n> copy ...` comments. OrcaSlicer 2.4.2's BBL-printer path returns from `GCode::set_object_info` before assigning `PrintObject::m_id`, so the reference ID is indeterminate and cannot be derived from the 3MF. Printing-time metadata is output behavior, not timestamp metadata, and must match.

All behavior is derived from the loaded 3MF model, effective typed project options, and generated geometry. Production code must not inspect fixture names, reference G-code, fixture digests, or known output constants. The CLI integration test is the external seam; focused core tests may cover option-driven motion invariants through `slice_project`.

## Upstream boundaries

Implementation proceeds as source-cited vertical slices of OrcaSlicer 2.4.2:

1. `src/libslic3r/GCode.cpp:6345-7047` and `src/libslic3r/GCodeWriter.cpp:587-617` — role speed selection, volumetric extrusion, travel, relative E formatting, acceleration, and processor tags.
2. `src/libslic3r/GCode.cpp:5744-6127` and `src/libslic3r/GCode/SeamPlacer.cpp` — loop seam placement, clipping, entity chaining, retraction, lift, and wipe paths.
3. `src/libslic3r/GCode.cpp:6990-7110` and the arc-fitting implementation consumed there — option-controlled G2/G3 emission.
4. `src/libslic3r/GCode.cpp:4539-6228` — object/layer prologues, fan and custom templates, ordered entity emission, and end sequence.
5. `src/libslic3r/GCode/GCodeProcessor.cpp:1100-1140` and its time processor — M73 progress and header time replacement.
6. `src/libslic3r/GCode.cpp:5348-5351`, `5471-5475`, and `8072-8099`, plus `src/libslic3r/Print.hpp:468-469,581-582` — object-comment ID normalization at the golden seam for the BBL path's uninitialized `PrintObject::m_id`; production Ares still emits deterministic project-derived IDs.
7. `src/libslic3r/GCode.cpp` export finalization — executable block termination and filament statistics.

## Incremental acceptance

Each slice adds a failing observable assertion before implementation, derives values from existing typed options, keeps Rust files below 400 LOC, and commits/pushes independently. Obsolete tests that pin Ares internals to encoded Orca source-stage artifacts are removed; behavior and fixture-output tests remain.

Completion requires the unignored normalized byte-for-byte CLI golden test, `cargo nextest run --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, an independent six-axis review, fixes, and reviewer re-verification.