# Classic Filled public RED — initial tracer bullet

SPEC approved at `b22726a64`; margin clarification retains upstream pin
`8500fcdcc` and its constant-derived `bridged_infill_margin`, not a new option.
Rust production base: `4c5812d18e26a0a555f457a23ca11d672271cf3f`.
This commit changes tests only (plus this evidence record), not core behaviour.

## Public seam and frozen inputs

`runner/counterbore_filled.rs` calls `ares_core::slice_with_profiles` and an
independent Native `--load-settings` / `--load-filaments` / `--arrange 0` /
`--slice 0` invocation. It uses the existing frozen binary STLs and
`geometry-manifest.json` / `profile-provenance.json` under
`tests/parity/counterbore-partiallybridge/`. Every input/profile parent hash is
asserted before slicing. Only `counterbore_hole_bridging=sacrificiallayer` is
changed from the original classic WEMAKE3D/Generic PLA stock selection.

Primary STL SHA256:
`1274c56d97fd205740505d88f077971c7ffe79f1552df78a4e7432de557e465e`.
Catalog SHA256:
`edaf9f0ac06153855062c5942b3424b1d8be9ef3fedba3dd1ca3f5f40fb64e65`.

Existing six partial/None tests are unchanged. All comparisons retain complete
ordered output, normalizing only the established generated-by identity.
The new Native Filled-vs-None diagnostic normalizes **both** as Orca streams,
then uses `first_difference`; it is a public output-change witness, not proof of
specific internal branch coverage.

## Execution and results

CPU15 single core, `CARGO_BUILD_JOBS=1`, nextest `--test-threads 1`,
`CARGO_INCREMENTAL=0`, Rust 1.99.0. No full/workspace runs.
Fresh valid evidence root: `/tmp/worker4-counterbore-filled-red-fbNb`.
Each mode has a separate `cache/`, `artifacts/`, `run.log`, `exit`.
Corrected run reused the first run's compiled target only:
`/tmp/worker4-counterbore-filled-red-bGXN/target`.
START/END receipts were posted to `${TMP_ROOT}/ares-coord-inbox.log`.

| Case | official | seeded |
| --- | --- | --- |
| Filled stepped-hole primary | exact counterbore guard RED | same RED |
| Filled first-layer-hole | exact counterbore guard RED | same RED |
| Filled straight-hole | exact counterbore guard RED | same RED |
| Filled two-islands | exact counterbore guard RED | same RED |
| Filled solid-plate | complete-output PASS | PASS |
| Existing classic partial/None controls (6) | 6/6 PASS | 6/6 PASS |

Each mode: **7 PASS / 4 expected RED failures** out of 11 selected tests.
Nextest reports 1076 other tests filtered out; no new ignore/skip was added.
All four errors are
`UnsupportedProjectFeature("counterbore_hole_bridging")`; Native commands
succeeded, not timeout or environment errors. rustfmt and test-target strict
clippy (`cargo clippy -p ares-cli --test orca_parity -- -D warnings`) passed.
This is not workspace strict/WASM/full evidence.

Official wrapper SHA256:
`1689a2ff46564cf723ad90c663051b58115920c3cce639bd2961340955cebbf3`.
It invokes Native through `setarch -R`. No Variant A occurred in the successful
control comparisons. Seeded wrapper also uses `setarch -R`, seed 0, CPU15.

Native primary Filled-vs-None normalized output differs at byte 1001, line 40:
None `M73 P69 R1`, Filled `M73 P71 R1`; lengths 45355 vs 41943 bytes in both
modes. Raw Filled SHA: official
`09fcb0b8cc6c4fda1096e200557cf118e1e02de267043caa0b57c1676cdcd6cd`, seeded
`3665b92ef3f4691b18e0c2d4652c7c7cc273d1d2fc0dc77f6b0eca9c0c7b7242`.
This proves a real Native output difference, not just the footer option text;
it does not itself prove nested-island/merged-region branch reachability.

Primary artifacts:
- official: `official/artifacts/orca-runner-oDNVmU/`;
- seeded: `seeded/artifacts/orca-runner-SCFmu6/`.

Each contains `native-filled.gcode`, `native-none.gcode`,
`native-filled-reachability.json`, `counterbore-result.json`, `ares-error.json`,
input, catalog, selection, settings/overrides and Native command receipts.
Passing controls contain `ares.gcode` and raw Native output with recorded SHA.
Those Ares bytes are the baseline for byte-identical non-Filled GREEN controls.

## Reproduction

At this RED commit, use a fresh Native cache/artifact root and retain the
wrapper SHA. Execute the following under CPU15 with the mode environment:

```sh
taskset -c 15 env CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 \
  RUSTUP_TOOLCHAIN=1.99.0 CARGO_TARGET_DIR="$target" \
  ARES_REQUIRE_ORCA=1 ARES_PARITY_PROFILE_ENTRY_REFERENCE=1 \
  ARES_ORCA_CACHE_DIR="$root/cache" ARES_PARITY_ARTIFACT_ROOT="$root/artifacts" \
  cargo nextest run -p ares-cli --test orca_parity \
  -E 'test(counterbore_)' --no-fail-fast --test-threads 1
```

Mode environment matches `${TMP_ROOT}/ares-parity-control-set-v2.sh`'s official or
seeded Native binary/resources selection, but **do not run that full script**.
Official uses `/tmp/orca-wrapper-inherit-cpu.sh` and the approved official
AppDir; seeded uses `scripts/orca-seeded-parity.sh`, producer
`/nix/store/1k0p5nlbs3zsfdcy08g3y5fb7bcm1qrv-orca-slicer-seed-replay-2.4.2`,
`ORCA_REPLAY_SEED=0`, `ORCA_REPLAY_CPU=15`,
`ARES_PARITY_SEEDED_REFERENCE=1`. Do not use the full suite's random CPU runner.
Native process commands and producer identities are independently retained in
artifact/cache receipts. The launcher is preserved at
`/tmp/worker4-counterbore-red-corrected.sh`.

## Invalid witness retained and remaining scope

First root `/tmp/worker4-counterbore-filled-red-bGXN` recorded the same expected
REDs and control PASSes. Its extra Native-vs-Native diagnostic incorrectly used
an Orca-vs-Ares comparator, returning `expected one Ares generator line, found
0`. **That diagnostic is VOID**, not reachability evidence. The harness was
corrected before this commit; fresh Native cache/artifacts reran the unchanged
11-case matrix. No retry-to-GREEN or assertion relaxation occurred.

This is the initial public RED commit, not a claim that the full SPEC matrix is
complete. Before production work claiming nested-island and compatible-region
recovery support, freeze and prove those two fixtures through public Native
inputs. Two independent islands do not establish an island-in-hole branch.
Retain Arachne as W4's next source-cited slice. Do not remove old private guard
assertions merely to conceal failures; update their contracts only with the
real upstream Filled port and public evidence.
