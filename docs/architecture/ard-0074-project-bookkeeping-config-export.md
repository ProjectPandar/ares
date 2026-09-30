# ARD-0074: Project bookkeeping config export

Upstream boundary: `PrintConfig.cpp:1828-1830` defines coStrings bookkeeping;
`GCode.cpp::append_full_config:5636-5655` serializes configured keys.
`OrcaSlicer.cpp:2815-2818` creates `different_settings_to_system` at
filament_count+2. CLI preparation remains a temporary compatibility shell,
not an Ares pipeline goal.

Rust destination: `options/config_export/writer.rs`. Include absent-key
construction and supplied-token preservation; defer preset inheritance,
updates and recursive profile comparisons. Do not retune slicing geometry.

The two-filament public witness also exposes a missing initial tool command.
Include `GCodeWriter.cpp::toolchange:565-584`, `toolchange_prefix:547-563`
and `reset_e:480-500` at the existing `gcode_emit/layer_gcode.rs` initial
filament selection boundary. Port the non-BBL configured-multiple-filament
condition, flavor/manual prefix, optional comment and absolute-E reset;
defer later dynamic tool changes. This is necessary for complete output,
not permission to ignore the command in the comparison.

Public native project-byte to complete G-code witnesses must cover one/two
filaments and supplied nonempty bookkeeping. Keep original KSR full RED;
retain its old official stream. A separate independently captured initialized
native full fixture may replace the UB-dependent semantic diagnostic oracle,
but comparison rules must not change or substitute for ordered-byte acceptance.

Require sensitive full RED/GREEN, source-owned counterfactual, no filters,
no source pinning or KSR-specific token, modules below400 LOC, workspace
nextest/fmt/Clippy/Tier-1 checks and independent six-axis review before
code/docs commit/push together.

KSR migration requires the paired `inherits_group` creation at the same CLI
source, and automatic AMS defaults from `OrcaSlicer.cpp:5993-6022` in existing
`config_export/transform.rs` (no empty-only gate). `Print.cpp:3166-3170`
rebuilds the original full config when the filament-map vector changes;
the two-filament Ender prime-tower config divergence remains deferred until
the native/Rust producer-map comparison proves the changed-map predicate. These
are existing export validation shells, not new slicing or preset orchestration.
Supplied bookkeeping preservation is scoped to already correctly sized vectors;
short/long supplied-vector resize remains deferred and unclaimed.
