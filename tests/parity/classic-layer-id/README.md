# Classic physical-layer ordering REDs

Upstream `PerimeterGenerator.cpp:1463–1471`; approved SPEC137b3e01c and
conditional brim deferral729b94370. Only the two later-owner Classic
Filled/None cases are active in matrix.json. No brim or lower-slice correction
is implied by this fixture.

The 3MF is byte-identical to accepted M-v3 discovery input SHA256
`afcd62a6835e860b9d92d67d159aef5bc3ad8b153ca1e6d25918178128ed24ff`.
M-v1/v2 Native container failures are preserved outside this fixture; M-v3
repaired plate/model_instance via Native arrange1 export, without replacing
mesh/options/transforms. Full frozen source evidence is archived outside /tmp
and cited by docs/architecture/classic-physical-layer-wall-order-spec.md.

Original M-v3 manifest and profile provenance bytes are retained unchanged
(including historical other-candidate manifest rows); only matrix case selection
is narrowed. Runner validates selected input, manifest/provenance and upstream
profile-parent hashes, Native effective engine/mode, then compares complete
ordered public slice_with_profiles output with generator-only normalization.
Movement, arcs, M73, statistics and footer remain part of acceptance.
