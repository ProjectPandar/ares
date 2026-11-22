# Preserve the bed and filament validation kind in the CLI

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`): `libslic3r/Print.cpp:2779–2806::Print::validate` returns `StringObjectException` with the `STRING_EXCEPT_FILAMENT_NOT_MATCH_BED_TYPE` kind (`libslic3r/PrintBase.hpp:28`) when a used filament has zero normal temperature for the selected bed. `OrcaSlicer.cpp:7012–7040` maps that kind to `CLI_FILAMENT_NOT_MATCH_BED_TYPE` (`libslic3r/Utils.hpp:82`, -61). It prints the detailed message to stderr and normally writes the error summary from `OrcaSlicer.cpp:231` to `result.json`; with `--no-check` it still rejects but writes the detailed message there instead.

## Rust boundary

Keep the existing guard in `ares-core/src/print/validate/bed_filament.rs`. Carry its kind through `SliceError::Validate` instead of treating the guard's result as a generic validation rejection. The existing generic error conversion is a compatibility shell for the upstream validation result; it must not discard this distinction. `ares-cli/src/failure/mapping.rs` selects the bed/filament code and sidecar summary by kind, not by inspecting the rendered message. Keep stderr, plate metadata, and the CLI's existing detailed-report selection unchanged. Other validation errors retain their existing classifications. The byte-oriented STL API retains its current error conversion.

Defer changes to the guard's eligibility, bed or filament selection, `Default Plate` handling, and CLI argument support.

## Verification

Use a synthetic first-plate project whose selected Cool Plate has a zero normal temperature but nonzero first-layer temperature for the used filament. For ordinary slicing and `--no-check` before `--slice`, compare the CLI exit status, stderr, output inventory, and complete result sidecar using the existing time normalization. A nonzero normal-temperature control must be accepted. Run both Native modes and the existing print validation and CLI result tests.
