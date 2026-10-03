# ARD-0097: Layer-change retraction reads the effective filament configuration

The upstream boundary is `Print.cpp::Print::update_filament_maps_to_config`
(lines3180–3195), which applies retraction overrides to the print config,
and `PrintConfig.cpp::compute_filament_override_value` (lines10300–10331),
which maps nullable filament overrides onto machine values. Its consumers
are `GCode.cpp::GCode::apply_print_config`, `FILAMENT_CONFIG` (line1605),
`GCode::change_layer` (line5693), and the layer retraction consumers at
lines5206/5541. Native retraction overrides are resolved before the active
filament reads the effective `retract_when_changing_layer` vector.

The Rust destination is `ares-core::project_slice::gcode_emit::motion::options`
field assembly and its retraction leaf. Existing `ProjectConfigViews::runtime`
already resolves nullable overrides and filament-to-physical-tool inheritance;
consume that view rather than the original full machine vector or a second
independently designed override resolver. The traversal remains a temporary
configuration transport shell around the upstream configuration concepts.

Included: selected-filament effective layer-change retraction, including
explicit false/true and nil inheritance, and complete native saved-3MF output
tests for every value of this nullable boolean. Deferred, not exempted:
other newly observed stock/default and option-domain output residuals, broader
multi-tool retraction combinations, and complete printer/option inventories.
No fixture, comparator or completion gate is loosened.
