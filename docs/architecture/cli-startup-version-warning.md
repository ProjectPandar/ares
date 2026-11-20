# Direct CLI startup version warning

Upstream: OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`).

- `src/libslic3r/PrintConfig.cpp:13160–13171` supplies `debug = 1` and an
  empty `logfile` in the CLI config when omitted. Thus omission uses level 1,
  not the level 2 fallback for a missing config entry at
  `src/OrcaSlicer.cpp:1529–1530`.
- `src/libslic3r/utils.cpp:160–200` maps level to Boost severity and installs
  the file sink. `src/OrcaSlicer.cpp:1524–1538` configures these after early
  action checks, then emits one warning before input processing. On this
  release, `--debug >=2` admits `cli mode, Current OrcaSlicer Version 2.5.0`
  followed by a newline; omitted, 0, and 1 do not.

Port only that record through the direct CLI dispatch and `diagnostics.rs`,
after early option checks and before input I/O. Use the Orca compatibility
version constant and the existing logfile destination rules. Truncate an
existing file on the first emitted record; append later records in the same
invocation. No admitted record means no file or parent creation. The existing
CLI diagnostic writer remains the adapter for the cited `libslic3r` severity
and sink rules. The core remains I/O-free; stdout, stderr, G-code and slice
reports are not diagnostic destinations.

Compare Native and Ares using a synthetic STL settings export at levels 0,
1, 2 and with the level omitted. At level 2, also exercise a nested logfile
and a triangle-limit failure (`--mtcpp 11 --slice 0`) with both records in
order. Compare the inventory, exact log bytes and other public channels in
both reference modes. Retain existing triangle and destination controls.

Deferred: other Native records, prerelease/negative-level interpretation,
Boost filename patterns, rotation, and sink I/O error behavior.
