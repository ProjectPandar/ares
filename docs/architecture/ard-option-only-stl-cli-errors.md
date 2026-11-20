# ARD: Option-only STL CLI failure codes

This extends [the option-only STL print lifecycle](ard-option-only-stl-project-slice.md) at its error boundary, not at its slicing boundary. Source references are to OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.

| Failure | Upstream owner and path | Process status (POSIX) |
| --- | --- | --- |
| Relative extrusion without a layer reset | `libslic3r/Print.cpp:1972,2723–2766`: `Print::validate` reports the missing reset with the default `STRING_EXCEPT_NOT_DEFINED` type (`libslic3r/PrintBase.hpp:37–44`). `OrcaSlicer.cpp:6986,7009,7015–7040` applies and validates the print, writes `err.string` to stderr, and uses `CLI_VALIDATE_ERROR` (`libslic3r/Utils.hpp:72`, `-51`) in its default validation branch. | 205 |
| Truncated binary STL | `deps_src/admesh/stlinit.cpp:97–103,317–327` rejects a partial facet. `libslic3r/TriangleMesh.cpp:231–232` and `libslic3r/Format/STL.cpp:18–29` propagate failure; `libslic3r/Model.cpp:353–354,493–498` throws `Loading of a model file failed.`. The input reader (`OrcaSlicer.cpp:1758,1999–2002`) prints `<input>: <reason>` and returns `CLI_DATA_FILE_ERROR` (`libslic3r/Utils.hpp:48`, `-6`). | 250 |

`flush_and_exit` returns the signed code (`OrcaSlicer.cpp:487–502`), and `main` returns `CLI::run`'s result (`:8609`). The OS exposes its low byte on POSIX.

## Decision

On `ares slice --options <json> <stl>` without profiles, retain the distinction between a model-read failure and a `Print::validate` failure across `ares-core/src/project_slice/option_stl.rs` into `ares-cli`. Stop flattening classified validation errors on the CLI-facing path. Map a model-read failure to `CLI_DATA_FILE_ERROR` only on that path and reuse the existing CLI validation family. Neither parsing nor validation is run a second time. The current plain error on the CLI is a temporary compatibility behavior, not grounds for a fallback importer or an implicit reset. Keep the byte-only APIs, preset-backed STL and 3MF entry paths, JSON options admission, and CLI argument surface unchanged. Comparison of stdout and result JSON, other CLI admission families, and geometry or G-code changes are outside this decision.

## Public checks

Use an authored cube with relative extrusion and neither layer template providing `G92 E0`; use a binary STL with one declared facet and one byte missing from its sole record, supplying a valid reset so the configuration cannot account for the rejection. Compare live official and seeded OrcaSlicer against the CLI. Assert the status and absence of G-code. For stderr, select the single key diagnostic line in each process: the relative-extrusion message as-is, and the model-read message with only the operand's path replaced by `<input>`. Require the path prefix to name the actual input, and require each selected line to match in full. Ignore other lines (launcher or logging noise) and line terminators; do not strip error text, change punctuation, or translate diagnostics.
