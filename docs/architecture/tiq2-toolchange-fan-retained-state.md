# TiQ2 tool-change fan: retained CoolingBuffer request speed

## Source ownership and decision

Port the distinction between CoolingBuffer's interpolated baseline
`m_fan_speed` and retained request speed `m_current_fan_speed`. They are not
synonyms for the last physical fan command.

Upstream `libslic3r/GCode/CoolingBuffer.cpp:737-810` calculates the baseline
and updates both values only when that baseline changes. `:984-998` sets
m_current_fan_speed when emitting an active overhang, bridge, support or
ironing request. `:1004-1005` emits baseline after a request ends WITHOUT
replacing m_current_fan_speed. `:890-898,1000-1002` restores the retained
value for FORCE_RESUME when no active role request takes precedence.
`GCode.cpp:7963-7972` appends that marker after custom change_filament_gcode;
the separate tower integration also appends it at`:980`.

Rust destination: project_slice/gcode_emit/cooling.rs and
cooling/role_fans.rs. The existing `physical_part_speed` scaffold stores
baseline END emissions as well as role emissions, losing the distinct
upstream retained request state. Replace that interpretation with the
source-owned retained state. Keep fan interpolation, PWM conversion,
minimum clamp and emission position unchanged; no Type2 condition or
fixture-specific workaround.

## Native confirmation

Gdb on the existing exact seeded Native binary confirms layer3 has total
layer_time13.8802 and baseline51%. The writer sequence includes baseline51,
role80, baseline51, then FORCE_RESUME80. At the FORCE_RESUME call site,
the fields are explicitly baseline51, retained80, emitted80. The appended
output ends with the custom tool-change M109 S265 T1; the restored fan is
inserted before T1. Native80% gives M106 S204; Ares incorrectly restores
baseline51%, giving M106 S130.

Disassembly confirms the force branch loads the retained field at offset
0xa4 while the baseline branch loads offset0x50. The debugger's complete
output equals the exact seeded oracle with generated-by-only normalization:
816581 bytes, SHA256
8e417e46deaef9e48f9390cf9ae0412b7977722b7722fdfcf56c2eb7cd467001.
No Native rebuild or generated output commit is needed.

## Included and deferred behavior

Included: retained state on baseline changes and active-role emissions;
baseline END emission without overwriting retained state; forced restore
with existing role precedence. Cover overhang plus existing bridge,
support-interface and ironing branches without adding new options.

Deferred: other fan interpolation changes, tool-change scheduling, speed
selection, tower generation and independent skirt hull discrepancies.
Complete TiQ2 equality remains the eventual goal, not assumed from fixing
the first fan difference. The observed full comparison agrees through1822
lines and fails at1823; no differences may be filtered.

## RED/GREEN acceptance (after spec approval)

Write a runtime-OrcaRunner integration RED using the committed TiQ2 input,
asserting the native-generated complete tool-change fan transition against
Ares, before changing production. No Native stream fixture is committed.
Run the complete generated-by-only comparator to expose any next residual.
Add source-state controls before implementation for role-end→force-resume,
changed-baseline resetting retained state, and active-role precedence.
Then implement only the cited state distinction, rerun affected seeded
controls and required workspace/clippy/fmt/wasm gates. Existing scaffolding
is replaced, not preserved as a legacy fallback.
