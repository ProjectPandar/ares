# Frozen direct-STL profile inputs

The geometry generator and manifest reproduce the binary STL inputs. The
profile provenance binds the original WEMAKE3D PhoenixProV1 0.4mm machine,
0.2mm Standard process and Generic PLA @System parent-chain hashes.

The direct-STL override bookkeeping E2E uses `solid-plate.stl` (SHA256
`252418ebf4f2e4c9c0685ec615a5d98c6f9fdb6312cdba6a12457cf49e5b10de`)
with the sole override `counterbore_hole_bridging=none` and a no-override control.
Native is generated at runtime using OrcaRunner's cached direct-STL recipe;
Ares uses public `slice_with_profiles`, original catalog/selection and
`Placement::None`. Complete-output assertions normalize only generated-by
identity and retain movement, M73, statistics and footer.

No Native outputs or execution records are committed. The other frozen inputs
are shared with the independently approved seam/counterbore slices; this branch
does not register their tests or implement their production behavior.

Source boundary: `docs/architecture/direct-stl-profile-override-bookkeeping.md`.
