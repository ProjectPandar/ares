# Empty G-code layer warnings and CLI refusal

## Upstream boundary

Source authority is OrcaSlicer
`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

- `libslic3r/GCode.cpp::collect_layers_to_print(const PrintObject&, bool)`,
  lines 2404–2448, detects unsupported height ranges using the last layer with
  extrusions. Lines 2457–2467 format a CRITICAL `SlicingEmptyGcodeLayers` warning.
- `libslic3r/Layer.hpp::LayerRegion::has_extrusions`, `Layer::has_extrusions`
  and `SupportLayer::has_extrusions` define extrusion presence.
- `libslic3r/Print.cpp::Print::process`, line 3662, invokes the collector before
  G-code export.
- `OrcaSlicer.cpp`, lines 7141–7172, consumes slicing warnings. In non-strict
  mode with checking enabled, a critical empty-layer warning causes
  `record_exit_reson` and `flush_and_exit` with `CLI_SLICING_ERROR` (-100).

The empty-first-layer exception at `GCode.cpp:2399–2401` is a separate behavior.
A generic CLI slicing error does not identify that exception. Intermediate
empty-layer warnings must not become core `SliceError::InvalidInput` errors.

## Rust destinations

Paths in this section are relative to `crates/ares-core/src`, except CLI paths.

- `project_slice/gcode_emit/layers/schedule/empty_layers.rs` owns the range
  detector and warning formatting. Its inputs are merged object, support and
  raft schedule records before print-wide Z grouping.
- `print/warnings.rs::SlicingNotificationType::EmptyGcodeLayers` represents
  source notification tag 3. The warning level is `WarningLevel::Critical`.
- `project_slice/gcode_emit/layers/schedule.rs`, `gcode_emit::orchestration` and
  `project_slice.rs` transport warnings through `ProcessedGCode.warnings` and
  `SlicedPlate.warnings`.
- `crates/ares-cli/src/failure/slicing.rs` owns the non-strict refusal policy.
  `failure/report.rs` writes the structured failure report. `main.rs` and
  `project_presets.rs` apply the policy before filesystem output.

Object, support and raft schedule records are compatibility representations of
upstream layer-collector concepts. This slice uses those representations without
introducing an independent slicing pipeline. Mesh topology and endpoint sorting
are outside this boundary.

## Included behavior

### Core range detection

Track the last record with extrusions separately for each object. Traverse merged
records once without rescanning the plate for every object.

Evaluate supportability when an object layer has extrusions or a support layer is
present. Empty support layers are eligible because support regions can exist
without support extrusion paths. Use object-layer height when both object and
support layers are present.

Round top and bottom support contact distances upward to multiples of the object
layer height. A configured bottom distance of zero uses the top distance. Select
bottom distance when the current record contains a support layer, otherwise top
distance. When the last extrusion record contains support, include the raw raft
contact distance in the maximum allowance. The rounded raft local in C++ is not
used by its maximum calculation.

The maximum supported Z is the previous extrusion Z, or zero when absent, plus
the current layer height and the nonnegative contact allowance. Record a range
only when the current record has extrusions and its Z exceeds that maximum by
strictly more than `2 * EPSILON`. Empty records do not advance last-extrusion
state.

### Warning and transport

Display the first five ranges using the existing six-significant-digit C++
ostream formatter. Each range contributes a line in this form:

```text
The object has empty layers between {from} and {to} and can’t be printed.
```

Append the object name and mesh guidance:

```text
Object: {name}
Parts of the object at these heights may be too thin or the object may have a faulty mesh.
```

Preserve existing support warnings alongside the critical gap warning. Preserve
warning snapshots on later export errors so the CLI can apply its pre-export
warning policy. A warning by itself does not abort core slicing or suppress the
API's G-code result.

### CLI policy

Apply refusal to plain-project, catalog/profile, loaded-preset and dump-moves
paths before writing G-code or moves. Apply it on an export-error path when the
plate snapshot already contains the critical empty-layer warning.

Report code -100, the active plate, the generic CLI slicing error, the complete
critical `warning_message` and `triangle_count=0`. Keep the critical warning out
of the noncritical warnings array. Preserve noncritical warnings, configuration
substitutions and upward-machine reports. The Unix process exit code is 156.

## Deferred behavior

- Belt-printer lead-in exceptions and belt-specific layer collection.
- Strict/no-check CLI flags and overlap-warning production.
- Mesh repair, welding, topology and endpoint-sort changes.
- Warning API redesign and complete intermediate-geometry equivalence.
- Web slicing-warning transport and notification UI.

`ares-wasm/src/moves.rs::SliceOutput::from_processed` does not expose
`ProcessedGCode.warnings`. Web `LoadWarning`/`loadWarning` notifications describe
configuration loading, not slicing warnings. This limitation does not change
core slicing success. Core logic remains independent of filesystem and UI
behavior on browser WASM, Windows, macOS and Linux.

## Failure modes and test contract

The E2E boundary is the public processed byte API and real CLI, compared with
Native OrcaSlicer using the same fixture bytes.

For `odd-rotated.3mf` and `odd-reverse.3mf`, require successful API G-code output
and an exact-message Critical tag-3 warning. The warning range is 0.4 to 1.8 and
the object name is `cube10.stl`. Require the Native warning to match the complete
literal independently of the Ares output.

For each fixture, exercise plain and catalog-aware CLI commands with and without
moves export. Require Native's corresponding process exit code, structured -100
failure and no G-code or requested moves output. Check the complete critical
warning text, report fields and preservation of noncritical warnings.

Retain the thirteen strict whole-output mesh controls, including `odd_original`,
and the noncritical support-warning success control. Fixture bytes, comparators,
operations and tolerances remain unchanged.

Tests must expose missing or incorrect warning tags, levels, range formatting or
object suffixes, lost warnings during output assembly, warning-induced core
errors, refusal after output files are written, adapter paths bypassing refusal,
critical warnings misclassified as noncritical, and spurious warnings on controls.
