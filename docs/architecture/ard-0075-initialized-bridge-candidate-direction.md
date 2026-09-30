# ARD-0075: Initialized bridge candidate direction

The upstream owner is `libslic3r/BridgeDetector.hpp::detect_bridging_direction`
(Lines, Polygons), consumed by `LayerRegion.cpp::detect_bridge_directions`.
Rust destinations are `geometry/bridge_direction.rs` and the existing
`project_slice/prepare_infill/external_surfaces/detect_bridge_directions.rs`.
The latter remains an integration shell around the upstream surface boundary.

Equal-cost candidate orientation is observable: opposite orientations place
Rectilinear outer-contour end extensions on opposite sides. Complete KSR
output at layer26 shows the same interior bridge coordinates but differing
outer endpoints. This does not yet prove candidate iteration is the cause.

The existing committed direction adapter models historical MSVC hash traversal;
the initialized Linux reference uses libstdc++. A source-owned repair must keep
normal arithmetic, first-emplace, quantization and strict minimum selection;
it must not choose a fixture-specific sign or normalize away the end extensions.
Any hash/traversal port must be host-independent for all four Tier-1 platforms.

Include only demonstrated candidate-direction behavior. Defer unrelated M73
ordering, other bridge geometry and full option/printer acceptance. Existing
main-worktree bridge changes are preserved and not assumed to be accepted.
Whole streams, including movement and speeds, remain the acceptance criterion.

The initialized native ELF's `.comment` records GCC15.3.0 and clang21.1.8;
`ldd` resolves libstdc++.so.6 to
`/nix/store/w2nk3f7prwzpm6h13rfxh8lh46yfdysj-gcc-15.3.0-lib`.
Supporting library owners are libstdc++-v3 `include/bits/functional_hash.h`
(double hash), `include/bits/hashtable.h` (unique insertion/rehash),
`libsupc++/hash_bytes.cc` (64-bit hash), `src/c++11/hashtable_c++0x.cc` and
`src/shared/hashtable-aux.cc` (prime policy). Prior original-AppImage GCC14.2.0
source reasoning is supporting history, not identity of this initialized build.
Carry FSF notices and GPL3/GCC Exception3.1 texts with the rewrite, without
claiming the exception exempts a source-derived implementation from its license.
