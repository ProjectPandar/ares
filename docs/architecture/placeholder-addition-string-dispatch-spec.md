# Placeholder addition string dispatch

## Upstream rewrite boundary

Port OrcaSlicer `src/libslic3r/PlaceholderParser.cpp:435-456`,
expr::operator+= string operand dispatch, to
`crates/ares-core/src/project_slice/gcode_emit/expression.rs::arithmetic`.
If lhs is string append rhs.to_string(); otherwise if rhs is string append
rhs to lhs.to_string(); only otherwise retain existing numeric addition.
Current numeric coercion of string operands is a temporary compatibility
shell, not upstream addition behavior.

Promote comparison.rs::comparison_string to ONE shared expression to_string
helper, citing upstream expr::to_string :302-327. Both comparison and addition
must use it; preserve the existing ostream numeric converter and bool/string/
integer formatting. No duplicated conversion implementation or global Value
coercion change.

## Scope

Only + with either string operand changes. Preserve operand order and
numeric addition. Defer - * / %, non-string admission/errors and global
coercion behavior. No parser changes, dependency, fallback or output constants.
This is a libslic3r rewrite boundary, not a new Ares pipeline.

## E2E acceptance

Real official Native-exported 3MFs use GUI machine_start_gcode comments:
control `{2 + 2}`, positive `{"2" + 2}`. Shared complete Native stream
comparator must pass control and genuinely fail positive before implementation,
then pass both in official and seeded modes. Only existing generated-by
identity/date normalization. Inputs/provenance/hashes/actual line27 difference:
`/home/indexyz/ares-coord/evidence/w5-184b-string-add-discovery/PROPOSAL-RR.md`.
Stack on approved compare_op GREEN 184b4ab12 for combined integration.
