# Seeded PlaceholderParser random parity

Owning source: OrcaSlicer8500fcdcc `PlaceholderParser.hpp::ContextData`,
`PlaceholderParser.cpp:647-657,1708-1716,2319-2320`, `GCode.cpp:2707`. Rust
destination: `slicer_context/rng.rs` and existing `project_slice/gcode_emit`
template/config boundaries. Follow [ARD](../../architecture/ard-template-rng-seeded.md).

Replace fixed5489 LCG scaffold with source-equivalent MT19937 and selected
libstdc++ distributions. Feed an independent per-emission template stream from
existing explicit SlicerContext seed; do not consume/reseed the fuzzy stream.
Include integer/real type dispatch, shared template draws and job isolation;
defer unrelated parser/global-variable features and other matrix failures.

Extend the separately labelled seed-only native reference for the same template
seed, with official untouched. Six frozen stock inputs: nhVbMr,bKs3EZ,1jESfd,
TY2alA,HoYp3R,1Bg0jk. Native0 repeats, native1 and full public Rust RED/GREEN
for all six must be recorded. Exercise real/mixed args and distribution edges
through an additional real-template E2E probe. No production code before RED.

Exit: four gates plus full official/seeded controls with no new FAIL and
full generator-only ordered bytes.
