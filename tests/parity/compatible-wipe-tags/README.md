# Compatible WIPE runtime cases

Boundary: `docs/architecture/gcode-processor-compatible-wipe-tags-spec.md`.
Input: existing `tests/parity/validation/kobra3-centered/0.8.3mf`.

Each CLI E2E directly calls OrcaRunner to slice the 3MF at runtime, using
its content-keyed Native cache. No generated G-code is committed here.
The test preserves all motion commands and changes only WIPE comment tokens
(spaced START/END or compatible prefix suffixes). The BBL control changes
role tag spelling and embedded printer_model as well. Source line count is
preserved; selected motion IDs are found from runtime WIPE tags.

The transformed G-code is processed by both the existing Native binary under
`nix shell nixpkgs#gdb -c gdb` and the public core processor API. Processor
references use OrcaRunner locking/staged publication and SHA-validated caches.
Tests compare complete Native role key sets, every f64 pair bit and the
selected motion type (`GCodeProcessor.hpp:30-44`). Maps, CSV, metadata,
input variants, SHA receipts and command captures remain in runtime artifacts.
There is no preflight or dependency on test order.
