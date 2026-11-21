# CLI `--allow-newer-file` saved-project admission

## Upstream boundary

This slice ports the BBL 3MF version admission performed by the OrcaSlicer
CLI in the pinned OrcaSlicer v2.5.0 source:

- `OrcaSlicer/src/OrcaSlicer.cpp:1613-1615` reads the boolean
  `allow_newer_file` option from the parsed CLI configuration.
- `OrcaSlicer/src/OrcaSlicer.cpp:1758-1779` performs the admission after a
  BBL 3MF has been loaded. With the default false value, it rejects only when
  the current CLI major is less than the file major, or when majors match and
  the current CLI minor is less than the file minor. The rejection uses
  `CLI_FILE_VERSION_NOT_SUPPORTED` (`Utils.hpp:67`, value `-24`), whose public
  diagnostic is `Unsupported 3MF version. Please make sure the 3MF file was
  created with the official version of Bambu Studio, not a beta version.`
- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:13247-13250` defines the option as
  a boolean with default false.
- `OrcaSlicer/src/libslic3r/Format/bbs_3mf.cpp:1540-1550` chooses the file
  version: an `OrcaSlicer` metadata tag takes precedence over the
  `Application` tag, and the latter is parsed from the `OrcaSlicer-` or
  `BambuStudio-` prefix. The actual major/minor comparison is in
  `OrcaSlicer.cpp`, not in `bbs_3mf.cpp`.

The option skips this one major/minor admission comparison only. It does not
skip archive parsing, metadata parsing, project-settings deserialization,
post-process admission, printer/process compatibility, config validation,
plate selection, print validation, slicing, or export checks.

## Ares destination

The Ares saved-project loader already extracts the effective BBL file version
from model metadata in `crates/ares-core/src/project/load.rs:235-264` and
stores it as `ProjectDocuments::preset_file_version`. The regular, catalog,
and explicitly loaded-preset CLI paths all receive the same 3MF bytes before
slicing. The current loader has no version rejection path: a newer effective
version reaches slicing instead of producing the Native `-24` family result.

Add the admission at the CLI saved-project boundary, before any route reads
external preset files or starts slicing. The core exposes a focused version
validation operation using the existing project loader and its effective
metadata version. The CLI maps the new error to the existing family failure
reporting machinery:

- POSIX process status `232` (the `-24` return code modulo 256);
- stdout `run found error, return -24, exit...\n`;
- the Native diagnostic above in `result.json.error_string`;
- empty stderr and no G-code output;
- the existing result fields remain zero/empty because rejection precedes
  plate setup.

The CLI option is one new `SliceArgs` boolean field named
`allow_newer_file`, with clap's long token `--allow-newer-file`, and no
reordering or alteration of existing fields. The default is false. The
explicit `--allow-newer-file=1` form is accepted by clap and is the argv form
used for the positive comparison.

The comparison uses only major and minor components, matching Native. Missing
or non-BBL metadata does not activate this admission. A valid version equal to
or older than the Ares compatibility version is unaffected by either setting.

## Verification contract

The public CLI parity cases use one valid saved project and modify only its
stored effective application/version metadata for the newer-version witness.
Native receives `--slice 0 --outputdir <dir> input.3mf`; Ares receives the
existing wrapper mapping `slice -o plate_1.gcode input.3mf`. The flag token is
identical in both invocations.

The cases assert all observable channels:

1. A newer version without the flag is rejected by both implementations with
   the same process status, stdout, stderr, complete `result.json`, and no
   G-code.
2. The same newer-version bytes with `--allow-newer-file=1` are admitted by
   both implementations and produce identical output channels and G-code.
3. A project at the compatibility version has identical successful output with
   and without the flag.

The paired checks run the saved-project loader and CLI/version-admission cases
in both official and seeded environments with the canonical reference
variables. Formatting, all-target clippy with warnings denied, and the WASM
check remain required. The fixture must not rely on environment-specific
output or undefined Native behavior.

## Deferred behavior

This slice does not implement `--allow-mix-temp`, any other CLI flag, newer
3MF format-number admission, or compatibility changes to adjacent project
constraints. Those remain separate source boundaries.
