# Kobra-3 centered stock-matrix coverage variants

Issue #37 is closed: the **four original inputs remain MATCH**. Their cube
build translation `(5,-56,5)` produces world bbox `(0,-61,0)..(10,-51,10)`,
outside plate 1's `(0,0)..(255,255)` bed (origin `(0,0,0)`). Both CLIs reject
at the volume gate before `Print::validate`; those rows never cover slicing.
Native records the same Empty text in `result.json.error_string` with
`CLI_NO_SUITABLE_OBJECTS=-50` (`Utils.hpp:52`, `OrcaSlicer.cpp:5718-5724`).
Exit 206 versus Ares 1 is the existing CLI-adapter category.

These additional variants change **only the build-item XY translation** to
`(127.5,127.5,5)`, producing world bbox
`(122.5,122.5,0)..(132.5,132.5,10)`. Mesh, component transform, all config,
profiles, templates, IDs, and plate metadata are unchanged. The retained
diagnostic authoring script compares every uncompressed ZIP entry and permits
exactly this one replacement (no script or comparator change is included here):

```sh
perl /tmp/ares-kobra3-gate-order/center-variant.pl \
  ORIGINAL.3mf VARIANT.3mf provenance.json
```

All four original profiles are Klipper/relative-E. Missing `G92 E0` is not a
reset violation: `Print.cpp:1732` requires reset only for Marlin flavors/non-BBL.
Sources are OrcaSlicer v2.4.2, commit `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Replay and findings (fresh base `818b414e81af271e6ab7edabae75040807b11bba`)

The CLI was rebuilt on this base and all four variants replayed. Previous
capture on `1c50bf50` gave the same behavioral results; the original four
Empty replays remain retained separately.

```sh
ORCA_REPLAY_BIN=/path/to/seeded-orca ORCA_REPLAY_CPU=0 ORCA_REPLAY_SEED=0 \
  scripts/orca-seeded-parity.sh --slice 0 --outputdir native-out \
  tests/parity/validation/kobra3-centered/0.2.3mf
target/debug/ares slice -o ares.gcode \
  tests/parity/validation/kobra3-centered/0.2.3mf
```

Native and Ares both exit 0 and emit real G-code for every variant. Raw bytes
are **not identical**, including the known generator identity/time line. The
**unchanged** `tests/ksr_fdmtest_v4/golden.rs` ordered-byte comparator, using its
existing generator-only policy, gives:

| Nozzle | Comparison | First behavioral difference (line 37; pre-Brim travel) |
| --- | --- | --- |
| 0.2 | DIVERGENT | Native `G1 X119.171 Y119.333 F21000`; Ares `G1 X119.474 Y119.028 F21000` |
| 0.4 | PASS | None under existing generator-only policy |
| 0.6 | DIVERGENT | Native `G1 X119.26 Y119.653 F21000`; Ares `G1 X119.359 Y119.553 F21000` |
| 0.8 | DIVERGENT | Native `G1 X119.296 Y119.815 F21000`; Ares `G1 X119.343 Y119.751 F21000` |

`findings.json` retains original/variant hashes, source command directories,
provenance, CLI outcomes, complete-stream hashes/lengths and first differences.
Full raw replay artifacts: `/tmp/ares-kobra3-gate-order/`.

These are **additional coverage fixtures**, not replacements for the four
original rejection controls or asserted-pass regressions for divergent paths.
No slicing code, diagnostics, comparators or gate ordering were changed.
Follow-up assigned to lint **after O6 plate origin**: establish why 0.4 passes while the other three
nozzles diverge. Trace actual Brim width/spacing/loop counts and the earliest
differing upstream `Brim.cpp` data before writing a rewrite spec. The first
G-code difference alone does not establish a root cause or owning function.
