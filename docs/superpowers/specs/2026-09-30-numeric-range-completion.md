# Honest numeric-range completion

Use ARD-0059's source-cited `ConfigOptionDef` defaults and the existing Rust
`option_coverage::{domains,bounds,OptionOutcome}` boundary. The coverage adapter
is only a temporary shell around upstream schema behavior.

Require min, max and a seeded sample of the actual legal domain before reporting
complete numeric coverage. Existing local comparisons remain evidence, but
one-sided/unresolved/default-bound domains remain INCOMPLETE until those
requirements are proved. Distinguish unsupported source expressions from absent
assignments, and do not treat a numeric prefix of an expression as its value.
Width-schema extraction shares that parser. Explicit bounded percent/absolute
branches both draw their seeded value from the full min/max interval; GUI
`max_literal` does not restrict it. Verify changed literal samples with Orca.

Retain verified skirt/hot-plate/vitrification exceptions and finite Boolean/enum
behavior. Tests must observe generated plans and completion outcomes, not pin
upstream code text. Original-binary selected sweeps must retain full paired
outputs even when the stricter completion gate rejects the run. Defer the actual
missing endpoint executions to follow-up slices; do not declare global parity.
