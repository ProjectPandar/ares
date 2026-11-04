# G4 sequence-count slice

Status: bounded GREEN, with one Native nondeterminism observation recorded below.

## Source boundary and retained scope

The admitted slice is `PrintConfig.cpp:1115-1123`, `ParameterUtils.cpp:6-29`, `GCode/ToolOrdering.cpp:1320-1362` and first-layer `498-513`, with plate admission from `Format/bbs_3mf.cpp:4428-4435`. The Rust consumers already existed; the key-local parity harness now proves count0/1/2, empty-vector and reverse-priority controls with actual extrusion roles/order and unchanged whole-output comparison.

Production repair is type-owned footer nil semantics from `Config.hpp:837,1015,1344,1878` and `GCode.cpp:5632-5633`, in `ares-core/src/options/config_types/{semantic,vector,opaque}.rs` and `config_export/{value,writer}.rs` / `value/sequence.rs`. Empty/all-nil numeric/bool vectors are nil, zero/false remain nonnil, canonical nil cannot be resurrected by raw fallback, and empty String/Point/Group footer tokens remain visible. No key exception, ToolOrdering, support/infill/travel, shared settings-stack, master preference or Manual-map production edits. Deferred G4 keys, malformed/unbounded/by-object sequences and load-settings/PCP stay outside.

Fixture admission failures and the rejected 211/233 PLA temperature receipt remain historical, not production RED. The accepted fixture uses 211/223 and the observer preserves inherited TYPE roles across layers. Source-backed zero JSON expectation correction and the sole old empty-Floats nil expectation correction are explicit; the latter is folded into production GREEN, not a surviving follow-up assertion weakening.

## Native nondeterminism observation

`option_coverage::normal_support_output::interface_fan::support_interface_fan_preserves_complete_output_and_role_speed`,
official `closed-layers`, failed once in a full run and is classified as Native nondeterminism, not a G4 regression. This is not a blanket flaky allowance; the original failure remains untouched.

All nine serialized ZIP members match across all eight observations. Ares raw SHA is identical 8/8: `96913956f36359125c7a264c4ced339e3295388728229ead6d7ed8dd0642f590`. Diagnostic generated-banner-only grouping of the Native streams is seven identical vs one outlier. This offline grouping changes neither saved bytes nor the comparator.

Outlier first difference line850 is `G1 X113.388 Y113.135 F9000` vs the seven passing Native streams' `G1 X113.244 Y113.297 F9000`. Nineteen differing lines exchange two Internal solid infill path blocks, not support interface extrusion. Configuration is one logical filament, Auto For Flush, sequence count0/vector[0]; no Manual map or new sequence harness is selected. Shared vector/footer serialization is traversed, but all eight Ares outputs including footer are identical and no G4 diff modifies infill/travel.

Candidate upstream investigation boundary: `GCode.cpp:6150-6172` (`extrude_infill`, collection chaining) and `ShortestPath.cpp:1061-1069` (entity reordering). Exact nondeterministic origin/tie-break is **not proved**; merely finding support `parallel_for` is not evidence that it caused this internal-solid reorder. No production change, comparator tolerance, output stripping or new golden was used.
