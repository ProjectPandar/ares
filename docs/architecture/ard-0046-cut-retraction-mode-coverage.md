# ARD-0046: Cut-retraction mode coverage

## Upstream boundary

`src/libslic3r/PrintConfig.hpp::LongRectrationLevel` names modes 0, 1,
and 2. `PrintConfig.cpp` defines `enable_long_retraction_when_cut`
as an unbounded coInt and `compute_filament_override_value` permits
filament cut-retraction overrides only in EnableFilament mode.
`GCode.cpp::_do_export` exposes the resulting scalar/vector cut
retraction placeholders before rendering machine start G-code.

## Non-negotiable decisions

The Rust destination is the CLI parity option-domain/activation tests and
`ares-core::project_slice::gcode_emit::placeholders`, validating the
existing `options::project_config_views` and G-code placeholder rewrite.
Replace the one-value authored probe with
all three named modes; do not invent an Ares cutting pipeline.

Use conflicting machine/filament values and a start-template witness to
prove runtime override selection, then compare the entire ordered output
with the original binary. Verify requested values in the exported project.
No output normalization beyond existing generator identity/timestamp rules.

Named-mode coverage is not an exhaustive coInt range proof: keep this
domain INCOMPLETE. Existing parity scaffolding is a temporary validation
shell around upstream concepts. Multi-filament changes, physical cutting
sequences, nullable override combinations, arbitrary integers and other
printer contexts are deferred, not certified by this slice.

## E2E-discovered rewrite gap

Mode 2 exports the requested settings but Ares renders machine false/12
instead of original Orca's filament true/16. The runtime placeholder
boundary must consume the resolved runtime cut-retraction vectors, as
upstream `GCode.cpp` uses `m_config`, rather than the full-config footer.
Preserve the full-config footer as supplied. Do not fix this by changing
the comparison, the reference, or the requested inputs.
