# Reproducible generation inputs

For the same project bytes and options, Ares CLI/browser default slicing must
return the same complete output bytes. Core callers' explicit generation time,
plate and `SlicerContext` seed are part of the input. No output-field filtering
is allowed in the Ares-to-Ares reproducibility assertion.

Source boundaries: `OrcaSlicer/src/libslic3r/GCode.cpp:2549–2581` (header and
file-start timestamp formatting), `PlaceholderParser.cpp:78–101`
(`update_timestamp`). Destinations: existing `generation.rs`, CLI `main.rs`,
WASM `lib.rs`; reuse existing metadata and RNG owners. The adapter default
epoch policy is a user-requested deviation from upstream wall-clock capture,
not an independently designed pipeline or a claim of wall-clock parity.

Included: fixed Unix epoch-zero adapter metadata; every Ares-created test ZIP
uses explicit DOS epoch defaults, preserving original member order,
compression and content. Explicit timestamp input-variation tests are exempt.
Official exports and existing complete Orca comparisons remain unchanged.

Deferred: proving reproducibility for all printers/options/platforms/worker
counts, and correcting other nondeterminism only when reproduced. Acceptance
requires independent CLI processes and actual browser repeat slicing with
full-byte equality and an exact fixed timestamp assertion (not same-second
equality), plus existing KSR complete comparison and quality gates.

Follow-up after ff8a2517: remove automatic header generation dates entirely,
retaining generator/version and every other byte. Port boundary is the emission
at `GCode.cpp:2576`, Rust `gcode_emit/header.rs` and its sole caller; template
initialization at2557/2705 and `PlaceholderParser.cpp:78–101` stays present in
the existing Rust timestamp owner. Preserve the seven template variables and
epoch0/seed0 defaults. Supplied vendor/user templates are not rewritten.
Public CLI and actual browser must first fail the exact undated-header assertion,
then pass it and raw complete repeat equality. Public saved-project template
output must prove the retained time-variable contract. Strict full Orca parity
and all comparator mutations remain required; dated Ares headers are rejected.
