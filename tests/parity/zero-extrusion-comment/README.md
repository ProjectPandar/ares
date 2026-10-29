# Ordinary zero-extrusion XY role-comment INPUT pair

Approved SPEC7018a6d5f778e28825925311a503bc38cadfe9a9:
`docs/architecture/2026-10-23-zero-extrusion-xy-role-comment-spec.md`.
Pinned8500fcd GCodeWriter.cpp933–956::extrude_to_xy omits E at zero dE but still
emits role comments under full_gcode_comment. GCode.cpp2038/7031–7036 supplies
verbose policy and role description;PrintConfig1366–1375 allows top flow0.
This RED changes no production flow selection/state or emitter code.

Exact Native-qualified INPUT bytes copied without regeneration. Stock-origin
already official-exported fan-mixed-timing/control.3mf SHA
b8bc9d8ab8db4eeaa21da61e759c64f230ddf4da800997424faf10fc53cfd298.
Ender3 0.4nozzle /0.20mm Standard @Creality Ender3 0.4 /GenericPLA cylinder
12×6mm32sides,30×0.2layers,Classic2walls/15%crosshatch/3top-bottom.
Common GUI VerboseON;only pairkey top_solid_infill_flow_ratio1→0 (GUImin0).
Both fanlead/kick0,FanMover inactive. All geometry/otherconfig/non-settingsZIP
payloads unchanged;no customprobeGcode/presetJSON/model/flavor substitution.
GUI-reachable stock-origin edits,not new GUI-export transactions/factory claim.

| INPUT | SHA256 |
|---|---|
|control.3mf|ef1951bd9f85d84ebfaf569b5a0ac4cf729e22e3c0c0d788ea5ad71b54a6e632|
|target.3mf|90ca1740ea4fece67054f2fdeb4202ed31f7fd43b521f9976aacf9647550631b|

Two direct ordinary unchanged native_stream_case::check `(false,|_,_|{})`
calls,grouped registration with no squeezing/include. Native2FIRST success/
stability then validated API/actualCLI success,unmasked complete ordered
comparison/rawAPI==CLI. Whole target equality covers every missing rolecomment;
no Nativeoutputs/hunkcount constants/private observers/goldens committed.
No comment/M73/time/fan/E/F/wipe/start/end/config/footer mask,tolerance,
inversion/ignore/rejection-as-success;originalgenerator/date normalization only.
Target first mismatch panic may precede rawEqASSERT/wrapper-tail;independent
raw-file equality is not a claim unreachable assertions ran. No GREEN yet.
W2 flow call/helper explicitly untouched. All3W3chains await own singlewindow.
