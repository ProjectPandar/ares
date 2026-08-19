# Spec: KSR FDM Test V4 task226 processor command and first-layer timing semantics

## Observable contract

The generated G-code processor changes modal feedrate only when parsing a supported `G0`, `G1`, `G2`, or `G3` motion. An `F` word owned by another command, such as `G130`, cannot affect the next motion block. First-layer estimated time ends at the first emitted `; CHANGE_LAYER` marker after the initial progress marker.

This keeps timing derived from generated G-code and machine commands. No fixture name, reference G-code, or expected duration is inspected by production code.

## Upstream boundary

This slice ports command ownership and first-layer attribution from OrcaSlicer 2.4.2 `src/libslic3r/GCode/GCodeProcessor.cpp`: the command dispatch table routes supported motion commands to `process_G1`/`process_G2_G3`, while `TimeMachine::calculate_time` attributes blocks with `layer_id == 1` to `first_layer_time` at lines 478-479.

Included: supported-motion feedrate updates, unchanged feedrate for unsupported commands, and first layer ending at the first layer transition. Deferred: remaining planner, delay, geometry, M73, and normalized G-code differences.
