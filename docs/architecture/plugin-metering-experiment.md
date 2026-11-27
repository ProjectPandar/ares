# Portable fuel for hosted plugins

## Boundary

OrcaSlicer v2.5.0 (`40ecc66f7`) installs its Python dispatcher in
`src/slic3r/plugin/PluginManager.cpp:137`; the implementation in
`src/slic3r/plugin/PluginHooks.cpp:72–140` runs outside `libslic3r`. The core calls
that dispatcher through `Print::run_pipeline_hook` in
`src/libslic3r/Print.hpp:1491–1496`. Guest bytecode limits replace host behavior;
there is no upstream WebAssembly budget to port. This experiment stays in
`spikes/plugin-runtime/host`, with a thin binding in `ares-wasm`. It does not
modify slicing, project loading, or the typed hook contract.

## Included behavior

- Try a shared `wasmparser` and `wasm-encoder` pass on the same core modules
  used by the Rust and QuickJS fixtures. Compare with `wasm-instrument`'s
  metering API and supported instruction set. Keep runtime-specific fuel off
  for the shared pass.
- Debit the static operator count for each straight-line segment, before
  running it; split at control transfers and their targets, including function
  entries and loop headers. A segment that cannot afford its debit traps via
  `unreachable`. Structured delimiters carry no cost. Only the host can reset
  the added fuel global; the guest cannot address it through its original code.
- Bind the budget to one host operation (the input reservation and the hook
  or source update together). Rewrite the guest-defined linear memory maximum
  to the smaller of the original maximum and the host cap, or set the cap if
  there was no maximum. Reject an initial size above the cap.
- Run the transform at module load, including module replacement. Updating the
  JavaScript source in an existing QuickJS guest uses the already transformed
  runner. Measure that separately from loading a fresh runner.

## Deferred behavior and failure modes

Reject invalid modules, collisions with the added export, and module shapes
outside the exercised subset (such as start functions, imported, shared, or
64-bit memory, and control instructions without a segment rule). No fallback
runs the original bytes when rewriting fails. One core module and one linear
memory are in scope; components, Boa, checked host import costs, stack limits,
weighted dynamic operations, compilation budgets, and admitting arbitrary
project files are not. An identical fuel trap on the test workload is not a
claim of equivalent engine stack limits or production isolation.

## Verification

Through the Wasmtime CLI harness and the Chrome worker, load the same counter
fixture with 1,000 units, check that both engines trap at the same nonzero
increment, and check that both refuse growth past the rewritten two-page cap
while permitting growth up to it. Exercise replacement using the existing
alternate Rust module and QuickJS script. Record release `ares-wasm` binary
size with and without the binding and measure per-layer hook time, load, and
reload on both engines for each recipe and language. Compare every edit and
G-code SHA-256 to the uninstrumented measurements; report timing separately
from correctness.
