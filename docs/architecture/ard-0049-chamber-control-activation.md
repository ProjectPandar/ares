# ARD-0049: Exercise upstream chamber controls, not support geometry

`PrintConfig.cpp:3892,6599-6643` owns the printer capability, filament
activation and two 0..1500 temperature vectors. `GCode.cpp::_do_export`
(3008-3013,3036-3038,3130-3134,3498-3499) computes activation/maximum,
sets placeholders, suppresses automatic startup if the rendered template
already controls the chamber, and shuts down enabled positive targets.
`GCodeWriter.cpp::set_chamber_temperature` (192-212) brackets the waiting
M191 with auxiliary fan commands when enabled.

The Rust rewrite destination is the existing
`ares-core::project_slice::gcode_emit::machine::temperature` and placeholder
projection, validated through CLI option activation and public full-output
parity tests. `support_chamber_temp_control` is not support-material logic
and is not a runtime gate in the cited export path. Do not infer gates
from option names or add a new Ares-owned temperature pipeline.

Use the Ender-3 cube fixture with active chamber settings and explicit
template markers. Verify injected values and dependencies, exact ordered
temperature/fan startup and shutdown commands, and minimum-temperature
placeholder output. Numeric endpoints remain upstream 0/1500; these tests
only generate files and never send commands to a printer. For minimum
temperature probes set target 1500, keeping minimum <= target.

Include all values of both chamber Boolean options, both numeric domains,
and the auxiliary-fan Boolean, plus M141/M191 startup suppression. Defer
multi-filament OR/max aggregation and other firmware/template interactions.
The harness remains a temporary validation shell around upstream concepts;
full byte comparison and retained artifacts cannot be weakened.
