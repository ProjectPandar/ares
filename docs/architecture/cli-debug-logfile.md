# CLI diagnostic level and destination

## Upstream boundary

OrcaSlicer v2.5.0 (`40ecc66f7`) defines `debug` and `logfile` in
`src/libslic3r/PrintConfig.cpp:13160–13171`. `src/OrcaSlicer.cpp:1524–1541`
configures the CLI log after parsing; `src/libslic3r/utils.cpp:160–200` maps the
level to a Boost severity filter and registers the file sink. The existing
triangle-limit diagnostic comes from `src/OrcaSlicer.cpp:6637–6641`.

## Contract

- `--logfile PATH` replaces the default destination, `00000.log`. An omitted or
  empty path selects that default. Relative paths are resolved from the process
  working directory, not from `--outputdir`; absolute paths are used directly.
- The sink opens and creates missing parent directories only when a message
  passes the filter. A preexisting file at the destination is truncated on the
  first write, not when the CLI parses the options.
- The configured default level is 1 (error). `--debug 0` admits only fatal
  messages, so the triangle-limit error creates neither a log nor its parent
  directories. Levels 1 and more verbose admit that error. The filter does not
  change the exit code, stderr or slice report.
- The triangle-limit record is the message and a newline, without a timestamp
  or severity prefix: `plate 1: triangle count <count> exceeds the limit:<limit>`.

The Rust boundary is the direct CLI adapter in `ares-cli`. It routes the
existing diagnostic using the triangle count and limit returned by `ares-core`;
no core or WASM logging API is needed. The slice failure and its report remain
independent of the diagnostic sink.

Release Orca emits additional warning and info records at levels 2 and 3. Those
and all other records Ares does not yet emit (including debug, trace and fatal)
are outside this change. Boost filename patterns, rotation, prerelease filter
overrides and failures opening the sink are also outside this contract.

Tests invoke both executables with identical arguments and inspect the exit,
stderr, complete working-directory inventory and exact log bytes. The Native
cases that produce a log use the default or error level and contain only the
triangle-limit record. Fatal-only and successful export cases produce no log.
