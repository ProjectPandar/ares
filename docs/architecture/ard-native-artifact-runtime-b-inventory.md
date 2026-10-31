# Native runtime slice B — individual product inventory

This is a deletion/migration contract, not a record of generated outputs. All
listed outputs become runtime/cache-only. Actual deletion waits for matching
consumer assertions and owner approval. Source/manual inputs are listed below.

| Tracked product | Owner | Runtime consumer recipe | Disposition |
| --- | --- | --- | --- |
| `tests/avoid_crossing_travel/anchor.orca.gcode` | worker4 | runtime corresponding3mf slice; CLI travel assertions | Delete after runtime consumer migration |
| `tests/avoid_crossing_travel/one-wall.orca.gcode` | worker4 | runtime corresponding3mf slice; CLI travel assertions | Delete after runtime consumer migration |
| `tests/avoid_crossing_travel/two-walls.orca.gcode` | worker4 | runtime corresponding3mf slice; CLI travel assertions | Delete after runtime consumer migration |
| `tests/ksr_fdmtest_v4/initialized-reference.json` | worker4 | KSR official/initialized runtime slice + configuration consumers | Delete after runtime consumer migration |
| `tests/ksr_fdmtest_v4/ksr_fdmtest_v4.gcode` | worker4 | KSR official/initialized runtime slice + configuration consumers | Delete after runtime consumer migration |
| `tests/ksr_fdmtest_v4/ksr_fdmtest_v4.initialized-orca.gcode` | worker4 | KSR official/initialized runtime slice + configuration consumers | Delete after runtime consumer migration |
| `tests/parity/arachne/default-prisms.orca.gcode` | worker4 | runtime matching3mf/selectedplate; core consumers→CLI | Delete after runtime consumer migration |
| `tests/parity/color-change/m73-stop/oracle.json` | worker6 | processor approved12 mappings; existing cache typed slice/process | Delete after runtime consumer migration |
| `tests/parity/color-change/oracle-cardinality.json` | worker6 (adjacent spec) | runtime nine manual-input procgcode cardinality | Delete after runtime consumer migration |
| `tests/parity/marker-time/format-red.log` | worker1 | runtime raw/stable source-kernel recipe→JSON/CSV/TSV; web/vgcode/WASM consumers | Deleted after live GREEN: test/build output retained in git history, not input or fallback |
| `tests/parity/marker-time/moves.csv` | worker1 | runtime raw/stable source-kernel recipe→JSON/CSV/TSV; web/vgcode/WASM consumers | Deleted after live GREEN: runtime Native JSON/CSV/TSV only; public vgcode/WASM/formatter assertions unchanged (marker-time/README.md) |
| `tests/parity/marker-time/native.json` | worker1 | runtime raw/stable source-kernel recipe→JSON/CSV/TSV; web/vgcode/WASM consumers | Deleted after live GREEN: runtime Native JSON/CSV/TSV only; public vgcode/WASM/formatter assertions unchanged (marker-time/README.md) |
| `tests/parity/marker-time/oracle.json` | worker1 | runtime raw/stable source-kernel recipe→JSON/CSV/TSV; web/vgcode/WASM consumers | Deleted after live GREEN: runtime Native JSON/CSV/TSV only; public vgcode/WASM/formatter assertions unchanged (marker-time/README.md) |
| `tests/parity/marker-time/red.log` | worker1 | runtime raw/stable source-kernel recipe→JSON/CSV/TSV; web/vgcode/WASM consumers | Deleted after live GREEN: test/build output retained in git history, not input or fallback |
| `tests/parity/marker-time/synthetic.tsv` | worker1 | runtime raw/stable source-kernel recipe→JSON/CSV/TSV; web/vgcode/WASM consumers | Deleted after live GREEN: runtime Native JSON/CSV/TSV only; public vgcode/WASM/formatter assertions unchanged (marker-time/README.md) |
| `tests/parity/marker-time/tagged-oracle.json` | worker1 | runtime raw/stable source-kernel recipe→JSON/CSV/TSV; web/vgcode/WASM consumers | Deleted after live GREEN: runtime Native JSON/CSV/TSV only; public vgcode/WASM/formatter assertions unchanged (marker-time/README.md) |
| `tests/parity/marker-time/tagged-stable.tsv` | worker1 | runtime raw/stable source-kernel recipe→JSON/CSV/TSV; web/vgcode/WASM consumers | Deleted after live GREEN: runtime Native JSON/CSV/TSV only; public vgcode/WASM/formatter assertions unchanged (marker-time/README.md) |
| `tests/parity/marker-time/transport-red.log` | worker1 | runtime raw/stable source-kernel recipe→JSON/CSV/TSV; web/vgcode/WASM consumers | Deleted after live GREEN: test/build output retained in git history, not input or fallback |
| `tests/parity/marker-time/ui-red.log` | worker1 | runtime raw/stable source-kernel recipe→JSON/CSV/TSV; web/vgcode/WASM consumers | Deleted after live GREEN: test/build output retained in git history, not input or fallback |
| `tests/parity/marker-time/vertices.tsv` | worker1 | runtime raw/stable source-kernel recipe→JSON/CSV/TSV; web/vgcode/WASM consumers | Deleted after live GREEN: runtime Native JSON/CSV/TSV only; public vgcode/WASM/formatter assertions unchanged (marker-time/README.md) |
| `tests/parity/motion-attributes/klipper_pressure_advance.actual-speed.rle` | worker6 (adjacent spec) | runtime dynamics process CSV→three exact RLE projections | Delete after runtime consumer migration |
| `tests/parity/motion-attributes/klipper_pressure_advance.rle` | worker6 (adjacent spec) | runtime dynamics process CSV→three exact RLE projections | Delete after runtime consumer migration |
| `tests/parity/motion-attributes/klipper_pressure_advance.time.rle` | worker6 (adjacent spec) | runtime dynamics process CSV→three exact RLE projections | Delete after runtime consumer migration |
| `tests/parity/motion-attributes/marlin2_classic_jerk.actual-speed.rle` | worker6 (adjacent spec) | runtime dynamics process CSV→three exact RLE projections | Delete after runtime consumer migration |
| `tests/parity/motion-attributes/marlin2_classic_jerk.rle` | worker6 (adjacent spec) | runtime dynamics process CSV→three exact RLE projections | Delete after runtime consumer migration |
| `tests/parity/motion-attributes/marlin2_classic_jerk.time.rle` | worker6 (adjacent spec) | runtime dynamics process CSV→three exact RLE projections | Delete after runtime consumer migration |
| `tests/parity/motion-attributes/marlin2_junction_deviation.actual-speed.rle` | worker6 (adjacent spec) | runtime dynamics process CSV→three exact RLE projections | Delete after runtime consumer migration |
| `tests/parity/motion-attributes/marlin2_junction_deviation.rle` | worker6 (adjacent spec) | runtime dynamics process CSV→three exact RLE projections | Delete after runtime consumer migration |
| `tests/parity/motion-attributes/marlin2_junction_deviation.time.rle` | worker6 (adjacent spec) | runtime dynamics process CSV→three exact RLE projections | Delete after runtime consumer migration |
| `tests/parity/motion-attributes/marlin2_option_moves.actual-speed.rle` | worker6 (adjacent spec) | runtime dynamics process CSV→three exact RLE projections | Delete after runtime consumer migration |
| `tests/parity/motion-attributes/marlin2_option_moves.rle` | worker6 (adjacent spec) | runtime dynamics process CSV→three exact RLE projections | Delete after runtime consumer migration |
| `tests/parity/motion-attributes/marlin2_option_moves.time.rle` | worker6 (adjacent spec) | runtime dynamics process CSV→three exact RLE projections | Delete after runtime consumer migration |
| `tests/parity/motion-attributes/marlin_classic_jerk.actual-speed.rle` | worker6 (adjacent spec) | runtime dynamics process CSV→three exact RLE projections | Delete after runtime consumer migration |
| `tests/parity/motion-attributes/marlin_classic_jerk.rle` | worker6 (adjacent spec) | runtime dynamics process CSV→three exact RLE projections | Delete after runtime consumer migration |
| `tests/parity/motion-attributes/marlin_classic_jerk.time.rle` | worker6 (adjacent spec) | runtime dynamics process CSV→three exact RLE projections | Delete after runtime consumer migration |
| `tests/parity/processor-wipe-tower/native.json` | worker4 | runtime full five filament maps; existing VG1 volume oracle | Delete after runtime consumer migration |
| `tests/parity/replay/ender3-classic.orca.gcode` | worker4 | runtime Ender3/classic reference; replay tests mutate Native result | Delete after runtime consumer migration |
| `tests/parity/replay/ender3.orca.gcode` | worker4 | runtime Ender3/classic reference; replay tests mutate Native result | Deleted: replay and CONFIG_BLOCK round trip (`runner/config_block_roundtrip.rs`) slice the 3MF with runtime Native |
| `tests/parity/role-filament-usage/cache-boundaries-map.json` | worker6 | processor approved12 mappings; existing cache typed slice/process | Delete after runtime consumer migration |
| `tests/parity/role-filament-usage/cache-boundaries.gcode` | worker6 | processor approved12 mappings; existing cache typed slice/process | Delete after runtime consumer migration |
| `tests/parity/role-filament-usage/native-map.json` | worker6 | processor approved12 mappings; existing cache typed slice/process | Delete after runtime consumer migration |
| `tests/parity/role-filament-usage/native.gcode` | worker6 | processor approved12 mappings; existing cache typed slice/process | Delete after runtime consumer migration |
| `tests/parity/validation/bbl-object-boundary-expected.gcode` | worker4 | runtime matching3mf/selectedplate; core consumers→CLI | Delete after runtime consumer migration |
| `tests/parity/validation/bbl-toolchange-boundary-expected.gcode` | worker4 | runtime matching3mf/selectedplate; core consumers→CLI | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/custom-gcode/flsun-qqs.native-receipt.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Replace historical receipt with source command recipe; runtime receipt only |
| `tests/parity/validation/cli-exit/custom-gcode/flsun-qqs.result.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/custom-gcode/flsun-qqs.stderr` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/custom-gcode/flsun-qqs.stdout` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/custom-gcode/ginger-1_2.native-receipt.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Replace historical receipt with source command recipe; runtime receipt only |
| `tests/parity/validation/cli-exit/custom-gcode/ginger-1_2.result.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/custom-gcode/ginger-1_2.stderr` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/custom-gcode/ginger-1_2.stdout` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/custom-gcode/ginger-3_0.native-receipt.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Replace historical receipt with source command recipe; runtime receipt only |
| `tests/parity/validation/cli-exit/custom-gcode/ginger-3_0.result.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/custom-gcode/ginger-3_0.stderr` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/custom-gcode/ginger-3_0.stdout` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/custom-gcode/ginger-5_0.native-receipt.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Replace historical receipt with source command recipe; runtime receipt only |
| `tests/parity/validation/cli-exit/custom-gcode/ginger-5_0.result.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/custom-gcode/ginger-5_0.stderr` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/custom-gcode/ginger-5_0.stdout` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/custom-gcode/ginger-8_0.native-receipt.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Replace historical receipt with source command recipe; runtime receipt only |
| `tests/parity/validation/cli-exit/custom-gcode/ginger-8_0.result.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/custom-gcode/ginger-8_0.stderr` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/custom-gcode/ginger-8_0.stdout` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/empty-plate.result.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/empty-plate.stderr` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/empty-plate.stdout` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/notfound.result.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/notfound.stderr` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/notfound.stdout` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/path-outside.native.gcode` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/path-outside.result.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/path-outside.stderr` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/path-outside.stdout` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/postprocess.result.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/postprocess.stderr` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/postprocess.stdout` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/range.result.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/range.stderr` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/range.stdout` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/validate.result.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/validate.stderr` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/validate.stdout` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/values.result.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/values.stderr` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/cli-exit/values.stdout` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/e5max-scv/expected.gcode` | worker4 | runtime matching3mf/selectedplate; core consumers→CLI | Delete after runtime consumer migration |
| `tests/parity/validation/exclude-outline-identity-expected.gcode` | worker4 | runtime matching3mf/selectedplate; core consumers→CLI | Delete after runtime consumer migration |
| `tests/parity/validation/exclude-outline-jitter-expected.gcode` | worker4 | runtime matching3mf/selectedplate; core consumers→CLI | Delete after runtime consumer migration |
| `tests/parity/validation/o6-stock-circle-inward.native-emitted.gcode` | worker4 | runtime matching3mf/selectedplate; core consumers→CLI | Delete after runtime consumer migration |
| `tests/parity/validation/o6-stock-circle-inward.oracle.json` | worker4 | runtime matching3mf/selectedplate; core consumers→CLI | Delete after runtime consumer migration |
| `tests/parity/validation/object-range-layer-height-epsilon.native-emitted.gcode` | worker4 | runtime range3mf slice; preserve layer/config-ranges assertions | Delete after runtime consumer migration |
| `tests/parity/validation/object-range-layer-height-epsilon.oracle.json` | worker4 | runtime range3mf slice; preserve layer/config-ranges assertions | Delete after runtime consumer migration |
| `tests/parity/validation/object-range-layer-height-gap-top.native-emitted.gcode` | worker4 | runtime range3mf slice; preserve layer/config-ranges assertions | Delete after runtime consumer migration |
| `tests/parity/validation/object-range-layer-height-gap-top.oracle.json` | worker4 | runtime range3mf slice; preserve layer/config-ranges assertions | Delete after runtime consumer migration |
| `tests/parity/validation/object-range-layer-height-overlap.native-emitted.gcode` | worker4 | runtime range3mf slice; preserve layer/config-ranges assertions | Delete after runtime consumer migration |
| `tests/parity/validation/object-range-layer-height-overlap.oracle.json` | worker4 | runtime range3mf slice; preserve layer/config-ranges assertions | Delete after runtime consumer migration |
| `tests/parity/validation/object-range-layer-height.native-emitted.gcode` | worker4 | runtime range3mf slice; preserve layer/config-ranges assertions | Delete after runtime consumer migration |
| `tests/parity/validation/object-range-layer-height.oracle.json` | worker4 | runtime range3mf slice; preserve layer/config-ranges assertions | Delete after runtime consumer migration |
| `tests/parity/validation/overhang-jerk/expected.gcode` | worker4 | runtime matching3mf/selectedplate; core consumers→CLI | Delete after runtime consumer migration |
| `tests/parity/validation/print-bed-two-plates-min-plate2-expected.gcode` | worker4 | runtime matching3mf/selectedplate; core consumers→CLI | Delete after runtime consumer migration |
| `tests/parity/validation/semm-m104-k3m-expected.gcode` | worker4 | runtime matching3mf/selectedplate; core consumers→CLI | Delete after runtime consumer migration |
| `tests/parity/validation/sequential-clearance/close.native-command.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Replace historical receipt with source command recipe; runtime receipt only |
| `tests/parity/validation/sequential-clearance/close.result.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/sequential-clearance/close.stderr` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/sequential-clearance/close.stdout` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/sequential-clearance/last-tall.native-command.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Replace historical receipt with source command recipe; runtime receipt only |
| `tests/parity/validation/sequential-clearance/last-tall.stderr` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/sequential-clearance/last-tall.stdout` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/sequential-clearance/lid-tall.native-command.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Replace historical receipt with source command recipe; runtime receipt only |
| `tests/parity/validation/sequential-clearance/lid-tall.result.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/sequential-clearance/lid-tall.stderr` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/sequential-clearance/lid-tall.stdout` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/sequential-clearance/rod-tall.native-command.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Replace historical receipt with source command recipe; runtime receipt only |
| `tests/parity/validation/sequential-clearance/rod-tall.result.json` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/sequential-clearance/rod-tall.stderr` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/parity/validation/sequential-clearance/rod-tall.stdout` | worker4 | Native command-result recipe: exact exit/channels/JSON/retained export | Delete after runtime consumer migration |
| `tests/processor_seam_cache/anchor.orca.gcode` | worker6 | processor approved12 mappings; existing cache typed slice/process | Delete after runtime consumer migration |
| `tests/processor_seam_cache/classification.orca.gcode` | worker6 | processor approved12 mappings; existing cache typed slice/process | Delete after runtime consumer migration |
| `tests/processor_seam_cache/solid-rectilinear.orca.gcode` | worker6 | processor approved12 mappings; existing cache typed slice/process | Delete after runtime consumer migration |
| `tests/processor_seam_cache/two-walls.orca.gcode` | worker6 | processor approved12 mappings; existing cache typed slice/process | Delete after runtime consumer migration |
| `tests/processor_seam_cache/wipe-off.orca.gcode` | worker6 | processor approved12 mappings; existing cache typed slice/process | Delete after runtime consumer migration |

| `tests/parity/validation/kobra3-centered/findings.json` | worker4 | replace historical Native capture/status rows with source-only recipe/inputs and architecture conclusions | Remove runtime capture report from Git |

## Retained source inputs

- All corresponding3mf/stl, presets and overrides.json; command argument recipes
  reconstructed from historical receipts without output hashes/timestamps.
- Nine producer-less tests/parity/color-change/*.gcode and marker-time/path.gcode
  are handwritten inputs, not Native outputs.
- tests/processor_seam_cache/classification-injected.gcode is manual injection;
  preserve it, with worker6 auditing its source-use contract.
- tools/chain-probe/marker-time original upstream excerpts, kernel adapters,
  source extraction manifest/diff/SHA are source inputs, not runtime Native
  products. tests/ksr_fdmtest_v4/stock_project_options.json is source-derived option
  inventory from generate_task5_inventory.mjs, not a Native slice product.
- README/ARD retains source/contract conclusions, not generated stream dumps.
  SHA256SUMS loses output rows and preserves input identity only.
- tests/parity/validation/kobra3-centered/0.4-outer-only-provenance.json is
  input transformation provenance (one brim setting), not Native output; retain.
- Existing input-oriented preset JSON under validation/e5max-legacy and
  validation/empty-filament-colour-cubicon stays; do not mass-delete JSON by extension.

## Consumers and shared boundary checks

Processor12 case consumers and exact deferred ignored disposition are governed
by the worker6 processor spec. Other core native consumers are explicitly named
in the parent ARD; they move to CLI/public API tests. CLI families include
cli_exit_channels, cli_custom_gcode_exit, cli_retained_exit, cli_sequential_clearance,
ksr_fdmtest_v4, avoid_crossing_travel and orca_parity replay/range controls.
Marker consumers include ares-vgcode/tests/marker_time.rs, ares-wasm/tests/
marker_time.rs and marker_time_tagged.rs, web/e2e/markerTimeFormat.spec.ts and
chain-probe marker-time generators; compile-time Native includes are removed.
Color/dynamics core live oracle tests migrate to CLI and keep exact raw/stable
comparison scope and attributes. Runtime products must remain independently
generated, not computed from Ares expected output.

All individual output files in this table are audited candidates with an explicit
recipe. Before deletion search full paths and directory-derived reads; update
shared consumers before removing their common file. Provenance/findings JSON
which stores Native capture output is additionally removed or replaced by
source-only input/recipe metadata; conclusion-only architecture facts stay.
Do not declare zero Native artifacts solely from this table/header search:
audit remaining tracked CSV/TSV/JSON/log/RLE and content provenance at final gate.
