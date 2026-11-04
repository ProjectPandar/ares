# ARD: preserve writer-used extruder membership in exhaust-fan reductions

## Context

Discovery is accepted on main 6f6ef10f5; a docs-first
SPEC is followed by a success-required public RED. This records non-negotiable
architecture. [SPEC](exhaust-used-extruder-spec.md) owns the
matrix, failure inventory, validation and included/deferred behavior.

## Decision

Port upstream8500fcdcc `libslic3r/GCode.cpp:3181–3196,3501–3516` reductions over
`m_writer.extruders()` into existing private Rust
`ares-core/project_slice/gcode_emit/machine/exhaust_fan.rs`, preserving existing
start/end placement in `machine.rs`. This is an upstream rewrite slice, never
an Ares-owned new fan pipeline. The current full-vector reducer is incomplete
temporary scaffold and must not remain a legacy fallback.

Use the existing authoritative writer-used ID owner: source provenance is
`GCode.cpp:2760,2791,5646–5648`, `GCodeWriter.cpp:63–80`,
`Print.cpp:522–547`. A separate list reconstructed from positive model moves
would lose support/custom-tool/wipe-tower participation and is forbidden as a
general production substitute. Private context handoff must be traced and
checked before GREEN; no registry redesign or print_order change is in scope.

Preserve capability/master/phase gates, source get_at indexing, eligible zero
speed, max/PWM conversion `GCodeWriter.cpp:1157–1171`, flavor and command order,
and full option serialization. Do not reject supported settings, add fallback,
hide fan commands or alter downstream processor behavior.

## Evidence and acceptance boundary

GUI controls exist in `Tab.cpp:4108–4123,4304–4312` and option definitions in
`PrintConfig.cpp:1886–1921`; machine capability is exposed by `Tab.cpp:4577`
and defined at `PrintConfig.cpp:3899–3904`. Only ordinary machine/filament settings distinguish
the four frozen input archives. No new Native constants/goldens/preparer.

Public success-required `native_stream_case::check((namespace, upstream), label,
project, sha, (false, callback))` is the agreed seam. Four labels+SHA witnesses,
unchanged shared parse-only activity helper and Native replay establish T0-only
model activity without movement/fan/layer constants. Complete ordered Native /
API / actual CLI comparisons and raw API==CLI remain the only acceptance; RED
target must genuinely fail while three controls pass. No expected-error PASS.

## Deferred and consequences

SPEC and tests are separate commits; all implementation waits for RED
and a separate GREEN. Defer nonzero-only/multiple-used/support/custom-tool runtime
witnesses, adjacent fan/temperature work, separate skirt/z-hop/print_order/
processor-M73/seam-occlusion work, full parity and platform runtime qualification.
Rust destination remains portable API logic, without filesystem/terminal/UI or
backend dependency. No public API expansion, unit tests, dormant consumer,
second registry, safety fallback or unrelated cleanup is justified by this slice.
