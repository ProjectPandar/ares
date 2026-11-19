# CLI plate selection and multi-plate export

Native at `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19` declares `--slice` as an integer action with value default `0` (`src/libslic3r/PrintConfig.cpp:12780–12785`). An omitted action does not start a Native slice. The existing Ares `slice -o FILE [--plate N]` entry remains single-output; it keeps its default first-plate selection. The separate `ares slice --slice N [--outputdir DIR] INPUT.3mf` entry accepts `0` for all and positive one-based selections, and does not combine with `-o` or `--plate`.

## PartPlate lifecycle

`src/OrcaSlicer.cpp:1573–1575,5291–5295,6443–6446,6524–6540` takes the selection into model loading, rejects out-of-range signed values as `CLI_INVALID_PARAMS` (`-2`, report plate index `0`), and iterates the `PartPlateList` in plate order. Explicit positive selection keeps the existing selected-import semantics. All-plate selection starts from the whole project. The `Print::apply`/`Print::validate` precheck of *every* plate precedes `Print::process` on multi-plate runs (`:6524–7055,7288–7292`; `src/libslic3r/Print.hpp:911,935`). An empty/unprintable later plate aborts with `CLI_NO_SUITABLE_OBJECTS` and that plate's number without writing any earlier G-code. Once processing begins, Native publishes plates in order rather than publishing a completed batch atomically.

The destination is the existing `ares-core` project input/validation/processing path. Reuse the load/compose routes for embedded 3MF, explicitly loaded preset files and catalog refresh, and the same `Project` plate views and applied-print validation. Keep the iteration, validation order and attribution in core. The CLI only decodes arguments and preset resources and serializes outputs/reports; it does not implement a plate loop around the public single-slice command.

With a nonempty `--outputdir`, `OrcaSlicer.cpp:7181–7192` and `GCode::do_export` publish `plate_<one-based-index>.gcode` there (including a new directory); without it, the temporary G-code is not retained. The Linux result (`OrcaSlicer.cpp:535–586,8126–8136`) is in `DIR/result.json` or `./result.json`, with top-level `plate_index` equal to the requested number on success (`0` for all) and ordered `sliced_plates` records. A precheck failure carries the failing plate and no completed plate records. As elsewhere in the CLI, Ares reports zero wall-clock fields; parity compares every other result byte using `native_wall_clock::result_comparison_bytes`.

## Verification and exclusions

Replay the committed two-plate synthetic project from a temp directory on both Native lanes. Compare both full streams, filenames, status, and full Linux result after the existing wall-clock normalization. Check `--slice 2`, negative and out-of-range selections, omitted output directory, a new output directory, and a runtime variant making just plate 2's instance unprintable. Verify that the variant leaves neither plate G-code nor a `sliced_plates` field. Keep the existing `-o`/`--plate` controls.

Deferred: STL combinations with `--slice` or the Native transform flags until the STL project-pipeline milestone; no STL placement or `ensure_on_bed` changes here. Native GUI launch, `--export-3mf`, and separate CLI actions such as `--load-slicedata`/`--export-slicedata` are outside this slice.
