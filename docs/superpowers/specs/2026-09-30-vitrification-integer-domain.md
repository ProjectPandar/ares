# Vitrification integer-domain coverage

Follow ARD-0058 and its `PrintConfig`, `ConfigOptionInts` and `GCode::_do_export`
source boundaries. Replace the lone numeric authored probe with the signed
32-bit serializable minimum, maximum (INT_MAX-1) and deterministic interior, retaining
110 as an additional regression. INT_MAX is the upstream non-nullable nil
sentinel and must have explicit rejection evidence, never a full-output pass.
Rust destination: existing project wire and
placeholder behavior, plus the native option-domain test adapter.

Acceptance: prove requested values survive official export, assert the original
start template emits each value, then compare complete ordered output and retain
both G-code files. No source-structure tests or comparator relaxation. The
single-filament fixture does not prove multi-filament minimum selection or
arrangement/UI behavior. The adapter remains only a temporary compatibility
shell around the source-cited upstream behavior.
