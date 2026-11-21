# Single 3MF import for CLI preset slicing

## Source and destination

OrcaSlicer v2.5.0 `OrcaSlicer.cpp:1755–1810` reads the model and config together (`Model::read_from_file`, `libslic3r/Model.cpp`, dispatching to the BBS 3MF importer in `libslic3r/Format/bbs_3mf.cpp`). After the file compatibility decisions, the same block rejects a nonempty embedded `post_process` vector when `normative_check` is true (`:1799–1810`). Supplied config files are read later (`:1953–2024`), and preset composition is later still. The importer owns the embedded config; the command owns the decision not to accept scripts.

`ares-cli::{load_settings,project_presets}::slice` currently calls `ares_core::validate_load_settings_input` before reading preset files. That temporary adapter around `load_project` drops the entire imported `Project`. `profiles::entry` then builds a `ProjectSource` whose `project_slice` loading stage imports the same bytes again, either for selected-profile overlay or project preset refresh.

Move the CLI import and embedded admission check into the core slice entry, with the CLI preset reads deferred until admission succeeds. Move the imported project into the existing overlay or refresh stage. Reuse the existing unconditional CLI check, `project_slice::validate_post_process`, on the embedded view. This preserves the behavior equivalent to upstream's `normative_check=true`, without adding a context field or a CLI option. Remove the standalone preflight API. Keep the adapter responsible for file I/O and CLI reporting. Opening the archive to read specific metadata is not a second model import.

Included: `--load-settings`/`--load-filaments` and project preset catalog slicing, with unchanged failure precedence and reporting. Deferred: adding a CLI switch to disable normative checks, changes to existing version compatibility behavior, plain 3MF routing, script execution, and new preset or plater semantics.

## Failure modes to preserve

- Argument, override, and input-file failures already handled before dispatch must still win. The load-settings filename check stays before admission; the catalog route checks the extension before admission but checks the filename after reading the catalog. Non-3MF bytes must not be admitted as 3MF based on a suffix or rejected earlier by an unrelated import.
- A malformed 3MF must fail at the original import boundary even with missing or malformed presets. Do not check script metadata before the original import finishes: a later invalid package member or model must still win. Preserve the ordinary import's settings validation and its error order; it materializes retained defaults while the overlay import keeps sparse raw settings. Do not feed filled defaults into the overlay or change which errors are reported before or after reading presets.
- Embedded `post_process: [""]` and nonempty scripts must reject before catalog, preset/source reads, overlay, machine compatibility, plate selection or print validation. Absent/empty values still proceed. Do not change any existing validation of the final configuration or let a later override hide the embedded array.
- Once admission succeeds, keep the two adapters' existing read/parse order, including the catalog route's original upward-machine identity lookup. Preserve initial versus later error mapping: no substitution text, upward-machine snapshot, progress, or warnings on an admission failure. Keep the same exit code, stdout, stderr, result.json fields, and no G-code or moves output on failure.
- Move the complete imported mesh and its preparation data, without reimporting or deep-copying it, so import scope, placement and flush-matrix inputs are unchanged. Do not change existing byte-oriented core, WASM or web callers that do not opt into this CLI admission phase.

## Verification

Run the existing public CLI and core cases for `post_process`, normative admission, load-settings and project presets before and after; compare the affected Orca parity cases in both reference modes. In particular, the competing missing preset versus embedded array case must still return `-19` with the same report and no output. Count model imports on both routes with the same large synthetic 3MF before and after; require two to become one on admitted inputs. Do not add a timing assertion to the test suite.
