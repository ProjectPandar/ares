# CLI error for a missing project plate

## Upstream contract

At OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`),
`CLI::run` rejects a selected plate beyond `PartPlateList::get_plate_count()`
(`src/OrcaSlicer.cpp:5291–5295`; the `--repetitions` branch rejects it earlier
at `:4947–4952`). This is `CLI_INVALID_PARAMS` (`src/libslic3r/Utils.hpp:44`),
not an input model read failure or an empty existing plate. It records plate
index `0` and the reason `Invalid parameters to the slicer.` (`:196,535–586`),
then exits without exporting G-code. The Linux exit macro (`:487–495`) emits
`run found error, return -2, exit...` on stdout and exits with signed `-2`
(POSIX status `254`). `record_exit_reson` writes `result.json` on Linux only.

On `tests/parity/custom-gcode-overview/two-plates.3mf` with `--slice 3`, both
the official and seeded CLI emit the same stdout, no stderr and no G-code.
The result contains `return_code: -2`, `plate_index: 0`, the reason above,
zero slicing parameters, false `strict_mode`, and an empty `warnings` list.
The observed `prepare_time` and `export_time` vary between Native runs; the
existing comparator normalizes only timing fields, not the reason or status.

## Rust boundary

`ares-core/src/project/domain/plate_selection.rs:13–20` already rejects an ID
that does not exist. Return a dedicated `SliceError` carrying the requested
plate ID instead of `InvalidInput`. Preserve the human-readable core message.
`ares-cli/src/failure.rs::slice_error_to_failure` maps only that variant to the
existing `CliFailure::Family` reporting path with the Native code and result
metadata. No matching on diagnostic text and no change to generic `InvalidInput`.

This ports the positive missing-ID rejection only. Valid selection, existing
but empty plates, malformed files, other option errors, and the CLI's
single-plate interface (`--slice 0` / all plates is out of scope) do not change.

## Witness

Use the committed two-plate project unmodified. Compare a fresh Native
`--slice 3` invocation to Ares `slice --plate 3`. Assert the Native status,
stdout and result anchors, both channels and full timing-normalized result
bytes, and absence of G-code on both sides. Before the fix, Ares rejects the
same request with status 1 and no result report.
