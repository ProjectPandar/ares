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
separate counterbore slice; its four failing RED tests are not part of the
seam slice.

Source boundaries:
- `docs/architecture/direct-stl-profile-override-bookkeeping.md`
- `docs/architecture/seam-visibility-eigen-distance-reduction.md`

# Classic counterbore partiallybridge fixtures

Source spec: `docs/architecture/classic-counterbore-partiallybridge.md`.

All geometry is frozen before the RED Native runs. `generate.py` reproduces the
binary STLs and `geometry-manifest.json`; every undirected mesh edge is used
exactly twice. The primary is byte-identical to the earlier Native/GDB diagnostic
input (SHA256 `1274c56d97fd205740505d88f077971c7ffe79f1552df78a4e7432de557e465e`).
No geometry was tuned against Ares output.

Stock selection (no primary overrides):

- WEMAKE3D PhoenixProV1 0.4mm nozzle
- 0.2mm Standard @WEMAKE3D PhoenixProV1 0.4mm nozzle
- OrcaFilamentLibrary Generic PLA @System

The process inherits classic + partiallybridge, first layer 0.32mm, later layers
0.2mm. `profile-provenance.json` binds the original parent-chain file hashes.
Native gets independently flattened profiles and generates its reference at
runtime through OrcaRunner's direct-STL recipe and `ARES_ORCA_CACHE_DIR`.
Producer, seed, resources, input bytes and flattened profile bytes bind the
cache entry. Ares gets the original vendor catalog and selection through public
`ares_core::slice_with_profiles`, with `Placement::None`, seed0 and deterministic
generation metadata. Native outputs and execution records stay outside Git.

| Case | Geometry / sole configuration change |
| --- | --- |
| primary | 24mm plate, 8mm lower hole → 4mm upper hole at Z0.32, height0.52 |
| none-control | identical primary STL; only `counterbore_hole_bridging=none` |
| first-layer-control | 8mm straight hole, height0.32; no lower slice |
| straight-hole-control | 8mm straight hole, height0.52; no counterbore step |
| solid-control | no hole, height0.52 |
| two-islands | two disconnected primary meshes at (128,150)/(172,150), one STL |

The assertions request complete Native/Ares G-code equality, normalizing only
the generated-by identity. Movement, M73, statistics and footer remain included.
A typed UnsupportedProjectFeature is recorded as GUARD_REJECTED, other errors as
ARES_ERROR, and successful but unequal byte output as DIVERGENT. No rejection
is accepted as a passing assertion, and the RED tests are not ignored.

The tested seam can fail through: profile/config rejection before slicing,
unsupported counterbore guard, geometry or fill errors, wrong surface grouping /
metadata / fill recovery, or full-output divergence. Actual observed status is
written before the parity assertion in `counterbore-result.json` and, for errors,
`ares-error.json`. A Native failure/TIMEOUT is not promoted to a parity verdict.

Run via `cargo nextest run -p ares-cli --test orca_parity -E 'test(counterbore_)'`
with the normal required Native harness environment. Prebuild outside the shared300s per-case deadline. Use fresh Native
cache/artifact roots and seed0. The independent
`orca_parity_seam_visibility_eigen_distance_complete_output` RED reuses the
unchanged None-control input and full-output assertion under
`docs/architecture/seam-visibility-eigen-distance-reduction.md`. No GREEN is
claimed by these input fixtures.
