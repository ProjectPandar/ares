# Empty destinations in ordinary filament slot assignment

## Source and destination

OrcaSlicer v2.5.0 (`40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`) resolves retained filament presets in slot order (`src/OrcaSlicer.cpp:2775–2817`), skips protected ordinary keys (`:3658–3698`), and calls `set_at(source, filament_index - 1, 0)` for each remaining ordinary vector (`:3737–3771`). `DynamicConfig::optptr` only supplies the default if the destination key is absent (`src/libslic3r/Config.cpp:1725–1745`). This leaves the empty-destination case outside `ConfigOptionVector::set_at`'s precondition (`src/libslic3r/Config.hpp:451–466`); [ARD-0114](ard-0114-empty-vector-set-at.md) records the defined Ares behavior.

In `crates/ares-core/src/profiles/project_refresh/filament/slot.rs::apply`, restore the option's registered vector default before growing an empty destination. Preserve existing behavior for missing and nonempty destinations. Grow using the first member of the destination, then assign source element zero to the requested slot. Do not change empty-source validation, keys kept by the project, per-variant restoration, expression groups, device vectors, project decoding, or the separate direct sliced export route.

## Verification

Build a synthetic saved project with multiple retained, resolvable filament slots and a bare empty `filament_notes` value. Enter unlisted refresh with notes unprotected, and ensure every notes slot is assigned from a nonempty preset vector; no slot in the result should retain padding from the undefined read. Compare G-code from the CLI `slice` route in both OrcaSlicer reference lanes.

Sliced 3MF archive parity is outside this verification: the direct sliced export route does not reach this refresh boundary.

If either Native lane crashes or produces inconsistent results on this input, record that outcome and verify the Ares invariant independently: CLI slicing succeeds and preserves the synthetic preset-authored notes in filament-slot order. Do not relax the comparator or treat an undefined Native result as an expected value.
