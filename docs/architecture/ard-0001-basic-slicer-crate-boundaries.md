# ARD-0001: Basic slicer crate boundaries

## Status
Accepted for M1.

## Decision
Ares starts with two Rust crates under `crates/`:

- `crates/ares-core` is the platform-neutral slicing API. It contains slicer data structures, option storage, and pipeline logic.
- `crates/ares-cli` is the command-line adapter. It owns argument parsing, filesystem reads and writes, process exit behavior, and user-facing errors.

`ares-core` must not perform direct file I/O. Callers pass already-loaded model bytes and options into the core API; adapters such as `ares-cli` decide where bytes come from and where generated G-code is written. This keeps the same core usable from browser WASM and native applications.

M1 preserves every provided OrcaSlicer option key and value dynamically instead of typing only a small subset. Typed option groups are deferred until profile parity work can map Orca semantics without losing unknown or not-yet-modeled keys.

Deeper crate splits are deferred. Geometry, model import, profile management, G-code emission, and WASM bindings remain inside the two-crate boundary until roadmap milestones prove that a split has a stable API and a real owner.

## OrcaSlicer structure evidence
The checked-in OrcaSlicer tree shows why Ares should begin with a narrow boundary:

- `OrcaSlicer/src/libslic3r` contains core slicing domains such as `Geometry`, `Fill`, `Support`, `GCode`, and `Format`.
- `OrcaSlicer/src/slic3r/GUI` and `OrcaSlicer/src/slic3r/Config` sit outside `libslic3r`, showing that UI/config surfaces are separate consumers of slicer logic.
- `OrcaSlicer/src/libslic3r/Format` contains file-format readers and writers, while Ares keeps filesystem ownership in adapters so the core can also run in the browser.
- `OrcaSlicer/resources/profiles` and dynamic config usage in `libslic3r` show that option compatibility is broad; M1 therefore stores all options without dropping unknown keys.
- `OrcaSlicer/tests` contains domain-level parity fixtures, so later milestones can add E2E comparisons against OrcaSlicer behavior.

## Consequences
- The first public API is small and stable enough for CLI and WASM callers.
- Unknown Orca options remain round-trippable through early milestones.
- Crate proliferation is avoided until import, profile, pipeline, or browser API work creates a concrete split point.
