# ARD-0045: Locked Zag width range evidence is context-specific

The upstream owner is `libslic3r/PrintConfig.cpp`'s skin/skeleton width
definitions and `validate(const FullPrintConfig&, bool)` width ceiling,
`Print.cpp::Print::validate`'s `validate_extrusion_width`,
`Fill/Fill.cpp::group_fills` and `Flow.cpp::new_from_config_width`.
The Rust destination is the existing CLI option-domain generator and
project-to-complete-G-code regression tests; the core flow rewrite is not
changed. This continues ARD-0044, not a new Ares slicing pipeline.

At nozzle 0.4 mm and layer height 0.2 mm, validation permits zero or
positive widths in (0.2, 2] mm. Literal zero invokes automatic flow width;
percent zero instead resolves to zero and fails Flow spacing. Therefore
the two wire units cannot share a single zero-success assumption.

Execute literal auto, 2 mm/500% upper endpoints, fixed-seed interior values
in both units, and near-lower-edge 0.20001 mm/50.001% values that survive
export serialization. The latter are near-edge samples, not mathematical
minima of an open interval or a claim about every representable float.
Preserve typed export/dependency proof and actual sparse-extrusion witnesses.
Execute rejection checks for 0%, 0.2 mm/50%, and 2.01 mm/501%; retain
command evidence and do not count these as matching G-code streams.
Upper violations fail CLI export validation (-18); exact lower bounds
fail slice validation (-51), and percent zero fails Flow spacing (-100).

Keep the domains INCOMPLETE: source schema upper bounds default to
FLT_MAX, and a single feasible nozzle/layer context does not establish
the requested all-printer/global-range coverage. Include the above range
evidence; defer other nozzle/layer contexts, raw-schema extremes,
object/region overrides and other Locked Zag options. Existing scaffolding
is temporary wherever it does not yet match the source-cited behavior.

Do not change reference binaries, presets, output normalization or the
completion gate to obtain a passing result. Every successful case still
compares complete ordered output with generator identity/time normalization
only; rejected cases are separate reference-feasibility evidence.
