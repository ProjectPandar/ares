# Compatible WIPE tags: upstream boundary and API contract

Status: Accepted (WIPE scope and runtime CLI/API boundary).
Upstream: OrcaSlicer `8500fcdcc`, `src/libslic3r/GCode/GCodeProcessor.{hpp,cpp}`.

## Source conclusions

| Source | Required behavior |
| --- | --- |
| `GCodeProcessor.cpp:2618-2633::process_file` | OrcaSlicer/Slic3rPE/Slic3r input loads embedded configuration; nonempty printer_model sets s_IsBBLPrinter from its Bambu Lab prefix |
| `GCode.cpp:2047::export_gcode` | Export assigns the same flag from print->is_BBL_printer() |
| `GCodeProcessor.hpp:375::reserved_tag` | Flag selects Reserved_Tags or Reserved_Tags_compatible |
| `GCodeProcessor.cpp:58-61,81-84` | BBL WIPE tokens have a leading space; compatible WIPE tokens do not |
| `GCodeProcessor.cpp:3146-3155::process_tags` | Prefix-match the selected token; recognized START/END changes m_wiping; unrecognized comments leave it unchanged |
| `GCodeProcessor.cpp:3835-3851,3895-3911` | Wiping precedes ordinary motion classification and does not enter ordinary extrusion filament accounting |

Ares `processor/estimate.rs:76-78` accepts both WIPE spellings regardless of
`limits.bbl_printer`, causing a deterministic processor-result difference.
Only the **non-BBL** branch is corrected: preserve comment-leading spaces,
recognize compatible WIPE_START/END prefixes, and ignore BBL-spaced forms.
State changes precede motion recording. An unrecognized END cannot clear an
active compatible wipe. Prefix suffixes retain Native starts_with semantics.

BBL behavior remains unchanged. Deferred: BBL-side symmetric strictness, other
reserved-tag dialects, indentation before semicolons, foreign-producer detection,
tower/flush tags, role parsing and unrelated motion/timing precision. Tower tags
have spaces in both Native tables (`cpp:74-75,97-98`) and are outside this slice.

## Rust destination and public processor boundary

WIPE correction belongs to `ares-core/src/project_slice/gcode_emit/processor/
estimate.rs::Estimate::record`, with necessary parsing in existing
`processor/moves/tags.rs`. Estimate is already over 400 lines; do not grow an
independent parser there. Existing Estimate/MotionState remains a temporary
compatibility shell around upstream GCodeProcessor. No second processor loop,
role-cache formula change, new configuration or independent timing model.

There is no current public arbitrary-G-code processor entry; public
slice_project_plate_processed accepts project geometry. The chosen
CLI E2E boundary requires minimal normal API exposure in the already-public
`ares_core::gcode_processor` module:

```rust
pub fn process_gcode(
    gcode: &str,
    settings: &ProjectSettings,
    bbl_printer: bool,
) -> ProcessedGCode
```

This platform-neutral API delegates to the existing processor, enables move
recording, disables progress injection and adds no external loading delay.
Typed settings supply existing metadata/machine configuration; reuse the
existing envelope constructor (`GCodeProcessor.cpp:2087-2122`). The explicit
printer flag is caller context, matching `GCode.cpp:2047`, never inferred from
WIPE comments. UTF-8 str input needs no filesystem/parser fallback. Private
processor configuration types remain private; no cfg(test) API or backdoor.

The owning upstream boundary is `GCodeProcessor.hpp:887::apply_config`,
`:905::process_file`, `:914-915::process_buffer/finalize` and
`GCodeProcessor.cpp:2025,2592,2677-2709`, exposed through the existing
postprocessing shell. API exposure must preserve the current WIPE bug until
the separately observable behavioral RED; it does not absorb unrelated options.

## Native E2E and observable acceptance

Native-dependent tests belong to `ares-cli/tests/orca_parity/runner/wipe_tags.rs`.
Each test directly calls OrcaRunner to generate/validate its Native reference,
then invokes the public core API with identical input/configuration. No core
Native-dependent tests, cache-prewarming preflight or test-order dependency.

OrcaRunner uses its existing content-keyed cache, locking, staged publication
and command capture. Identity includes input bytes, actual Native ELF,
reader/launcher recipe and producer context; hits validate output SHA receipts.
Only test inputs and test/probe code are committed. Native maps, move CSVs,
metadata/result JSON and stdout/stderr stay in ARES_ORCA_CACHE_DIR/runtime
artifacts. Main's old Native products belong to a separate cleanup slice.

The only slicing input is the existing
`tests/parity/validation/kobra3-centered/0.8.3mf`. Each CLI test requests a
runtime slice through OrcaRunner, then deterministically transforms comment
tokens in that Native G-code. All motion commands and line counts remain
unchanged. Cases cover spaced START/END, a spaced END after compatible START,
compatible prefix suffixes, unchanged compatible control, and canonical BBL
control (role tags and embedded printer_model select the BBL dialect).
No Native G-code, transformed G-code or extracted expected result is committed.

The same runtime variant is supplied to Native process_file and public core
processing with the input project's settings. Compare complete role key sets
and every f64 pair bit, then the selected Native CSV motion type. Select the
first motion inside a WIPE region, or immediately after its END for the ignored
END case; never assume a frozen output line number. Runtime receipts link 3MF,
Native source G-code and transformed-input SHA. Native enum values come from
GCodeProcessor.hpp:30-44, not libvgcode ordinals. Differences in ordinary
travel versus Wipe are observable even when filament-role totals are equal.

The existing ELF is pinned by SHA256
`59f9ea7e16165df6979a243de7d6acd49d8cf523ac5de6d3fec08d6e25c7896e`.
Use nix shell nixpkgs#gdb on that binary without rebuilding Native. The already
validated map reader observes s_IsBBLPrinter read-only; the test checks it
against input dialect. Unequal-material/flush process-mode maps intentionally
have no footer-total equality assertion. The prior same-process slice/footer
validation establishes extraction reliability; rounded totals cannot substitute
for complete Native role results.

Required controls cover compatible spelling, compatible START followed by an
ignored spaced END, compatible prefix suffixes and canonical BBL input. All
Native references are generated at runtime; no manually reconstructed expected
map or masked difference is accepted. A missing API/reference is not the WIPE
behavioral RED. Corrected cases must match complete Native results, while BBL
and unaffected inputs remain invariant. Any non-BBL timing/M73/footer change
must be explained by corrected WIPE state; normal G-code comparison permits
only generated-by normalization.

Acceptance requires a separately observed Native behavioral RED before the
WIPE correction, directed controls, fresh paired seeded/official comparison
(candidate failures a subset of base, all transitions explained), workspace
nextest, strict clippy, fmt and wasm32. Evidence and execution history remain
outside this specification.
