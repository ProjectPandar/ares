# FanMover verbose text INPUT pair

Approved SPEC2c7e9a76cbf286d7ab7b4cabef70bbb80aa8fa4d:
`docs/architecture/2026-10-23-fanmover-verbose-text-spec.md`.
Pinned8500fcd GCode.cpp2038 selects writer comment policy;
GCodeWriter.cpp1095–1130 adds enable/disable fan comments;
FanMover.cpp231–236 delegates new fan commands there,398–418 kickstarts.
FanMover.cpp75–87 replaces only numeric spans,preserving raw separator/comment;
splitting124–212 consumes it. One whole-output RED covers both text contracts.

Exact qualified INPUT bytes copied without regeneration. Original stock-origin
already official-exported `fan-mixed-timing/control.3mf` SHA
b8bc9d8ab8db4eeaa21da61e759c64f230ddf4da800997424faf10fc53cfd298.
Creality Ender-3 0.4nozzle /0.20mm Standard @Creality Ender3 0.4 /GenericPLA;
32-sided12×6mm cylinder,30×0.2mm layers,Classic2walls,15%crosshatch,3top/bottom.
Common GUI Verbose G-codeON,fanlead0;pair only fan kick-start0→0.5seconds.
All geometry/otherconfig/non-settings ZIPpayloads preserved. GUI-reachable edits,
not newly transacted GUI exports or unchanged factory projects. No synthetic
presets/customprobeGcode/flavor/PWM-floor/temperature/tower/adhesion edits.

| Input | SHA256 |
|---|---|
|control.3mf|ef1951bd9f85d84ebfaf569b5a0ac4cf729e22e3c0c0d788ea5ad71b54a6e632|
|target.3mf|42123da0f4fd38bf64a1ea498e9985640816f952b16e170fbc0a63817aa2ce7f|

Two ordinary DIRECT unchanged native_stream_case checks `(false,|_,_|{})`:
Native2 FIRST successful/stable,then validated public API/actualCLI success,
complete unmasked ordered equality and raw API==CLI. No Native expected G-code
output committed,no private-state/callcount/hunk-count constants in tests.
The target's whole equality requirement covers ALL observed differences:
generated fan comments AND split G1 separator spaces,not just first mismatch.
No comment/M73/time/fan/E/F/wipe/start/end/config/footer masks/tolerance/golden/
inversion/ignore/rejection-as-success. Original generator/date normalization only.
Target panic can precede wrapper-tail/raw-equality assertion;independent raw
file audit is not a claim unreachable assertions executed. No production fix.
Flavor/PWM-floor/timing/parser/otherowner behavior remains deferred.
