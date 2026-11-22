# Writer temperature placeholders

## Upstream contract

This slice ports the shared `GCode::_do_export` parser bindings from OrcaSlicer 2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`). `src/libslic3r/GCode.cpp:3648,3679,8386` registers writer extruders before template rendering. At `4001–4013,4031–4034`, `overall_chamber_temperature` is the maximum over those IDs, starting at zero independently of chamber activation, and `min_vitrification_temperature` is the minimum over those IDs, starting at `INT_MAX`. Both are shared integer options. `src/libslic3r/Config.hpp:676–680` indexes each nonempty filament vector by logical tool ID, using its first slot for an out-of-range ID.

Successful public exports do not expose an empty writer set: `Print::validate` rejects a print with no extrusions (`Print.cpp:1988–1994`). By-layer export additionally checks that the tool order is nonempty (`GCode.cpp:3654–3661`); sequential export rejects a missing initial tool before installing the validated print's extruders (`GCode.cpp:3633–3648`, `Print.cpp:773–781`). The reductions retain the upstream identity values even for an empty input. A nonempty writer set whose chamber targets are all zero is valid and publishes zero.

`GCode::placeholder_parser_process` uses the shared parser with optional per-template overrides (`GCode.cpp:5385–5388`). The relevant settings are `file_start_gcode` (`4230–4231`), `machine_start_gcode` (`4381`), `filament_start_gcode` (`4431,4461`), `before_layer_change_gcode` (`6227,6698`), `layer_change_gcode` (`6140–6142`), `filament_end_gcode` (`4867,4875`) and `machine_end_gcode` (`4878`). None locally overrides these two bindings. Template evaluation order is unchanged.

## Rust destination

In `ares-core/src/project_slice/gcode_emit`, compute both values once in the export orchestration from `schedule.tool_ordering.all_extruders()`, the same IDs used to construct the existing `motion::ExtruderRegistry`. Use the typed effective filament options and reuse the existing chamber reduction. Supply those values to the shared template builder in `placeholders.rs`, including `file_start.rs`, which emits before `EmitState` is constructed. This replaces the whole-config minimum and object-derived chamber maximum, temporary approximations of the shared upstream parser state. Remove duplicate overrides in `layer_gcode.rs` and `machine/start_config.rs`.

Included: those two integer bindings for every render using the shared builder. Deferred: writer membership and active-tool selection, hardware temperature commands and activation, parser evaluation, other placeholders and template-specific overrides, and changes to scheduling or render order.

## Public regression

Slice synthetic two-filament projects whose extrusion roles use only the second filament. Compare the Native, public API, and CLI G-code streams with the existing whole-file comparator. Both projects interpolate the two placeholders in the seven templates above; one gives the unused slot the lower vitrification and higher chamber temperature, and the other sets both chamber targets to zero. Verify that the reference actually extrudes the model only with the second filament.
