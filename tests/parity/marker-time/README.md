# Marker Time native source-kernel fixtures

Approved spec: `docs/architecture/web-marker-time.md` (`75c08a84`). This is a
**native source-kernel oracle, not execution of the full native viewer**.
The diagnostic executes the real `GCodeProcessor` and
`libvgcode::convert(GCodeProcessorResult, ...)` from `LibVGCodeWrapper.cpp`.
No GL initialization, renderer load or fake converter is used.

`tools/chain-probe/marker-time/` commits the original five-line
`ViewerImpl.cpp:1461-1465` excerpt, adapted kernel, unified diff and SHA-256
manifest. The only adaptations are member function → free function, vertices
and mode parameters, and capturing mode rather than `this`. The accumulator,
`0.0f` seed, range and addition remain unchanged. Upstream whitespace is
retained deliberately. `get_time_dhms` is called from the actual `Utils.hpp`;
its unmodified excerpt and empty adaptation diff are also archived. Upstream
code is AGPL-licensed OrcaSlicer/Prusa libvgcode code at the pinned revision.

```sh
node tools/chain-probe/extract-marker-time.mjs --check
ORCA_PROCGCODE_SOURCE=/tmp/ares-orca-procgcode-source \
ORCA_PROCGCODE_NIXPKGS=/nix/store/9qzkr0n1671awz5sjrmbgzmbp0y58wpc-source \
nix build --impure --no-link --print-out-paths -f tools/chain-probe/marker-time.nix
ARES_MARKER_TIME_BIN=<output>/bin/orca-slicer \
node tools/chain-probe/marker-time-fixtures.mjs
cargo nextest run -p ares-vgcode --test marker_time
```

`oracle.json` identifies the native executable/hash, input/output hashes,
source extraction manifest and command. `moves.csv` is the raw processor
output. `native.json` includes 71 real converted vertices (two layers, one
seam, inserted zero-duration vertices and repeated arc/speed-split G-code IDs),
six synthetic numeric inputs exercising ordered f32 accumulation, and 48
native-formatted boundary cases. Neither expected sums nor strings are
computed by Rust/JS.

The TSVs are mechanical projections of native JSON for dependency-free Rust
integration tests: `id, gcodeId, type, role, layerId, x, y, z, durationBits[0],
durationBits[1], elapsedBits[0], elapsedBits[1]` (tab-separated). The synthetic
records are numeric boundary inputs, not claimed to be converted G-code.

The public Rust viewer/SequentialView tests cover exact normal-mode bits,
phantom/repeated IDs, filter-independent prefixes, reverse queries, reload
and the existing seam property-window selection. The failure inventory is
recorded before implementation in that test and the approved spec. The first
RED is the missing timing query/record API; no production changes accompany
that checkpoint. Source-kernel diagnostics never replace official full
G-code/M73 oracles or grant a timing exception for marker-remapping differences.
