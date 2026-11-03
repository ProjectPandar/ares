# Placeholder min/max numeric result typing

## Upstream rewrite boundary

Port result promotion from OrcaSlicer
`src/libslic3r/PlaceholderParser.cpp:617-644`,
`expr::function_2params`, `min`, `max`, to the min/max arms in
`crates/ares-core/src/project_slice/gcode_emit/expression/functions.rs`.
Upstream preserves TYPE_INT for integer operands and produces TYPE_DOUBLE
when either operand is double. `operator/=` :494-510 observes this distinction.
The current always-Number reduction is a temporary compatibility shell.

## Included and deferred

Include only min/max result typing: all-integer arguments yield
Value::Integer, otherwise retain Value::Number. Share a single min/max path,
without duplicating selection/promotion logic. Preserve the existing reduction
and admission behavior. Do not alter other functions or arithmetic.
Arity/admission/errors, invalid inputs and unrelated numeric behavior remain
deferred. No new dependencies, fallback, allow, include! or output constants.

## E2E acceptance

Real official-exported INPUT3MFs use GUI Machine start G-code comments:
- control `{min(5.0, 7) / 2}` (floating promotion retained)
- min positive `{min(5, 7) / 2}`
- max positive `{max(3, 5) / 2}`

Shared complete Native stream comparison must pass control and fail both
positives before implementation, then pass all three in official and seeded
modes. Only existing generated-by identity/date normalization. Input provenance,
SHA256 and observed line27 differences are retained in
`${COORD_ROOT}/evidence/w5-6a2e-minmax-discovery/PROPOSAL-RR.md`.
Stack on approved rounding GREEN 6a2e96e1f for joint W5 integration.
