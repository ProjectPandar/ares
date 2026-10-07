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

## Approved tagged stable authority

`tagged-stable.tsv` is the stable native JSON projection
`id, gcodeId, durationBits[0], durationBits[1], elapsedBits[0], elapsedBits[1]`.
It contains 7,347 vertices for the real project slice using
`tagged-overrides.json`; `tagged-oracle.json` records its provenance. The
operator-approved bounded raw/stable difference is documented in
`docs/architecture/web-colorchange.md` under Known difference: tagged marker
Time. Stable vertex IDs are authoritative; raw IDs must not be compared by
ordinal. The first ordinal diagnostic failure remains in the artifacts.

```sh
cargo nextest run -p ares-wasm --test marker_time_tagged
# Build marker-time-stable-options.nix with the same source/nixpkgs variables.
ARES_MARKER_TIME_BIN=<raw-output>/bin/orca-slicer \
ARES_MARKER_TIME_STABLE_BIN=<stable-output>/bin/orca-slicer \
node tools/chain-probe/marker-time-pair.mjs \
  target/marker-time-stable-proof/oracle-input.gcode /tmp/fresh-marker-pair
node tools/chain-probe/marker-time-bounds.mjs /tmp/fresh-marker-pair
cmp tests/parity/marker-time/tagged-stable.tsv /tmp/fresh-marker-pair/stable.tsv
```

The real processed-slice → public WASM adapter integration checks every
stable identity, both duration modes and Normal elapsed bits after moving
display buffers. The separate bounded-impact proof checks the extra phantom,
exact five-field direct-difference set, unaffected nonzero-duration ordering,
and both complete native prefix recurrences, including tails. Scripts emit
only counts and a first difference; complete evidence is written to files.
