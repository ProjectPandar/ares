# Postprocessed filament statistics rewrite slice

Source: upstream `GCode.cpp::update_print_stats_and_format_filament_stats`
and `GCode/GCodeProcessor.cpp::run_post_process` vector allocation and
`process_used_filament` replacement, as cited in ARD-0071.
Destination: core `project_slice/gcode_emit/finish.rs`, existing combined
statistics shell only. Include complete configured-width weight/cost vectors
when writer-side gates produced those lines. Defer multi-tool/wipe-tower
accounting and unrelated timing/configuration differences.

Exit: minimized actual Orca full-stream RED/GREEN at the public byte API,
input activation proof, writer-side line-omission regressions, complete
retained artifacts, workspace nextest/fmt/Clippy/platform gates, independent
six-axis review and repairs, code/docs committed and pushed together.
