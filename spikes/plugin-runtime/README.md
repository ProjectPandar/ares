# Plugin runtime experiment

This is a standalone benchmark, not a slicer integration. `api` models the
pure Rust hook and option types, `samples` implements the two Rust recipes,
and `host` projects the core types into the serialized ABI. The JS specimen
uses the same byte schema and option values. Rust and JS guests share the
wire layout; direct and guest JS use the same engine wrapper. The fixtures
are synthetic and no slicing or G-code emission paths are changed.

Use the repository Rust toolchain, Bun, Chrome, Clang with a WebAssembly
backend, and the `wasm-bindgen` CLI matching the root lockfile. From the
repository root:

```sh
rustup target add wasm32-unknown-unknown wasm32-wasip1 \
  x86_64-pc-windows-msvc aarch64-apple-darwin
bun ci --cwd web
export PLUGIN_SPIKE_OUT=/absolute/artifact-directory
export CARGO_TARGET_DIR=/absolute/isolated-target-directory
export ARES_E2E_CHROME=/absolute/path/to/chrome
export CC_wasm32_unknown_unknown=clang
bash spikes/plugin-runtime/build.sh
bash spikes/plugin-runtime/web-build.sh
bash spikes/plugin-runtime/native.sh
bun spikes/plugin-runtime/browser.mjs
python3 -I spikes/plugin-runtime/summarize.py "$PLUGIN_SPIKE_OUT"
```

Run the build and measurement commands under the same processor affinity,
one at a time. The browser command serves on localhost port 4291 unless
`PLUGIN_SPIKE_PORT` is set. `PLUGIN_SPIKE_LAYERS=1` and `PLUGIN_SPIKE_TRIALS=1`
allow a smoke run; do not use those settings to collect comparison numbers.
For the browser runtime component experiment, also install
`@bytecodealliance/jco@1.42.0` with `bun add --exact` in a separate directory
and set `PLUGIN_SPIKE_JCO_DIR` to that directory before running Chrome.

Hosted timings start with the serialized layer already available. The native
Rust baseline times only the typed hook; the Chrome Rust baseline includes
its Wasm-bindgen calls and byte projection. The native driver separately
checks that the hosted runtime can be invoked through the typed trait.

This prototype does not admit untrusted project files or load a real 3MF. It
has no production-level validation, import admission, failure isolation, or
persistent plugin identity. The virtual file fixtures only implement the
WASI operations used by their `std::fs` call sites. The browser adapter
snapshots and commits the fixture data in IndexedDB outside a timed hook.
