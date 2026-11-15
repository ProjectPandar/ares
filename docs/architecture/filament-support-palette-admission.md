# Filament support/palette admission

## Source boundary

OrcaSlicer v2.5.0, commit `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`:

- `src/OrcaSlicer.cpp:3929–3933` rejects unequal `filament_is_support` and project `filament_colour` lengths during flush-matrix preparation, records `CLI_CONFIG_FILE_ERROR` with plate index zero, then returns that signed code.
- `src/libslic3r/Utils.hpp` defines `CLI_CONFIG_FILE_ERROR = -5`.
- `src/OrcaSlicer.cpp:193–243,487–503,535–586` owns its generic reason, stdout, signed return, and Linux result.json reporting.

This is an adapter slice around the existing Rust rewrite of upstream flush-matrix preparation, not a separate slicing algorithm.

## Rust boundary

`ares-core::options::flush_matrix` already detects this mismatch. Replace only that occurrence of generic `SliceError::InvalidInput` with `SliceError::FilamentSupportPaletteMismatch`, carrying support and palette counts. Its Display retains the existing count detail for API consumers.

`ares-cli::failure::CliFailure::config_file_error` provides a reusable Family -5 report constructor: the upstream generic reason, plate index zero, zero slicing metadata, and caller-supplied stderr. It does not classify errors. `slice_error_to_failure` maps only this source-specific variant to that constructor with empty stderr. POSIX exposes status251; result.json retains -5. Non-Linux stdout and return behavior remain owned by the existing adapter.

The core remains platform-neutral: no process exit, filesystem access, or terminal reporting. Other InvalidInput and I/O errors do not acquire this mapping. Any module extraction needed for the file-size limit preserves the existing function path and behavior.

## Included and deferred

Included: retaining this one error source and its existing detail, plus CLI exit/stdout/stderr/Linux result.json/no-G-code parity.

Deferred: duplicate/type/from admission, missing preset files, unresolved preset inheritance, palette overlay semantics, dataset coverage, and the ordering or calculation of flush-matrix preparation. Other CLI error families are unchanged. The remaining generic error mapping is not a fallback for this source.

## Verification

A public CLI E2E generates its project at runtime from `tests/parity/classic-layer-id/m-later-owner.3mf`, changing only `filament_colour` in project_settings from one color to two while preserving the one-entry support vector and all other archive members. No binary fixture or Native output is committed.

Run pinned Native and Ares with equivalent explicit palette inputs. Compare the actual process status, stdout, stderr, complete result.json through the existing timing-only comparator, and G-code absence. Persist the generated input, channels, exit status, Native receipt, and result files. An unexpected Native return or product inventory fails the test.

Compare the scoped cli_exit_channels, load_settings, and cli_result cases before and after the implementation in both Native modes, retaining existing failures separately from regressions. Keep the mechanical module extraction independently inspectable from the behavior change.
