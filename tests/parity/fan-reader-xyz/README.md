# FanMover private-reader XYZ mode input pair

Approved SPEC:docs/architecture/2026-10-23-fanmover-reader-xyz-mode-proposal.md.
Pinned8500fcd FanMover.cpp291-304 calls GCodeReader.hpp43-45 raw-axis deltas;
hpp129-135 calls callback before cpp110-123 raw-axis position assignment.
No upstream G90/G91 tracker in this private reader;actual firmware/template
state is outside this slice.

Exact Native-qualified bytes copied without regeneration. Origin existing
fan-layer-tag/control.3mf SHA17896c6fdf4d5d570c604843fafd049cbc1ac6cde86a21c07dd20ebfdd8a13b2.
Both share GUI before_layer_change_gcode with balanced plain commands:
G91 / G1 X1 F6000 / G1 X-1 F6000 / G90,each newline terminated. Common
min/maxfan20,OnlyOverhangs0;allgeometry/modifier4/placement/nonsettingsZIP
unchanged. Existing customized Standard-Mine Bambu,not pristine factory.
Only pair difference is fan_speedup_time0→3. No embedded G-code outputs.

| Input | fan_speedup_time | SHA256 |
|---|---|---|
|control.3mf|0|06a72c88130a9c73bf6a4ef04ec54d7e2e22a85acd4220283b660513333bb87e|
|target.3mf|3|ed8a8a1c5fbed8a032dfbe222d2a1f4bbaa06ac79607fb0b5283bce25d071d04|

Two direct unchanged shared native_stream_case checks,(false,|_,_|{}).
Native twice before public validated API/actual CLI;complete ordered equality
with original generator/date normalization. No fan/M73/time/E/F/wipe/footer
mask,tolerance,golden output,private observer assertions,ignore or inversion.
Expected RED1controlPASS1targetwholeFAIL per mode,notruntime/refusal failure.
Target API panic may precede wrapper-tail;raw audits are not executed assertions.
