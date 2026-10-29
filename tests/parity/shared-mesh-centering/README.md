# Shared mesh C2 input fixtures

Approved SPEC5d0f9f2d9; source: bbs_3mf.cpp4867-5052,
Model.cpp1241-1307/2600-2617/2821-2824. These ZIPs contain authored input only.
`generate.py` reproduces deterministic archives from `orca_cli_ender3.3mf`.
`provenance.json` records base/input SHA, actual parameter changes and settings.

The first case is implicit integer id1 sharing across model files, with a
10mm owner and16mm later source: historical Native wall Xmax176.69 versus
Ares179.69. Distinct id4 is the independent-mesh control. Cases cover explicit,
unseen and-1 keys, signed/prefix/no-digit atoi values, reversed build order,
different raw center, fresh/shared rotation and scale, repeated leaves,
nested BFS and current-volume wall-loop metadata.

The two-plate fixture puts the10mm owner on plate1 and the16mm source on plate2
(global X374, plate origin264). Separate Native imports must choose different
owners. A single loaded Project is selected A,B,A and its publicly observable
mesh geometry is recorded and checked; public byte slicing and actual CLI are
compared strictly with Native for each selected plate. The byte API imports
bytes independently: it does not accept or slice that loaded Project directly.
No new public slicing contract is assumed.

Facet isolation/control use a5mm tapered box, manual normal support, owner
blocker code8 and current-volume enforcer code4. Their output comparison must
preserve current-volume annotations when vertices/triangles are shared.
All oracle products, diagnostics, commands and comparison evidence stay in
external fresh artifact roots. No Native product is committed.

## Import-contract dependency (SPEC415d796bd)

`generate_import_contracts.py` authors three input-only ZIPs from the existing
fixtures. Its separate manifest records every changed input field and SHA.
Their hashes exactly match the retained Native preflight inputs:

- unassigned-owner: owner2 becomes nonprintable and loses only its plate1
  membership; explicit import must exclude it before shared-owner selection.
- default-scope: owner2 moves to plate2/globalX374 and object3 to plate1/globalX155;
  plate object/label membership swaps. Native slice0/all-import and slice1/explicit
  import have different owners. API None and CLI without --plate use slice0.
- local-id-bfs: omit only the root component's self p:path. This is the local-ID
  BFS control; the original literal-ID nested-bfs input remains unchanged.

The original nested fixture's expected imported volume count is one, approved
separately against bbs_3mf.cpp:4867-4891. Its equality assertion and ordered
comparison remain strict. Five new public E2Es cover the four contract groups,
including both default and explicit requests on the same authored input.
The shared helper now records Native/API/CLI products before asserting imported
membership, preserving the exact prior membership assertions and comparator.
Selected-view assertions additionally check unassigned exclusion, owner widths,
and retention when selecting None on an already selected view.

At authoring time the directed window is closed: no compilation or oracle run,
and no RED/GREEN claim for these new tests. Fresh official and seeded runs are
required after the coordinator reopens the CPU12 window, before production.
