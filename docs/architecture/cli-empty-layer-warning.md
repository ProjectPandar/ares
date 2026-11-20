# Direct CLI: checked empty-layer warning

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`):
`libslic3r/GCode.cpp:2459–2467` produces a critical `SlicingEmptyGcodeLayers`
notification for gaps in the printable layers. The existing Rust rewrite in
`project_slice/gcode_emit/layers/schedule/empty_layers.rs` already produces
`SlicingNotificationType::EmptyGcodeLayers`. Keep that producer and the core
API unchanged; this slice ports the direct CLI consumer of its notification.

`OrcaSlicer.cpp:7141–7175` copies the notification into the plate warning
and writes a diagnostic at warning verbosity, including the action-local
`no_check` value. With checking enabled it records `CLI_SLICING_ERROR`
(`libslic3r/Utils.hpp:92`, `-100`, POSIX exit `156`) and exits before setting
that plate's triangle count or exporting its G-code (`OrcaSlicer.cpp:7180–7193`).
The failed plate retains its warning, zero triangle count, and effective print
settings. Earlier noncritical warnings remain in `warnings`; the critical
notification itself is only the plate warning, not a noncritical record.
The diagnostic belongs to the log, not stderr.

When `--no-check` precedes `--slice`, retain the plate warning and log the
notification, but continue with the ordinary export and success report.
An occurrence after `--slice` is too late to affect the action
(`OrcaSlicer.cpp:6426–6444`). The flag does not disable the warning producer.

Apply the policy in `ares-cli` on the direct project and STL routes before
export, using the existing `failure::slicing` failure-report boundary and
CLI diagnostic log. Do not change the `ares slice` subcommand, the existing
strict treatment of noncritical warnings, or any other notification family.
Processing-time path conflicts and all-plate traversal are separate.

Exercise the direct CLI with the synthetic `mesh-edge-pairing/odd-rotated.3mf`
input. Run both executables with identical argv for each flag order. Compare
the process exit, stdout/stderr, result JSON (elapsed times only normalized),
G-code presence, and the empty-layer diagnostic record. Other log records and
G-code contents are not part of this comparison.
