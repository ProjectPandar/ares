# Beta empty-plate CLI channels

Every `file:line` is at OrcaSlicer v2.5.0-beta `345efec64` unless it names 2.4.2 (`8500fcdcc`). Against the beta oracle, three empty-plate parity tests still fail after the model-read period fix. This slice ports the one gap that is Ares behaviour, retargets the one whose expectation is 2.4.2 behaviour, and hands the third to its owner.

## (a) The `-50` reason text: ported
- **Upstream.** When `Model::update_print_volume_state` (`Model.cpp:839-852`) counts no printable instance on the selected plate, the CLI records `CLI_NO_SUITABLE_OBJECTS` (-50, `Utils.hpp:71`) with `cli_errors[CLI_NO_SUITABLE_OBJECTS]` and exits (`OrcaSlicer.cpp:6562-6573`). At the beta that reason reads "The plate has nothing to slice: it is empty, or no object on it is both printable and fully inside the print volume." (`OrcaSlicer.cpp:220`). The 2.4.2 text, "One of the plate is empty or has no object fully inside it. …" (2.4.2 `OrcaSlicer.cpp:137`), is gone.
- **Ares destinations.**
  - `project_slice/state.rs:76-83`, the core's `NoSuitableObjects` text. Its citations (`OrcaSlicer.cpp:5710-5724`, `Model.cpp:692-702`) are 2.4.2 lines and become `:6562-6573` and `:839-852`.
  - `ares-cli/src/failure.rs:166-182`, the CLI's result.json `error_string`. Its 2.4.2 citations move to the beta, among them `record_exit_reson` (`OrcaSlicer.cpp:535`).
  - The beta table matches every other reason in `failure.rs`.
- **Deferred:** the beta's arrange-without-rotations reason, also printed to stderr (`OrcaSlicer.cpp:6564-6571`). The Ares CLI has no arrange.
- **RED.** `OUTSIDE` in `empty_selected_plate_controls.rs` and the four expectations in `plateless_instances_hit_no_suitable_objects_gate.rs` take the beta text. The core test fails without Native; `empty_selected_plate_control_nonempty_outside_volume_keeps_later_error` passes its Native leg and then fails on the Ares leg (`:117`).

## (b) Plate 2's stream: out of scope here
`empty_selected_plate_control_populated_plate_two_preserves_native_stream` (`:59`) is a G-code comparison.
- On this branch's base, the first difference is Klipper's `MINIMUM_CRUISE_RATIO`, which main `44fb3e691` ported.
- On main, the first difference is an early `M73 P5 R4` (line 90): M73 placement (G-code processor time estimate), out of scope here. The test is expected to move when the time-estimate port lands.

No change here.

## (c) The circle plate-origin case: retargeted test
`plate_origin_cli::circle_plate_origin_matches_native_empty_selected_plate` (`:40`) expects 2.4.2's `-50` rejection (exit 206) of plate 2.
- **What the beta does.**
  - `update_print_volume_state` counts the instance inside: its log line is "print_volume {77, -55, 0} to {187, 55, 115}, got 1 printable istances".
  - The plate is sliced and `plate_2.gcode` written.
  - `gcode_check_result` then reports G-code in an unprintable area, and the CLI records `CLI_GCODE_PATH_IN_UNPRINTABLE_AREA` (-102, `Utils.hpp:94`; `OrcaSlicer.cpp:7194-7204`, reason `:242`), exiting with 154.
- **Ares on main** already returns -102 with exit 154 and the same stdout, and writes the plate's G-code. That G-code differs from Native's only in `M73` placement (G-code processor time estimate), out of scope here.
- **Retarget.** A separate test commit expects that channel: exit 154, return code -102, plate 2, and the plate G-code written by both. Ares's exit, stdout, stderr and result.json still compare with Native's. The G-code bytes are not compared; that would test `M73` placement (G-code processor time estimate), out of scope here.
- **Unknown.** Why 2.4.2 counted no instance is not localized. The model-side state code is unchanged apart from logging and a repaired-mesh branch, and there is no 2.4.2 official binary to compare against. Ares already matches the beta, so nothing depends on it.
- The retarget passes as soon as it lands: it records behaviour Ares already has, so it has no RED.

## The 2.4.2 documents
`empty-selected-plate-model-read-error.md` and `native-artifact-cleanup-plate-origin-spec.md` cite `Model.cpp:354-355` and the text without the period. Both pin 2.4.2 `8500fcdcc`, where that is the throw and the text has no period, so their citations are correct for their baseline. Each gains a short "Beta" note instead: at `345efec64` the throw is `Model.cpp:500-501` and ends with a period, which Ares follows.

## Verification
The 7 empty-plate parity tests and the core gate test, each reported. Expected:
- (a)'s control passes, along with the four model-read cases.
- (c) passes after the retarget.
- (b) stays on `M73` placement (G-code processor time estimate), out of scope here.
