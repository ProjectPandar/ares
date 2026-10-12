# OuterOnly contour/mask: finite GREEN

SPEC a68680255; fixtures/tests/Native RED `9c589662e` was committed and
reported to inbox before production changes. Tests, fixtures, assertions and
comparator have no diff from that RED commit.

## Source-owned implementation

Private `brims/outer.rs` rewrites Brim.cpp:512-570: contour-only Round outer
ring and line527 width re-quantization; reversed-hole no-inner exclusion with
negative-spacing default Miter3 inset; actual source ExPolygon objectIsland
Round offset. `make_brim:903-904` owns spacing as no_brim_offset.
BrimPlan dispatch reuses the existing bed subtraction, own-object contact and
shared path emitter. InnerOnly, combined and ears implementations are unchanged.

Only three production files change: outer.rs51 lines, brims.rs333 lines and
gcode_emit/brim.rs194 lines. No core Native/IO/platform dependency, new public
API, option, crate, fallback, geometry-kernel or template/profile behavior.

## Finite whole-output acceptance

GREEN root: `/tmp/worker3-outer-only-brim-green-j9a5eu1x`.
Driver: `/tmp/worker3-outer-only-brim-green-driver.py`.
Original control-set-v2, jobs2/threads2, load<30 and each mode's fresh cache/root.
Same 47-test filter as RED, all features, unchanged ordered generator-only
comparison of Native against validated API and real CLI.

Seeded **47/47 PASS**, exit0, 31.258s; official **47/47 PASS**, exit0, 26.567s.
The eight REDs become whole-byte GREEN. Native/Ares movement witnesses are
identical on all eleven new inputs in both modes: no hole-side or detached
Brim extrusion; active exterior paths remain. Clipped-contact retains the
Native226 movements and removes the previously observed52 detached movements.

`raw-control-byte-preservation.json` matches input SHA and raw Ares SHA across
RED/GREEN. It verifies **138 unchanged raw streams** for existing combined,
inner, N/O/E, bed and new no-hole/inactive controls. Eight new active inputs
per mode intentionally change; no existing control output changes. Each pair
records input identity, both paths and hashes. Native timestamp metadata is
not used to waive a geometric or whole-output difference.

## Engineering and remaining gates

Engineering root: `/tmp/worker3-outer-only-brim-engineering-zi2bl7lq`.
Strict workspace/all-targets/all-features Clippy -D warnings, rustfmt check and
ares-core/ares-wasm all-features wasm32 check pass on this implementation.
These checks precede the implementation commit; no Rust changes follow them.

Committed-head directed dual-mode option coverage, code review, workspace and
full are separate gates. Workspace/full require explicit coordinator allocation;
there is no automatic watcher. Final exact-hash approval is required to push.
