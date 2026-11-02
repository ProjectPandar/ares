# Empty selected plate: Model read diagnostic rewrite

## Source boundary

Locked OrcaSlicer 2.4.2 (`8500fcdccaa10b5099ac20d252af3a7c560046f1`):

- `OrcaSlicer.cpp:1616-1640` passes the explicit `--slice` plate number to `Model::read_from_file`.
- `Format/bbs_3mf.cpp:2059-2080`, `_BBS_3MF_Importer::_load_model_from_file`, omits objects outside the selected plate during assembly. `2351-2363` removes objects without loaded volumes.
- `Model.cpp:320-325,354-355`, `Model::read_from_file`, calls that importer and rejects the resulting empty model with `RuntimeError("The supplied file couldn't be read because it's empty")`.
- `OrcaSlicer.cpp:1874-1877` catches the read exception, writes `<input path>: <exception message>\n` to stderr, records `CLI_DATA_FILE_ERROR` and exits. `Utils.hpp:30` defines this category as `-6` (POSIX process exit250). The structured result uses the distinct generic `cli_errors[CLI_DATA_FILE_ERROR]` text at `OrcaSlicer.cpp:118`: `The input model file to the slicer can not be parsed.`

`Model.cpp:228` has the same text in **STEP loading**; it is not the selected-3MF owner. This rewrite must not cite the STEP branch as its causal boundary.

## Rust destination and non-negotiable decisions

The owning byte-oriented compatibility shell is `Project::select_plate` in `crates/ares-core/src/project/domain/plate_selection.rs` (separated from `domain.rs` to keep the changed modules below 400 LOC). Its object/instance projection corresponds to selected-plate importer assembly; when that existing projection is empty, its current Ares-specific `plate {id} has no printable objects` text must become the source Model read exception text above.

- Preserve `SliceError::InvalidInput` and the existing selection stage before print-volume and `Print::validate`. Do not turn a failed model read into a print-validation failure.
- Apply the exact source diagnostic to the existing empty-projection branch only. Do not change instance filtering, plate origin, geometry, object ordering, overrides or processing eligibility.
- Preserve the distinction from a nonempty plate whose objects are outside the volume: that later gate retains `CLI_NO_SUITABLE_OBJECTS=-50`/the existing `One of the plate is empty…` diagnostic.
- Retain Ares's current CLI error adapter (exit1 and `Error: InvalidInput(...)` wrapper). Literal Native exit250, its path-prefixed stderr and its generic result.json field are reference **channel classification**, not requirements to add a Native CLI adapter or return filesystem paths from core.
- Public composition and validation entry points selecting this empty plate must expose the same exact Model diagnostic. Core remains filesystem/terminal free and browser-WASM compatible.

## Included and deferred behavior

Included: diagnostic parity for an explicitly selected, existing plate whose current model projection has no objects; Native read-stage classification; independent public API RED coverage and unchanged populated-plate behavior.

Deferred: plate-not-found text, selecting all plates, globally empty archives, STEP/STL import errors, importer instance/object identity redesign, malformed archive handling, literal Native CLI exit/result.json emulation, and unrelated bridge/multifilament processing. Existing Ares loaders/selection are temporary compatibility shells, not a new Ares-owned pipeline.

## Verification contract

Use the unchanged original `tests/parity/validation/print-bed-two-plates.3mf` (empty plate1/populated plate2) and its existing single-filament `-min` variant. Independent fresh Native `--slice 1` runs must establish read rejection250/-6, the exact path-prefixed detailed stderr and the distinct generic result.json diagnostic, with no G-code emitted. Only then compare public Ares `validate_project_plate`, unvalidated/validated slicing, and processed validated composition against the detailed Model message; retain verifiable input hashes, commands and complete results before assertions.

Prewrite these public RED tests in a new test module; do not relabel the old O6 test that deliberately pinned its pre-correction text as the independent RED. Update that obsolete pin only in the later approved correction. Include the existing `-min` plate2 Native full-stream success control, preserve later outside-volume rejection, and run workspace nextest, strict Clippy/fmt/WASM and reference-enabled profile-entry gates before exact-hash review/integration.

## Beta

At OrcaSlicer v2.5.0-beta `345efec64` the throw is `Model.cpp:500-501` and its text ends with a period: "The supplied file couldn't be read because it's empty.". Ares follows the beta.
