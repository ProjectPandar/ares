# Role filament usage: Native oracle proof and first RED

Date: 2026-10-08. Approved spec: `cfd1bb5ef`.
RED baseline: `de57be8b0`; production files remain unchanged.

## Oracle reliability before assertions

Used the existing raw procgcode ELF's ordinary `--slice` entry, under
`nix shell nixpkgs#gdb -c gdb`, on the unmodified
`tests/parity/validation/kobra3-centered/0.8.3mf`. No Orca rebuild was done.
The extractor is `tools/chain-probe/role-filament-map.py`; complete frozen
Native output and records are in `tests/parity/role-filament-usage/`.

The source boundary is `GCodeProcessor.cpp:1621-1639` (role conversion),
`:1641-1649,2709` (final cache processing), `:6036` (result map copy), and
`ExtrusionEntity.hpp:20-43` (core role ids). `UsedFilaments` is declared at
`GCodeProcessor.hpp:583-619`. The existing ELF SHA256 is
`59f9ea7e16165df6979a243de7d6acd49d8cf523ac5de6d3fec08d6e25c7896e`.

Before extracting data, the helper checks that exact ELF hash. Its offsets
were checked against the ELF's `process_role_cache` disassembly:
cache `UsedFilaments+0x130`, map `+0x138`, root `+0x148`, count `+0x160`,
node role `+0x20`, pair doubles `+0x28/+0x30`. The tree traversal verifies
unique nodes, complete node count, sorted unique keys and a zero final role
cache. It captures the map on return from `process_caches`, in the same
Native process which subsequently writes the G-code/footer.

| Same Native slicing run | Length | Mass |
| --- | --- | --- |
| Sum of all eight role map entries | 343.16809722384134 mm | 1.0235158534141071 g |
| Sum formatted at footer precision | 343.17 mm | 1.02 g |
| Native-generated footer | 343.17 mm | 1.02 g |

This satisfies the requested extraction prerequisite. The fixture has one
filament and no flush; it avoids the unequal-material/tool-cache and flush
scope differences described in the spec. The reference stores both original
f64 bit patterns, so rounding is confined to the footer cross-check.

Captured roles: Perimeter, ExternalPerimeter, InternalInfill, SolidInfill,
TopSolidInfill, BottomSurface, InternalBridgeInfill and Brim. A second Native
slice without gdb has identical complete G-code after generated-by-only
normalization. Fresh live gdb extraction in the nextest run reproduces the
entire frozen role map and footer proof. Output directories are newly created;
the reader rejects an existing directory and cannot reuse an old snapshot.

## RED result on current main

```sh
ARES_ROLE_ORACLE_LIVE=1 \
CARGO_TARGET_DIR=${TMP_ROOT}/ares-worker6-green-target CARGO_BUILD_JOBS=8 \
cargo nextest run -p ares-core --test processed_role_filament_usage \
  --test-threads 16 --no-fail-fast
```

Result: **one PASS, one expected FAIL**, no skipped tests:

- `native_role_map_is_bound_to_same_run_footer`: PASS, fresh Native slice,
  complete role map reproduction, same-run footer check and G-code control.
- `processed_role_filament_usage_matches_native_map`: FAIL at public JSON
  `used_filaments_per_role`, with message
  `Native has eight nonzero roles; Ares must export the role map`.

The failing E2E first passes reference SHA links, footer consistency and the
complete Native/Ares G-code comparison. Thus there is no divergent G-code
line in this RED: the missing public processor-result field is the difference.
This is the spec-approved result-level RED, not a compile failure or a
manually constructed floating-point example. Both role key set and both f64
values per role are asserted after the missing-field check for GREEN.

All three complete G-code streams (Native under gdb, Native without gdb,
Ares public processed slice) share generated-by-only normalized SHA256:
`431cecf3a124f0cde14e1933332245ea60abff21198904314ca962b74003b128`.
Frozen Native file SHA256:
`23db2576bcf2a2a7db7ddeccfd9f2a0812aa31b186a7feff92bcd4c9b34cb43b`.
Ares file SHA256:
`e999e0e7fd3dd78a46b3e82a2ce84d9166172682e7dc2dc7220aa45e51cc77b8`.

An initial harness-only SHA formatting compile error was fixed before the
actual RED runs; it is retained in `red.log` and is not counted as RED.
The final run is `/tmp/worker6-role-oracle/red-final.log`; the identical
post-rebase predecessor is `red-new-main.log`. Lint checks passed:
`cargo clippy -p ares-core --test processed_role_filament_usage -- -D warnings`,
`cargo fmt --all --check`, and source/docs-only `git diff --check`.
The full patch whitespace check reports Native-generated trailing spaces and
blank lines in `native.gcode`; those bytes are retained verbatim. No artifact
trimming or whitespace normalization is added to make that check quiet.

## Evidence and next boundary

The committed fixture README contains the full reproduction command. Runtime
Ares JSON and G-code, plus fresh Native logs, map and output, live under
`target/role-filament-usage/`. A SHA inventory is retained at
`/tmp/worker6-role-oracle/evidence-index.json` for the diagnostic logs,
disassembly, reference captures and test outputs.
The inventory contains 40 files; SHA256:
`c62ae74228862b4373b00e51088a1917fe8b5e12d04366bbaac1b978ab3ab8ca`.

This first RED covers the real-project accumulator/export boundary with eight
roles. It does not claim the spec's unequal-material/cache-span or viewer
cases: those tests must still precede their respective production changes.
No production, comparator weakening, new normalization or GREEN is included.
The browser window announcement arrived after the directed runs and lint;
no new heavy task is started during the window.
