# Stock profile provenance: beta cutover

Status: Accepted test-data cutover; not a slicer GREEN.

## Decision and upstream boundary

The shared stock profile provenance fixture follows OrcaSlicer beta commit
`345efec64c7b95a05ce5cf96df0dde66a158835c`, replacing its v2.4.2 data pin
`8500fcdccaa10b5099ac20d252af3a7c560046f1`.

The boundary is the original resource profile inheritance data, not a new
configuration pipeline. Existing source-cited Preset/PrintConfig adapters,
Native settings export, and `ares_core::slice_with_profiles` stay unchanged.

The selected profiles remain:

- WEMAKE3D PhoenixProV1 0.4mm nozzle;
- 0.2mm Standard @WEMAKE3D PhoenixProV1 0.4mm nozzle;
- OrcaFilamentLibrary Generic PLA @System.

The guard pins nine raw JSON files across three parent chains. It does not
pin the Native executable or the complete profiles tree. All nine legacy
hashes reproduce the v2.4.2 files. Names, paths, inheritance edges, machine
files, stock classic/partiallybridge controls, and 0.32/0.2mm heights remain
unchanged in beta.

## Included data changes

Four upstream files change under `resources/profiles/`:

| File | Source change |
| --- | --- |
| `WEMAKE3D/process/fdm_process_common.json` | Remove obsolete adaptive-layer-height and tree-support-with-infill entries. |
| `OrcaFilamentLibrary/filament/base/fdm_filament_common.json` | Add multitool ramming enable and volume defaults. |
| `OrcaFilamentLibrary/filament/base/fdm_filament_pla.json` | Move filament identity to the leaf and add ramming flow. |
| `OrcaFilamentLibrary/filament/Generic PLA @System.json` | Add the beta filament identity, extruder variants, and per-variant volumetric limits. |

Update their four raw-file hashes and two derived process/filament overlay
hashes in `counterbore-partiallybridge/profile-provenance.json`. The derived
metadata retains its original format: parent-to-child JSON overlay, keeping
`inherits`, two-space indentation, and a trailing newline. That metadata is
not the current variant-aware runtime adapter output digest.

The provenance document SHA changes from
`7b16569b0e94b04b1ae80f08ee3cbc2b964fb695f5726d014c7cd97f7d9ad18e` to
`e20a012fd3140515b8dfad39e77bda12c44b97ad0ebfff4db3b74ee76d379a29`.
Refresh only its three dependent SHA fields in the Arachne counterbore and
None matrices and the counterbore N-split manifest.

## Non-negotiable preservation

Keep every raw-file assertion and outer-document assertion. Do not bypass
hash failures, weaken comparisons, change reference gates, or retain an old
profile fallback. Geometry, selections, explicit overrides, stock controls,
case identities, and whole-stream generator-only comparisons stay intact.

The shared document has 28 consumers: 15 Arachne/visibility cases, six classic
Bridges/None controls, five classic Filled cases, and two direct-STL
bookkeeping cases. Observations use all 28 before and after on the same
production tree. The test-data commit does not include production changes.

## Deferred behavior

The separate `classic-layer-id/profile-provenance.json` copy and its matrix
are excluded despite their identical old digest. Do not globally replace
that digest or rewrite historical v2.4.2 data.

Beta filament schema changes are not semantically inert. New admission,
Native-premise, or output failures after this prerequisite are observations,
not permission to alter the runtime flattener or tune profiles to pass.
Production configuration, slicing, geometry, emission, and independent
provenance copies remain outside this data-only cutover. Passing the hash
guard does not establish Native/API parity or close a complete-file row.
