# Optional project filament refresh: vendor-discovery miss

## Boundary and revision

Port the Native CLI disposition at `OrcaSlicer.cpp:2774–2816`, reached through
`PresetBundle.cpp:403–482 find_preset_vendor` and `:724–731 resolve_system_preset`,
at revision `345efec64c7b95a05ce5cf96df0dde66a158835c`.

**This explicitly revises the earlier required-missing rule:** an unresolved
optional `from_project` filament system-preset refresh is not an unavailable
explicit replacement selection. When the supplied original system catalog has no
matching system preset, keep that imported filament slot unchanged and continue
without adding a refreshed filament config. Explicit replacement selection still
errors for a missing requested preset/parent. A malformed supplied document or
unresolvable parent of a found document is not silently converted into this miss.

The preceding rename-resolution premise was withdrawn. Native CLI vendor discovery
matches original manifest names exactly and returns false on the witnessed old
name before `PresetCollection::find_preset` can consult `renamed_from`. Do not port
that collection-internal rename path as an implementation of this CLI route.

## Actual source flow

- `OrcaSlicer.cpp:1733–1758,1818–1895,1994–1996` imports the 3MF config and records
  its slot identities/inheritance/dirty keys before optional refresh.
- `PresetBundle.cpp:403–482` finds no vendor for the witnessed old name;
  `:724–731` returns false with `No vendor lists the preset`.
- `OrcaSlicer.cpp:2790–2816` logs a warning and continues the slot loop before
  inserting into loaded-filament names/config/index/refresh vectors.
- Ordinary later default synchronization (`OrcaSlicer.cpp:4253–4259`) is unchanged.
  An absent flow value may become its registered default1
  (`PrintConfig.cpp:2921–2931`); the miss does not load a new default preset.

## Rust destination and non-negotiable behavior

`crates/ares-core/src/profiles/project_refresh/preset.rs` owns the per-slot optional
lookup. Represent a vendor-discovery miss by the existing absent refreshed-config
slot, consumed by `project_refresh/filament.rs`. This replaces the overly strict
error in the temporary refresh shell with the upstream CLI disposition. It is not
an independent pipeline or a fallback design.

1. Keep imported values, identity, vector lengths/order and nils for the missing
   refresh slot; do not apply canonical renamed documents or seed preset defaults.
2. Preserve normal application of genuinely found system presets and existing
   explicit replacement selection errors.
3. Do not catch arbitrary request/load/parent errors. No retry as a bare request,
   file/environment lookup, name guessing, allowlist, fixture exception, synthetic
   catalog document, exporter patch, or tolerance change.
4. Native warning text for the witnessed miss is
   `CLI: system filament preset 'Creality Generic PLA' not resolved (No vendor lists the preset); filament 1 keeps its values`.
   Current byte/settings APIs return `Result` and have no warning output channel;
   this slice must not print from core or invent terminal/logging behavior. If an
   existing adapter exposes this warning, preserve Native's wording there.
5. ROOT registration, original fixtures, assertion bodies, Native preconditions,
   and comparators remain unchanged. New regression assertions are additive.

Included: optional filament system-preset miss, imported-slot retention, and public
request qualification. Deferred: machine/process miss dispositions, parent/load
failure policy beyond an absent system document, the withdrawn rename lookup,
validation/preflight ordering, adapter protocol conflicts, replay/scarf admission,
and velocity/SYNC body/metadata diagnosis. Each deferred slice keeps its own owner
and review; no blanket exemption is created by this change.

## Native-first public RED and acceptance

Use the unchanged real fixture
`tests/parity/arachne-alternate-extra-wall/spiral-off.3mf`, the off leg of
`arachne_alternate_effective_spiral_control`, SHA256
`acac29bab7d4a4c56e34acd312b004aff8d0a8da0023247bae96964e68732ee5`.

Supply original pinned vendor documents to both Native and Ares. Native slicing
runs twice first and must complete exit0, retain old filament identity and authored
flow0.98/diameter1.75, omit both dynamic footer keys, and log the unresolved-system
warning. Then the real public `slice_with_profiles_validated` request with
`from_project=true` must produce complete ordered output equal to Native using the
existing generator-only comparator. Before the port, the qualified RED is the
runnable `system preset 'Creality Generic PLA' was not supplied` request error.
Setup/compilation failures and the withdrawn rename premise are not this RED.

After the literal port, run this witness both modes, unchanged explicit-selection
and required-parent controls, and all owned directed cases. Regenerate the
mechanical wiring last on the accepted logic/dependency tree. Both entire present
inventories must be rerun. Landing remains HOLD until there are zero prior-PASS
regressions or an approved owner and explanation for every remaining regression;
ownership alone is not an exemption.
