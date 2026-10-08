# Native role-filament reference

Source boundary: `GCodeProcessor.cpp:1621-1639` role cache, `:1641-1649`
finalization and `:6036` result-map copy. See
`docs/architecture/gcode-processor-role-filament-usage-spec.md`.

The unmodified project is `tests/parity/validation/kobra3-centered/0.8.3mf`,
SHA256 `0dc37a07daedf1aa01a48832a2ff3b4e70f6e367075274391e42884a98db1b0c`.
The pinned raw procgcode ELF runs its normal `--slice` command for this
capture; the diagnostic `--process-gcode` branch is not used. No Native
rebuild or behavioral patch was added. The existing probe patch is
`tools/chain-probe/procgcode.patch`; ELF SHA is recorded in `native-map.json`.

Reproduce the reference from the repository root:

```sh
ARES_ROLE_ORACLE_INPUT="$PWD/tests/parity/validation/kobra3-centered/0.8.3mf" \
ARES_ROLE_ORACLE_OUTPUT=/tmp/role-filament-fresh \
nix shell nixpkgs#gdb -c gdb -nx -batch \
  -x tools/chain-probe/role-filament-map.py > /tmp/role-filament-fresh.log 2>&1
```

`native.gcode` is the **complete** output of that same Native slicing run.
`native-map.json` holds all eight role keys, both f64 values and their original
u64 bit patterns. The reader verifies the ELF SHA before using offsets,
checks the map's node count, unique sorted keys and final cache reset, and
only writes the oracle JSON after a successful Native exit and footer check.
It does not recompute role values from extrusion commands.

The map sums to **343.16809722384134 mm / 1.0235158534141071 g**. Native's
same-run footer is **343.17 mm / 1.02 g**. The comparison uses the footer's
two-decimal format; map comparisons use the original f64 bits. This fixture
has one filament and no flush, so footer totals and role totals have the
same accounting scope. Do not apply this equality to flush-inclusive totals
or to unequal-material caches spanning a ToolChange.

An additional uninstrumented run of the same ELF/project produced identical
whole G-code bytes after generated-by-line normalization alone. Captured
diagnostic logs and disassembly are retained under `/tmp/worker6-role-oracle/`:
`frozen.log`, `uninstrumented.log`, `process-role-cache.disassembly` and the
two Native output directories. There is no use of the stable-options
diagnostic oracle or a reconstructed Python/Rust role formula.

Run the public API E2E against this frozen, footer-validated Native result:

```sh
ARES_ROLE_ORACLE_LIVE=1 \
CARGO_TARGET_DIR=/tmp/ares-worker6-green-target CARGO_BUILD_JOBS=8 \
cargo nextest run -p ares-core --test processed_role_filament_usage \
  --test-threads 16 --no-fail-fast
```

`ARES_ROLE_ORACLE_LIVE=1` makes the oracle test run a fresh Native slice under
gdb, compare its entire role map/footer check with the frozen record, and
check whole G-code bytes with generated-by-only normalization. Without this
flag the tests use the frozen reference and remain runnable without Linux/nix.
Fresh oracle logs/output are retained at
`target/role-filament-usage/native-live-*/`. Each run creates a fresh directory;
the extractor rejects an existing output directory, preventing stale snapshots.

The test first validates reference SHA links and the same-run footer sums,
then compares complete Ares/Native G-code with generated-by-only normalization,
and finally compares the complete public statistics role map and both f64
bit patterns per role. Runtime Ares artifacts are written to
`target/role-filament-usage/`. On the RED baseline the G-code control passes
and the missing `used_filaments_per_role` result fails.

This first RED locks down real-project role accumulation and result export.
The spec's targeted unequal-material/tool-cache and viewer cases remain to
be added before their respective production changes; this reference does
not claim coverage of those independent input sequences.
