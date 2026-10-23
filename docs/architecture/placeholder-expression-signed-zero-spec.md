# Expression to_string signed zero

## Source-cited rewrite boundary

OrcaSlicer `src/libslic3r/PlaceholderParser.cpp:343-344` negates a double
without discarding its IEEE sign. expr::to_string :311-324 streams the double
directly into ostringstream, preserving negative zero as `-0`.
Brace output :2173 -> to_string2 :532-537 uses this conversion.

Rust destination: ONLY
`crates/ares-core/src/project_slice/gcode_emit/expression.rs::to_string`.
Its reused motion ostream formatter normalizes zero signs; that is a
compatibility shell for expression conversion, not the upstream sign contract.
Handle negative double zero locally before delegating other doubles to the
existing formatter. Do not change motion::format::ostream or duplicate it.

## Scope and verification

Include only shared expression conversion's negative-zero sign preservation.
Motion formatting, global Value coercion, legacy rendering and arithmetic
remain unchanged. No dependencies/fallback/allow or expected Native constants.
Real official-exported INPUT3MF control `{0.0}` and positive `{-0.0}` in GUI
machine_start_gcode must pass/fail respectively before implementation, then
both pass in official/seeded modes through the shared strict comparator.
Only existing generated-by identity/date normalization.
Input hashes, provenance, actual line27 Native-0/Ares0 and replay evidence:
`/home/indexyz/ares-coord/evidence/w5-ccf-negative-zero-discovery/PROPOSAL-RR.md`.
Stack on approved brace-output GREEN ccf1277b2.
