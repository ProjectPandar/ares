# Motion attributes fixtures

Per-vertex `MoveVertex::{pressure_advance, acceleration, jerk}` f32 bits for
`crates/ares-core/tests/processed_motion_attributes.rs`, as `count,pa,acc,jerk`
hex runs in `GCodeProcessorResult::moves` order.

Produced by the initialized-native oracle `tools/chain-probe/procgcode-dynamics.nix`
(OrcaSlicer 8500fcdcc + procgcode + stable-options + dynamics patches):

- binary sha256 `d14230b50ea082ca3bfc91fe9f9e7cd122b1b5bde8ce53fa67575a0873cf6f2c`
- `procgcode-dynamics.patch` sha256 `837ce7460949c7c3eecf2faa3bdf788c266eec32dd5a0402d4a8c58f21b4759f`

Each `.rle` is the oracle's `native.rle` for the Ares-sliced G-code of the
case (producer line renamed to `OrcaSlicer 2.4.2`), after the test asserted
that every vertex identity (`gcode_id`, type, `internal_only`) matches.
Regenerate: `ARES_PROCGCODE_DYNAMICS_BIN=<out>/bin/orca-slicer cargo nextest
run -p ares-core --test processed_motion_attributes`, then copy
`target/motion-attributes/<case>/native.rle`.

The oracle initializes `m_pressure_advance` to 0 (upstream leaves it
uninitialized; UB-adjacent, not compared against official) and applies the
slicing-path flavor gate to `machine_max_acceleration_travel`.
