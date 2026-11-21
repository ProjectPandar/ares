# Direct CLI: consume slicing warnings in emission order

## Source and destination

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`):
`libslic3r/PrintBase.hpp::PrintBase::SlicingStatus` carries warning step, type,
level and text. `OrcaSlicer.cpp:456–472` appends callback statuses with a warning
step; `OrcaSlicer.cpp:7142–7178` consumes them in arrival order for the current
plate, skipping default notifications. The Rust boundary is `SlicedPlate.warnings`
(`ares_core::PrintWarning`) and its direct CLI consumer in
`ares-cli/src/failure/slicing.rs`. Keep the existing typed core producers; the
separate `check_strict` and `check_direct` adapter passes are the temporary
approximation being replaced. The Rust warning type already excludes progress
and default notifications. Call the same ordered consumer from `direct_project.rs`
and `direct_stl.rs`, before their output writes. Do not
change the `ares slice` subcommand's policy or add new warning producers.

## Ordered consumption

For each eligible notification, update the plate warning to its text and then:

- Noncritical: log `plate {id}: found NON_CRITICAL slicing warnings: {text}` and
  record the `slicing_warning_non_critical` entry. With `--strict`, stop here.
- Critical: log `plate {id}: found slicing warnings: {text}, no_check={0|1}`.
  Stop only for `EmptyGcodeLayers` or `GcodeOverlap` with checking enabled.
  Critical warnings do not become noncritical report entries.

The first stopping notification wins; preserve only noncritical records through
that notification, inclusive, and use the stopping text as `warning_message`.
Without a stop, the last eligible text remains the plate warning. Emit warning
diagnostics via the existing file log, not stderr. Preserve preset substitutions,
upward machine reports, and the existing strict/no-check flag parsing.

Both warning stops use `CLI_SLICING_ERROR` (`libslic3r/Utils.hpp:92`, result
`-100`, POSIX status `156`). Their precedence affects report and log contents,
not the exit code. A failure retains the current one-based plate ID and effective
settings, but leaves its triangle count zero and does not write its plate G-code
(`OrcaSlicer.cpp:7155–7172,7180–7193`). Today the strict prepass can select a later
noncritical notification before an earlier rejectable critical one. Independently
of that hypothetical order, both existing paths omit the noncritical log record.

## Paired regression

Generate an input-only 3MF with a single named, watertight object mesh on one
plate, in a fixed-bed, single-extruder profile. Use a grounded cantilever (a
4 × 10 mm stem, 3 mm tall, with a centered 20 × 10 mm cap extending to Z=5 mm)
and a second closed component of the **same mesh** over its footprint
(8 × 8 mm, Z=9–11 mm). Keep supports, enforced support layers, raft and prime tower off;
use 0.2 mm layers and a support top clearance smaller than the 4 mm gap. The
cantilever supplies a noncritical support warning independently of the gap; the
upper component resumes extrusion after several empty object layers. Using two
separately placed objects would not exercise the same per-object gap check.

Sources for the proposed fixture: `libslic3r/PrintObject.cpp:961–986,5289–5303`
(`PrintObject::generate_support_material`, `is_support_necessary`), called by
`Print::process` at `libslic3r/Print.cpp:3581`, before that function's call to
`GCode::collect_layers_to_print` at `:3662`. The latter emits the gap warning at
`libslic3r/GCode.cpp:2426–2467`. In Rust, `project_slice.rs:271–297` collects
support necessity before invoking the G-code layer consumer; the latter appends
in `project_slice/gcode_emit/layers/schedule/empty_layers.rs`. Thus the two
currently produced types naturally exercise noncritical **before** critical, not
its reverse. For example, the later upstream Precise Seam warning
(`libslic3r/GCode.cpp:4496–4503`) is not an Ares producer. Do not reorder the
fixture artificially or add a producer merely to reach the reverse order.

Before changing the consumer, verify both producers fire on this same input in
an ordinary checked Native run, and that the paired regression fails on the
missing/misordered diagnostic sequence, not on unrelated slicing or validation:

| Direct CLI action with `--debug 2 --outputdir out` | Ordered plate diagnostics | Stopping plate warning |
| --- | --- | --- |
| `--slice 1` | Support, then empty layers | Empty layers |
| `--strict --slice 1` | Support only | Support |

Paired runs use identical argv and retain input, output inventory, process
status/stdout/stderr, result JSON and logs for Native and Ares. Both actions
must exit `156`, leave stderr empty and write no `plate_1.gcode`. Compare the
whole result JSON using the existing elapsed-time normalization; require the
earlier support record in both runs. Compare the ordered post-slice plate warning
messages from the file logs, matching complete messages including continuation
lines, not log metadata. The two status/stderr pairs alone cannot distinguish
these warning branches. Keep the existing strict, checked empty-layer and
`--no-check` action-order cases in the affected paired set, in both reference
modes.

## Adjacent behavior

`OrcaSlicer.cpp:6988,7134–7138` passes the action's `no_check` into `Print` and
rejects nonempty `get_conflict_string()` before consuming warnings, using
`CLI_GCODE_PATH_CONFLICTS` (`-101`) and `index+1`. Direct single-STL and project
paths already have a pre-warning conflict check. The arranged STL entry
(`ares-core/src/profiles/entry/load_settings.rs::slice_stl_models_with_load_settings`)
reconstructs the per-plate `SlicerContext` with only the triangle limit and custom
G-code, losing the action's `no_check`; the arranged branch of `direct_stl.rs`
also does not request the path-conflict check. Errors from that entry currently
lose the failing plate's ID and report at the CLI boundary. Merely enabling
conflict detection would not port the indexed failure or earlier-plate output
semantics. Defer arranged-STL context propagation, indexed processing failures
and per-plate traversal/export timing to a separate slice. No change to the
existing processing-time conflict precedence is needed for this warning pass.
