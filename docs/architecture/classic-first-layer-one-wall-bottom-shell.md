# Classic first-layer one-wall reduction requires a bottom shell

## Upstream boundary and decision

Port the scalar first-layer gate from OrcaSlicer v2.5.0-beta,
`345efec64c7b95a05ce5cf96df0dde66a158835c`:
`src/libslic3r/PerimeterGenerator.cpp:628-631` defines
`has_bottom_shell_layers(config)` as `bottom_shell_layers.value > 0`;
`:1540-1556` combines it with `only_one_wall_first_layer` and consumes that gated
value in `PerimeterGenerator::process_classic`.
With no bottom shell, the first layer retains the requested ordinary walls.
Bottom-surface density and shell thickness are not substitutes for the layer-count
predicate.

The Rust destination is the existing configuration preflight in
`crates/ares-core/src/project_slice/perimeters/classic/preflight.rs::validate_record`.
Its old unconditional first-layer override is the temporary 2.4.2 compatibility
shell to replace. Keep ordinary classic generation and numeric operations intact;
no new pipeline, public API, geometry operation or backend switch is needed.
This change does not depend on the Clipper2 contract.

Included: the missing bottom-shell predicate on the existing classic first-layer
wall-count reduction. Deferred: the Arachne counterpart, top-wall behavior, any
other inherited wall-count semantics, and Clipper2 geometry migration.

## Public witness and failure modes

The seam is the official beta CLI plus `ares_core::slice_project_plate_validated`,
not a private configuration or perimeter helper. The frozen project derives from
`tests/parity/orca_cli_ender3.3mf`, preserving all geometry and other archive members.
It requests classic generation, two walls, first-layer one-wall reduction enabled,
and zero bottom/top shell layers, infill, brim and support. Other changed settings
only isolate ordinary non-arc, non-overhang wall emission.

Potential failures are the old unconditional reduction, gating on thickness or
density instead of the shell-layer count, or applying the predicate to the wrong
layer. The witness isolates the zero-bottom-shell first-layer case; positive-shell
and other wall-count cases are not newly claimed by this slice.

`classic_first_layer_keeps_inner_wall_without_bottom_shell` first qualifies Native
double replay, verifies that the runtime Native first layer has inner and outer
wall roles, then compares its wall block byte-for-byte with the public API wall
block. The frozen input is in
`tests/parity/first_layer_one_wall_requires_bottom_shell/`; Native output is never
committed. Raw Native replay, public API output and extracted wall blocks are
repeatable external artifacts. The assertion covers that first-layer wall block,
not complete beta output agreement. The case is registered in the existing
option-domain group; the test root remains byte-identical to main. Existing strict
comparators remain unchanged; whole-output migration qualification belongs to
integration.

The test is based on the approved beta cutover commit
`49397e3b98522066eeccb80801943a649ad4a155`, so the reference-version contract is
correct. Per-slice validation is the
single public RED/GREEN filter, fmt and workspace/all-target Clippy. This slice
queues for review and cannot merge before parity cutover.
