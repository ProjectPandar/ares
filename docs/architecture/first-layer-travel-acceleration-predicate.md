# First-layer travel acceleration predicate

## Upstream boundary

Port the first-layer acceleration predicate in OrcaSlicer v2.5.0
(`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`):

- `src/libslic3r/GCode.cpp:10519-10531::GCode::travel_to` initializes
  `acceleration_to_set` to zero and selects
  `initial_layer_travel_acceleration` only inside `on_first_layer()`.
- `src/libslic3r/GCode.hpp:991` defines that predicate as an active layer,
  `layer.id() == 0`, and `abs(layer.bottom_z()) < EPSILON`.
- `src/libslic3r/Layer.hpp:152` defines `bottom_z()` as
  `print_z - height`.
- `src/libslic3r/PrintObjectSlice.cpp:81-97` starts model-layer IDs at
  `raft_layers()`, so model-layer ID zero is not assumed when a raft exists.
- `src/libslic3r/Support/SupportCommon.cpp:1492-1540` assigns support-layer
  IDs from zero while grouping support by print Z. A support layer with ID zero
  can nevertheless have nonzero `bottom_z` when support begins above the bed.
- `GCode.cpp:10532-10558` is the non-first-layer arm and selects ordinary or
  short-travel acceleration, including at support layer ID zero if its bottom
  is not at the bed.

This is a source-cited rewrite slice of the upstream `travel_to` boundary, not
an Ares-specific acceleration policy.

## Rust destination and current discrepancy

The destination is `crates/ares-core/src/project_slice/gcode_emit/motion/state.rs::begin_path_travel`.
Replace the temporary `state.layer_index == 0` check for initial travel
acceleration. The state already carries
`state.on_first_layer` for the upstream layer-ID plus bottom-Z predicate. The
minimal fix is to use `state.on_first_layer` for this acceleration arm only.
The existing unsigned-local truncation and `rounded_acceleration` operation
remain unchanged and already match `GCode.cpp:10526-10528` for finite defined
values. No option, parser, public API, writer, or fallback changes are needed.

## Synthetic witness

The synthetic `tests/parity/first-layer-travel-acceleration-predicate/inset-roof-matched.3mf`
contains a stepped mesh with a roof inset from the base. It uses Marlin,
`default_acceleration=500`, `travel_acceleration=1400`, and
`initial_layer_travel_acceleration=650`. Support may be generated off the
build plate and uses a positive bottom gap. The first support is emitted at
print Z 2.4, with support layer ID zero and a positive `bottom_z`, so upstream
`on_first_layer()` is false. First-layer and corresponding ordinary extrusion
speeds and accelerations match; wipe and cooling slowdown are disabled in the
input to isolate travel selection. Progress remains enabled and compared. The current Rust branch emits repeated
`M204 S650` at that layer, while Native emits `M204 S1400`. The difference is
specific to the predicate: ordinary travel acceleration is 1400, initial-layer
travel acceleration is 650, and both are positive. Native/API/CLI whole-stream
comparison therefore fails before the fix.

The same mesh with support disabled and a zero-gap support variant were also
probed read-only. Only the elevated-support case is needed in the parity suite. No raft or Z-offset input produced a qualified difference: raft
object IDs start after raft layers and the raft schedule's first-layer state is
already explicit; a global Z offset changes machine Z but not the layer
`bottom_z` relationship. Those paths remain deferred rather than receiving an
unwitnessed branch.

## Included and deferred behavior

Included: replace the acceleration arm's layer-index-only condition with the
existing complete `on_first_layer` state, preserving the upstream positive
amount gate, unsigned truncation, rounding, and all non-first-layer behavior.
The behavior test observes the elevated support layer's ordinary travel
acceleration and compares the complete ordered Native/API/CLI stream.

Deferred: raft and Z-offset-specific behavior because the public probes did
not make this acceleration selection observably diverge; nullable acceleration
inheritance and other acceleration predicates; non-first-layer short-travel
role selection; writer flavor, cache, and machine-limit behavior; jerk
selection, which is already covered by the preceding slice.

## Test-first exit

Commit this specification first. Then add a behavior-named public
`native_stream_case::check` for the synthetic elevated-support input and run
it against unchanged production code; it must fail at the first `M204 S650`
versus Native `M204 S1400` difference while Native replay remains deterministic.
After observing the failure, replace only the acceleration condition, rerun in
both official and seeded Native environments, and require strict whole-stream
API/CLI parity. Also run `cargo fmt`, workspace all-targets clippy with
`-D warnings`, and the wasm32 core/adapter check. Do not commit Native output.
