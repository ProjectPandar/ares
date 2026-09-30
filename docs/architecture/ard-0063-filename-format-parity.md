# ARD-0063: Filename-format values in the native project parity path

Upstream owner: `libslic3r/PrintConfig.cpp:4967-4972` declares the free-form
`filename_format` string. `Print::output_filename` (`Print.cpp:3692-3721`)
passes it to `PrintBase::output_filename` (`PrintBase.cpp:64-91`) for recommended
output naming. `PrintBase::output_filepath` (`PrintBase.cpp:94-109`) preserves
an explicit file path without evaluating that template. The original CLI
supplies `plate_<n>.gcode` under `--outputdir` (`OrcaSlicer.cpp:6204-6215`).

Rust destination is the existing native `option_coverage/domains/probes.rs`
adapter and a separate filename-format regression module. Existing Ares
`ProcessPrintSourceOptions::filename_format` and config-header emission are
the production boundary exercised by the byte-output API. The fixture remains
a temporary adapter around the cited upstream config/export path, not a new
Ares pipeline.

Include empty, literal, and placeholder-bearing requested values. Prove exact
export application and final original config literals, retain each first
original/Ares complete ordered output, and verify the native original's
explicit output path remains `plate_1.gcode`. No output filtering, new fallback,
or template substitution is needed in the byte-output API for this CLI path.

Defer recommended filename generation (no path / directory path), timestamp
and print-statistics placeholders in names, and GUI file dialogs. Passing this
slice must not be described as implementing or testing those naming branches.
Free-form string probes are finite representative coverage, not enumeration
of arbitrary strings or global parity completion.
