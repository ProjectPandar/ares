# Cut-retraction mode coverage

Source boundary: `PrintConfig.hpp::LongRectrationLevel`,
`PrintConfig.cpp::compute_filament_override_value`, and
`GCode.cpp::_do_export` cut-retraction placeholder initialization.
Rust destination: CLI option-domain activation and complete-output tests
of the existing core config-view/placeholder rewrite. See ARD-0046.

The active E2E exposes mode-2 stale placeholder values. Repair the core
placeholder boundary to read resolved runtime cut-retraction vectors while
retaining the original full-config footer and strict output comparison.

Include modes 0/1/2 with conflicting machine and filament settings in both
Boolean directions (six complete-output comparisons).
Require exported-project application proof, rendered runtime-value
witnesses and whole-stream byte parity with original Orca.
Keep the integer domain INCOMPLETE: these named semantic modes do not
establish raw numeric endpoints. Physical cutting/tool changes, other
contexts and nullable combinations remain deferred.

Acceptance: domain RED/GREEN, active E2E mode comparisons, selected sweep,
workspace Nextest, fmt/Clippy, Tier 1 compiler checks and independent
six-axis review. Code and documents commit/push together after repair
and re-review. Scaffolding remains a temporary upstream validation shell.
