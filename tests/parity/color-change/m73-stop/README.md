# Runtime M73 stop-time byte comparisons

Owner: OrcaSlicer 8500fcdcc, GCodeProcessor.cpp:5715 stop registration,
:576-579 elapsed update, :1098-1101 initial C and :1260-1293 next-stop formatting.

Only input overrides are committed here. CLI processor_migration/stops.rs applies
them to tests/parity/orca_cli_ender3.3mf for a runtime OrcaRunner slice, cached via
ARES_ORCA_CACHE_DIR. Public slicing still compares every M73 C/P/R line, complete
normalized length and all bytes; only the generated-by line is normalized.
A separate public process_gcode(post_process=true) replay in each case reverses
generated progress/C/duration fields and must reproduce all Native bytes.
Native reference/input/Ares products stay in runner runtime/cache directories.

- normal: SYNC before ColorChange/Pause can leave already-flushed stop IDs zero.
- stop-times: no SYNC, exercising the nonfinal last-minute C decimal format.
- zero-before: ColorChange before model use still requires the initial C0.

Native must actually emit C lines; no tolerance, footer-only comparison or
removed-event workaround is accepted. Required/optional Native availability
follows tests/support/native_environment.rs. web/e2e/colorChangeOracle.mjs
remains a runtime generator, not a committed oracle source.
