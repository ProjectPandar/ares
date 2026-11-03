# Generated printer-id materialization: separate upstream rewrite slice

## Status

Implementation authorized after independent RED `e3e82da3155411dc87f0490686e3d91c6971d235`
and the source/Native trace ruling below. Independent of X2D-FM (#55).
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
- `PrintApply.cpp:1166–1169`, `Print::apply`: ordered printer-variant-one,
  printer-variant-two and process updates against the mutating full config;
  preserve the configuration each later selector sees.
- Rust destination: `ares-core/src/options/project_variants/{index,printer,select}.rs`
  and `materialize_project_variants`. These existing compatibility shells must
  converge on the cited source, not introduce another Ares mapping authority.

Included: a source-discriminating public byte-input E2E for generated ids and
exact full-output equality after a narrowly proven materialization correction.
Deferred: FM arithmetic/predicate and scaling, filament mapping and
recommendation, general UI transient-state behavior, emitter order, geometry
and timing. The narrowly admitted `Print::apply` variant-two negative-index
branch below is included, not a new first-id/zero fallback. If a witness relies
on undefined source reads rather than generated-id semantics, stop and revise
this spec before implementation.

## Existing RED and admission requirement

Test originally introduced in `b39047a8884db752dbd702a4c67cbca763fc2cb3` as
`x2d_flush_matrix_incomplete_printer_id_map_uses_generated_variant_ids`.
Retained input `${TMP_ROOT}/ares-x2d-flush-matrix/green/generated-ids-legal-red/case-75aoF4`
has two nozzle types, two ids, four variants and two variant groups. Identified
Native succeeds; public `load_project`/`slice_project` on pre-FM base9e3 fails
with `invalid Orca option printer_extruder_variant`. Same-input adapter evidence
is `green/generated-ids-base-same-input.txt` (no repository debug additions).

Move the witness out of the FM acceptance module; preserve its old source and
artifacts. Once this spec is approved, bind a separate durable E2E module and
fresh base/main RED receipts before production edits. Establish whether metadata
reselection, incomplete ids or another source phase causes the error; do not
assert the currently suspected mechanism as fact.

## Source/Native trace and admitted correction

Fresh RED on integrated FM main310f5ae9a retains the exact original input
SHA600d0bdd7b626855411a6d85b342fc1fc0a0b85b6a676e998a742c429d7e9d08.
Native succeeds; Ares reports `invalid Orca option printer_extruder_variant`.
Receipt: `${TMP_ROOT}/ares-generated-printer-ids/red/admission.json` and
`artifacts/case-q0fVO4`. No test input/assertion/comparator modification.

Read-only GDB against the identified seeded binary SHA59fb88a2cb441713c2f572cb06e30e4ab58c22a676cacca1812372b9575e73ff
records actual `get_index_for_extruder` calls and their arrays/returns in
`${TMP_ROOT}/ares-generated-printer-ids/trace/native.json`; `native-gdb.py`,
`index-disassembly.txt` and `finding.json` preserve the repeatable probes/ABI.
Trace Native output differs from the fresh RED Native stream only in its
existing generator-clock line, not any config or motion byte.

1. CLI flush selection and Print::apply variant-one see ids[1,2], four variants
   and two ordered groups; generated-id lookup correctly returns[0,2].
2. Variant-one materializes the actual metadata with `get_at`: variants become
   [Direct Drive Standard,Bowden Standard], ids become[1,1]. Do not repair ids
   to[1,2], freeze the original indices, or reorder updates.
3. PrintApply.cpp:1167 calls variant-two on that mutated config, so its second
   physical request returns -1. `PrintConfig.cpp:9677–9681` then explicitly
   maps negative to0. Although the comment mentions UI transients, the GDB
   caller is `Print::apply` on the CLI path and the branch is defined behavior.
   Rust currently converts this miss to an error before materialization.

Coordinator authorizes this exact variant-two `Print::apply` invocation to
map the selector's None (upstream -1) to0 before stride-two materialization.
All other `resolve_printer_indices` invocations retain their existing error
policy; process/filament/error-admission policies are not broadened. The shared
selector itself returns None for no match and never implements negative→0.
No catch-all error conversion or generic fallback flag is introduced.

Extract the existing selector into one core function named
`get_index_for_extruder`, porting PrintConfig.cpp:9004–9060: complete ids versus
ordered generated ids, trim/compressed-comma groups, first matching variant
and stride. Reuse this authority in the existing materialization and the FM
caller at OrcaSlicer.cpp:3461–3462. The FM module must not regain an inline
id generator. Incomplete id coverage is now a resolvable dependency only when
the shared selector resolves every required nozzle; unresolved dependencies
continue to decline reconstruction. Calculator, colour predicate, independent
multiplier resizing and GCode scaling/rounding/size guard stay unchanged.

Failure inventory before production: applying negative→0 globally masks real
errors; freezing pre-update indices picks wrong stride-two data; repairing ids
changes metadata; duplicate FM selector drifts; missing generated groups must
not produce a replacement dataset; diagonal/support constants remain independent.
Verify the unchanged original RED against complete Native output, then seeded
full v2 and per-case complete raw Ares byte equality for FM cases against the
integrated FM candidate. Every byte difference must be inventoried and explained.

## Exit

Separate spec, admitted RED and GREEN commits. Native provenance and complete
byte comparison, unchanged existing streams, workspace nextest, strict Clippy,
rustfmt and Tier1 checks. Generated-id correction cannot hold FM acceptance
hostage and FM cannot silently absorb generic materialization work.
