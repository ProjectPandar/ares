# Brace expression result conversion

## Source-cited Rust rewrite boundary

Port OrcaSlicer `src/libslic3r/PlaceholderParser.cpp:2173` brace
conditional_expression -> expr::to_string2 (:532-537) -> expr::to_string
(:302-327) into the brace-output branch of
`crates/ares-core/src/project_slice/gcode_emit/template.rs`.

Upstream doubles use default ostream conversion, including six significant
digits and scientific notation selection. Booleans use true/false (:309).
Rust currently routes brace output through Value::as_string, whose integral
Number shortcut bypasses ostream notation; this is a temporary compatibility
shell, not the upstream result rendering contract.

## Included and deferred

Change ONLY brace expression output to reuse the ONE existing shared
expression::to_string helper from comparison/addition. Widen helper visibility
inside gcode_emit as needed, without changing its implementation or adding a
formatter. No global Value conversion/coercion changes. Legacy [var]
expansion, parser/admission/errors and unrelated numeric rendering remain
deferred. Boolean formatting already uses true/false, so no speculative
boolean test or change is included without another Native difference.

## E2E acceptance

Real official Native-exported INPUT3MFs use GUI machine_start_gcode comments:
control `{999999.0}`, positive `{1000000.0}`. Shared complete stream comparator
must pass control and genuinely fail positive before implementation (line27
Native1e+06 / Ares1000000), then pass both official/seeded modes. Existing
generated-by identity/date normalization only. Provenance/input hashes:
`/home/indexyz/ares-coord/evidence/w5-ebfb-double-text-discovery/PROPOSAL-RR.md`.
Stack on approved max_print_z GREEN ebfbf309d for combined integration.
