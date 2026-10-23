# Processor reader numeric-word boundary INPUT pair

Approved SPEC f487181a1aad675fcbc7e37e76c1a18ba0a7bf8d:
`docs/architecture/2026-10-23-processor-reader-numeric-word-boundary-spec.md`.
Upstream8500fcd GCodeReader.cpp29–84 requires numeric endptr at word end,
else skips the entire word;hpp183–196 defines delimiters. Processor.cpp3813–3822
forwards only present axes/feedrate. Future GREEN must port literal tokenizer,
not regex or witness special-case;preserve fast_float rejection of leading `+`.

Exact already-qualified bytes copied without regeneration. Origin existing
`tests/parity/fan-mixed-timing/control.3mf`,SHA
b8bc9d8ab8db4eeaa21da61e759c64f230ddf4da800997424faf10fc53cfd298.
Official-exported stock-origin Creality Ender-3 0.4 nozzle /0.20mm Standard
@Creality Ender3 0.4 /Creality Generic PLA;new GUI-reachable edited inputs,
not new GUI-export transactions or unchanged factory presets.
Geometry/config/ZIP payloads unchanged except Before layer change G-code:
retain original header/comments/G92E0,append common `G1 X100 Y100 F6000`,then
control `G1 X101 Y100 F6000` vs target `G1 X101Y100F6000`,newline-terminated.
Only pair delta is whitespace in this GUI field. Both fan delay/kickstart0;
no startupfan/temperature/E/geometry/brim/tower edits or synthetic preset JSON.

| Input | SHA256 |
|---|---|
|control.3mf|3a9c71fea578cca734cc03236cfbfc3cc1c664ef33b7b4d275a0d7ef76c3a7d9|
|target.3mf|14a8c16229dad5db584b4b15573db49c37aadc2c0b804836047b50acddf6cb54|

Two ordinary direct unchanged native_stream_case checks `(false,|_,_|{})`:
Native2 FIRST successful stable,then validated API/actual CLI success,unmasked
complete ordered equality and raw API==CLI. Expected RED is controlPASS and
actual target equalityFAIL,not rejection/runtime failure. Keep M73/time/fan/
E/F/wipe/start/end/config/footer;only original generator/date normalization.
No output golden/mask/tolerance/inversion/ignore/private-state observer.
Target comparison panic may precede wrapper-tail/raw equality assertion;
independent raw-file equality is not a claim that unreachable assertions ran.
No production fix is included;the compact spelling remains accepted at public
project admission,while parity follows upstream analysis-reader semantics.
