# Kobra centered input provenance and historical report removal

## Boundary

P2 removes the unused historical Native capture report
`tests/parity/validation/kobra3-centered/findings.json` (616 lines). It preserves
the authored input contract in that directory's README. No Rust, production,
cache, test registration, comparator, or configuration changes are included.

Upstream revision: `8500fcdccaa10b5099ac20d252af3a7c560046f1`.
The owning source boundaries are:

- `libslic3r/Model.cpp:2976,3014-3026`, `Model::setPrintSpeedTable`: obtains the
  configured bed polygon and subtracts excluded areas.
- `libslic3r/Brim.cpp:421,643-683`, `outer_inner_brim_area`: obtains per-extruder
  printable polygons, defaults to the model bed polygon, and subtracts the
  unprintable region from object/support brim areas.
- `slic3r/GUI/PartPlate.cpp:2510-2520`, `PartPlate::set_print`: passes plate
  origin to Print; `libslic3r/Print.cpp:2618,2636-2638`, `Print::export_gcode`:
  passes that origin into the G-code export offset.

The unchanged Rust verification boundary is
`crates/ares-cli/tests/orca_parity/brim_bed_region.rs`. Its existing runtime
adapter invokes Native slicing through `OrcaRunner`/`stages::run`, reads the
fresh `plate_1.gcode`, and compares the complete stream through
`artifacts::compare`. This slice removes a historical report alongside that
adapter; it introduces no independent Ares pipeline or replacement oracle.

## Consumer audit

Audit base: `a24da8aaa`, before adding this spec. Commands:

```sh
git grep -n -I -F 'findings.json' -- scripts web crates .github tests
git grep -n -I -E 'kobra3-centered/findings\.json|kobra3-centered.{0,120}findings|findings.{0,120}kobra3-centered' -- . ':(exclude)OrcaSlicer/**'
git grep -n -I -F 'kobra3-centered' -- scripts web crates .github
```

The first command finds only this fixture directory's README, line 53. The
second finds only the architecture inventory's historical removal entry,
`docs/architecture/ard-native-artifact-runtime-b-inventory.md:132`.
No active report consumer was found in scripts, web, crates, or CI. Actual
fixture consumers read the retained 3MF inputs: brim tests/controls, processor
role and WIPE tests, and the WASM role-filament test. These remain unchanged.

The separately requested classic replay audit uses:

```sh
git grep -n -I -F 'ender3-classic.orca.gcode' -- . ':(exclude)OrcaSlicer/**'
```

| Hit at audit base | Kind |
| --- | --- |
| `crates/ares-cli/tests/stock_project_whole_file/ordered_bytes_tests.rs:63` | Active KSR `include_bytes!` consumer |
| `docs/architecture/ard-native-artifact-runtime-b-inventory.md:45` | Inventory documentation |
| `docs/architecture/native-artifact-cleanup-replay-reference-spec.md:41` | P1 retention decision |
| `tests/parity/replay/README.md:38` | Runtime/retention contract |

Classic remains tracked for the KSR migration. The original
`ender3.orca.gcode` remains for the separate web/encode-roundtrip
migration. Neither product nor any KSR/web reader is writable in P2.

## Input contract to retain

Keep all five inputs byte-identical: `0.2.3mf`, `0.4.3mf`,
`0.4-outer-only.3mf`, `0.6.3mf`, and `0.8.3mf`. Keep the handwritten
`0.4-outer-only-provenance.json` byte-identical; it is input transformation
provenance, not a Native receipt.

Preserve the report's authored information in the README: original input SHA,
centered variant name/SHA, changed ZIP member, common build-item translation,
bed/plate geometry, and upstream source ownership. The four centered variants
change only `3D/3dmodel.model`: translation `(5,-56,5)` becomes
`(127.5,127.5,5)`, retaining grounded Z. The local cube bounds are
`(-5,-5,-5)..(5,5,5)`; centered world bounds become
`(122.5,122.5,0)..(132.5,132.5,10)` on bed `(0,0)..(255,255)` with origin zero.
Other ZIP members, profiles, templates, IDs, and mesh geometry are unchanged.

Document the explicit 0.4 outer-only derivative separately: only
`Metadata/project_settings.config` key `brim_type` changes from `auto_brim` to
`outer_only`. The existing handwritten provenance records this recipe. The
0.2/0.6/0.8 centered inputs already select `outer_only`; the 0.4 runtime outer
test uses the derivative, not `0.4.3mf`.

Remove historical Native/Ares output hashes, sizes, result/status tables,
captured differences, temporary capture paths, and generated receipts. Do not
copy these into new constants or new reports in Git. The four existing tests
retain input-SHA, Native brim-activation, and full-stream comparison assertions
with their existing generator-only normalization and dependency policy.

## Verification and deferred work

Use a repository-external tracked-file absence inventory as the deletion RED:
it fails while the report is tracked and passes after removal. Add no Rust
path-existence test. Before and after removal, run the four existing
`brim_bed_region_kobra3_{02,04,06,08}_outer_matches_native` tests in required
mode against real Native; preserve logs and paired artifacts.
Audit protected input/provenance SHA values, unchanged Rust assertions, and the
final consumer search. Run default-environment workspace, clippy, and fmt.
No deletion-only gate waiver is assumed from earlier groups.

Fresh seeded/official pairing is unnecessary for this report/README
only deletion; no production or
test execution path changes. Historical report cleanup does not resolve any
other Native artifact group. Brim algorithms, plate transforms, runtime runner
strategy, KSR, web CONFIG_BLOCK consumers, and the remaining artifact groups are
deferred.
