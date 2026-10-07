# Generated printer-id materialization: separate upstream rewrite slice

## Status

Spec only, not implementation approval. Independent of X2D-FM (#55).
The retained generated-id witness is a pre-existing base failure, not an FM
calculator regression or a required FM GREEN dependency.

Upstream v2.4.2 revision `8500fcdccaa10b5099ac20d252af3a7c560046f1`.

## Owning source and Rust boundary

- `OrcaSlicer/src/libslic3r/PrintConfig.cpp:9004–9060`,
  `DynamicPrintConfig::get_index_for_extruder`: complete printer-id arrays versus
  ids derived from ordered `extruder_variant_list` groups, and variant matching.
- `PrintConfig.cpp:9634ff`, `update_values_to_printer_extruders`: per-key-set
  materialization, selection by the pre-update indices, stride and `get_at`.
  Trace the metadata arrays after each call before identifying the precise bug.
- `OrcaSlicer.cpp` call order of printer-variant-one and variant-two updates:
  preserve their source order and the configuration each later selector sees.
- Rust destination: `ares-core/src/options/project_variants/{index,printer,select}.rs`
  and `materialize_project_variants`. These existing compatibility shells must
  converge on the cited source, not introduce another Ares mapping authority.

Included: a source-discriminating public byte-input E2E for generated ids and
exact full-output equality after a narrowly proven materialization correction.
Deferred: FM arithmetic/predicate, dataset selection, filament mapping and
recommendation, UI transient-state behavior, emitter order, geometry and timing.
Do not add a new first-id/zero fallback or numeric patch. If the witness relies
on undefined source reads rather than generated-id semantics, stop and revise
this spec before implementation.

## Existing RED and admission requirement

Test originally introduced in `b39047a8884db752dbd702a4c67cbca763fc2cb3` as
`x2d_flush_matrix_incomplete_printer_id_map_uses_generated_variant_ids`.
Retained input `/tmp/ares-x2d-flush-matrix/green/generated-ids-legal-red/case-75aoF4`
has two nozzle types, two ids, four variants and two variant groups. Identified
Native succeeds; public `load_project`/`slice_project` on pre-FM base9e3 fails
with `invalid Orca option printer_extruder_variant`. Same-input adapter evidence
is `green/generated-ids-base-same-input.txt` (no repository debug additions).

Move the witness out of the FM acceptance module; preserve its old source and
artifacts. Once this spec is approved, bind a separate durable E2E module and
fresh base/main RED receipts before production edits. Establish whether metadata
reselection, incomplete ids or another source phase causes the error; do not
assert the currently suspected mechanism as fact.

## Exit

Separate spec, admitted RED and GREEN commits. Native provenance and complete
byte comparison, unchanged existing streams, workspace nextest, strict Clippy,
rustfmt and Tier1 checks. Generated-id correction cannot hold FM acceptance
hostage and FM cannot silently absorb generic materialization work.
