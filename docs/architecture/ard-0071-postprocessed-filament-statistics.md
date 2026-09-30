# ARD-0071: Postprocessed filament statistics vectors

Upstream boundary (8500fcdcc): `GCode.cpp:2314-2373`
`update_print_stats_and_format_filament_stats` gates weight/cost lines using
writer-side positive usage; `GCode/GCodeProcessor.cpp:993-1008,1148-1175`
`run_post_process` replaces each existing line with a complete configured
filament-count vector, including unused trailing filaments as `0.00`.

Rust destination: `project_slice/gcode_emit/finish.rs` statistics emission.
Its combined writer/processor scaffold is a temporary compatibility shell,
not an Ares-owned pipeline. Preserve writer-side line presence, but format
present weight/cost lines at processor vector width. Keep accounting and
total sums unchanged. Multi-tool extrusion, wipe-tower accounting and the
processor's M73 timing behavior are deferred to their owning upstream slices.

Public seam: project bytes to complete G-code, with independently generated
Orca output. Use a small two-configured-filament project printing only the
first filament; confirm input activation and full ordered RED/GREEN. Retain
the original KSR failure and all exact input/output artifacts. No new output
normalization, captured numeric injection or source-structure pinning tests.

This slice cannot claim KSR/all-printer/all-option parity completion.
