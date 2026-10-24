# ARD — Auto-lift geometric query coordinates

## Decision and source ownership

`libslic3r/GCode.cpp::GCode::needs_retraction:7593–7604` owns the clipped travel query's current-plate frame: translate by `m_origin - m_writer.get_xy_offset()`. `GCodeWriter.hpp:98–99` defines the plate-offset accessor; the extruder-offset subtraction used by emitted nozzle coordinates (`GCode.cpp:8124–8127`) is a different transform.

Correct only the translation in existing Rust `project_slice/gcode_emit/motion/auto_lift.rs::AutoLift::crosses_overhang`. `motion/origin.rs::set_origin` proves `state.offset` contains extruder-offset subtraction and must not define this geometric query frame. Retain the existing caller in `motion/path/start_travel/approach.rs`, corresponding to upstream Auto Spiral/Slope selection (`GCode.cpp:7607–7615,7635–7647`). AutoLift remains a compatibility shell around that named upstream boundary, not an independent Ares pipeline.

## Scope constraints

Include only the query-frame correction and verification of its existing spiral/progress consequences. For the reviewed zero-plate-offset witness, the upstream transform is the existing plate-relative origin. New plate-offset plumbing or nonzero-plate cache-domain changes require separate review.

Preserve scaling, clipped endpoint truncation, cache geometry/detection, z-window, retraction/wipe, fallback and explicit modes, output-coordinate conversion, writer formatting and processor behavior. Do not repair downstream M73 bytes directly: the missing spiral changes the global time used by existing progress generation. Tokenizer, fan and zero-E ownership remain separate. Add no API, fallback, feature flag or new pipeline abstraction.

Use the original P1S two-object ByLayer public Native-first witness, exact spiral and early progress assertions, raw API/CLI equality and the unchanged complete-output comparator. Docs-first SPEC review precedes genuine tests-only RED review; only then may translation-only GREEN proceed. No FULL or integration; review and hold after every phase. See [SPEC](auto-lift-query-frame-spec.md) for the bounded test contract.
