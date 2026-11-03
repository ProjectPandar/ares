# Spec: Prusa XL0.25 terminal wipe segment

Source OrcaSlicer8500fcdccaa10b5099ac20d252af3a7c560046f1:
`GCode.cpp:440-488 Wipe::wipe` iterates every line returned by
`Polyline.cpp:22-31`. `GCodeWriter.cpp:933-952` emits XY even for zero dE.
Rust destination is only `motion/travel/wipe_path.rs::wipe_moves`: replace its
zero-length segment filter with the upstream line representation, preserving
`Polyline.hpp:89-95` append-junction semantics and source denominator floor.
Existing Ares stage is a compatibility shell, not a new wipe implementation.

Include zero-length segment preservation, single append-junction dedup and
short-path EPSILON denominator. Defer clipping, writer/comment policy, other
retraction behavior, wipe tower, timing and the six random-coordinate cases.

Target frozen case-HJumeY, Prusa XL0.25. On31996f48 public replay is RED at
line3318: missing `G1 X175.829 Y183.23`. Artifact `${TMP_ROOT}/ares-prusa-wipe-red` and
immutable copies `${TMP_ROOT}/ares-prusa-wipe-replay`. Debugger confirms clipped
points A=(-4171147,2229684),B=(-4171147,3229684),B with configured distance1e6.

Require full target and fresh original CPU0 reference equality, known68+823
controls, re3D3 and extra-overhang stock2/step2, workspace Nextest, fmt, default
Clippy and actual core+adapter WASM build. Coordinator approval, latest-main
rebase/revalidation and ff-only integration; no comparison waiver.
