# Arachne first-layer one-wall reduction requires a bottom shell

## Upstream boundary and decision

OrcaSlicer v2.5.0-beta `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`src/libslic3r/PerimeterGenerator.cpp:628-631`, defines
`has_bottom_shell_layers(config)` as `bottom_shell_layers.value > 0`.
`PerimeterGenerator::process_arachne` at `:2682` combines this predicate with
`only_one_wall_first_layer`; `:2708-2711` applies the resulting local flag when
resetting the bottommost layer's loop count. Without a bottom shell, requesting
first-layer one-wall reduction must not remove the ordinary inner walls.

The analogous classic eligibility was already ported separately. That does not
cover the independent Arachne configuration materialization and consumer.

## Rust destination and scope

The owning Rust boundary is
`crates/ares-core/src/project_slice/perimeters/arachne/config.rs::validate_record`.
Its `ArachneRecordConfig::only_one_wall_first_layer` currently copies the raw option
without the shell-layer predicate. The consumer in `arachne/walls.rs` checks this
flag together with `is_bottom_layer` before setting the loop number to zero.

The intended port gates that existing flag with region `bottom_shell_layers > 0`,
replacing the old 2.4.2 compatibility shell while preserving consumer logic,
wall-count arithmetic, raft-layer identity and all geometry/beading operations.
This is a scalar eligibility port; no new Clipper2 operation or API is needed.

Included: Arachne bottom-shell eligibility for the existing first-layer one-wall
option. Deferred: the Arachne top-shell counterpart, new top-fill wall reclamation,
outline/Clipper2 behavior, beading and any broader seam migration. Previously
approved classic gates and center-wall ordering remain unchanged.

## Public witness and failure inventory

The public seam is official-beta Native CLI replay plus
`ares_core::slice_project_plate_validated`. The frozen input derives from
`tests/parity/first_layer_one_wall_requires_bottom_shell/classic_without_bottom_shell.3mf`,
changing only `wall_generator` from classic to Arachne. The unchanged cube, two
requested wall insets, first-layer one-wall flag enabled and zero bottom shells
isolate this generator's missing predicate. Top shells/infill/brim/support are
zero, and no Native output is committed.

Failure modes are continuing to use the raw flag, gating on thickness or density
instead of shell-layer count, changing bottommost/raft identity, or unintentionally
altering classic/center-wall behavior. This RED exercises only the no-bottom-shell
Arachne first layer, not positive-shell, raft or intermediate-layer cases.

`arachne_first_layer_keeps_inner_wall_without_bottom_shell` requires stable runtime
Native double replay and Native Inner-before-Outer roles, then strictly compares
the entire first object-layer block with the public API output, including wall
entry travel, widths and E. Expected output comes from runtime Native rather than
a committed artifact or a copied implementation. Raw outputs and compared blocks
remain repeatable external evidence, not complete G-code/CLI parity proof.

No private test is added or executed. Registration stays in the existing
option-domain group, and the test root remains byte-identical to main. The
per-slice gates are the directed public RED/GREEN filter plus fmt and workspace
all-target Clippy. Broad E2E, migration-ledger linkage and batch integration remain
integrator responsibilities; this source slice does not authorize a merge.
