# Signed FanMover constructor input pair

SPEC: `docs/architecture/2026-10-22-fan-speedup-sign-construction-spec.md`.
Source owner: pinned8500fcd `GCode.cpp:3732-3740,3830-3838` constructs
`FanMover.hpp:63-70` with abs(float(t)) **and original t>0 flag**.
Other FanMover behavior is deferred; no production change in RED.

Origin `modifier-bottom-shell/classic-four.3mf`, customized stored Bambu
Standard-Mine profile, not pristine factory. Both inputs share fan_min_speed20,
fan_max_speed20,fan_speedup_overhangs0. All other source settings,geometry,
placement,modifier bottom-shell4 and non-settings ZIP bytes are exact.
Target versus control differs only in fan_speedup_time. No embedded G-code.

| Input | fan_speedup_time | SHA256 |
|---|---|---|
|../fan-layer-tag/control.3mf (reused)|0|17896c6fdf4d5d570c604843fafd049cbc1ac6cde86a21c07dd20ebfdd8a13b2|
|target.3mf|−3|19507db8da1597d32a348f299edaa8d24a5e16aa203b6f3150da6ca308dc049d|

Exact accepted sealed discovery bytes; the zero control is reused from
fan-layer-tag/control.3mf without a duplicate in this directory.
Two direct unchanged native_stream_case::check calls, (false,|_,_|{}), first
replay Native twice, then public validated API/actual CLI ordered full equality.
Only original generator/date normalization; no fan/M73/time/E/F/footer mask,
golden outputs,private observer counts,inverted assertion,ignore or fallback.
SHA records freeze inputs only; live Native outputs remain the reference.

Expected formal RED: control full PASS,target genuine whole-stream FAIL in each
mode. All Native/API/CLI runtime statuses must succeed and Native repeat stably.
Target API comparison can panic before wrapper-tail/raw equality assertion;
independent raw equality audit must not claim an unexecuted assertion ran.
