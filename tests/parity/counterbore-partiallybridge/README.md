# Frozen direct-STL profile and seam inputs

`generate.py` reproduces the binary STL inputs and geometry manifest.
`profile-provenance.json` binds the original WEMAKE3D PhoenixProV1 0.4mm machine,
0.2mm Standard process and Generic PLA @System parent-chain hashes.

The direct-STL override bookkeeping E2E uses `solid-plate.stl` (SHA256
`252418ebf4f2e4c9c0685ec615a5d98c6f9fdb6312cdba6a12457cf49e5b10de`)
with the sole override `counterbore_hole_bridging=none` and a no-override control.
The seam visibility E2E uses unchanged `stepped-hole.stl` (SHA256
`1274c56d97fd205740505d88f077971c7ffe79f1552df78a4e7432de557e465e`)
and `solid-plate.stl`, with the sole override `counterbore_hole_bridging=none`.

Native references are generated at runtime through OrcaRunner's cached
`--arrange 0 --slice 0` direct-STL recipe. Ares uses public
`ares_core::slice_with_profiles`, original catalog/selection and `Placement::None`.
Complete-output assertions normalize only generated-by identity and retain
movement, M73, statistics and footer. No Native outputs or execution records
are committed. The remaining generated geometry is reserved input for the
independently approved counterbore slice; its four failing RED tests are not
part of the seam integration branch.

Source boundaries:
- `docs/architecture/direct-stl-profile-override-bookkeeping.md`
- `docs/architecture/seam-visibility-eigen-distance-reduction.md`
