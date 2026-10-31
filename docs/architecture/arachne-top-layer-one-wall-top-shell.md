# Arachne top-layer one-wall reduction requires a top shell

## Upstream boundary and decision

OrcaSlicer v2.5.0-beta `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`src/libslic3r/PerimeterGenerator.cpp:619-622`, defines
`has_top_shell_layers(config)` as `top_shell_layers.value > 0`.
`PerimeterGenerator::process_arachne` at `:2681` gates `only_one_wall_top` with this
predicate. The topmost-layer consumer is `:2713-2716`; `:2735-2739` uses the same
local flag for the remaining-inner-loop count and main single-wall pass.

With no top shell, this option has no top surface to act on. Surface density,
shell thickness and fill expansion are not the shell-layer-count predicate.
The classic counterpart was ported separately and does not cover this Arachne
configuration record.

## Rust destination and complete flag ownership

The owning Rust materialization is
`crates/ares-core/src/project_slice/perimeters/arachne/config.rs::validate_record`,
`ArachneRecordConfig::only_one_wall_top`, currently copied from the raw region bool.
`arachne/walls.rs` consumes it for topmost loop reduction, remaining inner-loop
count and the main single-wall pass. Those values determine whether
`arachne/top_surface.rs` regenerates inner walls. The existing materialization
inputs do not add another raw option copy.

The intended port gates this single config field with region `top_shell_layers > 0`.
Keep all consumers, wall counts, regeneration geometry and data ownership unchanged.
This replaces the old 2.4.2 eligibility shell around an upstream local flag, not
an Ares-owned stage. No new Clipper2 operation, numeric policy, helper or API is needed.

Included: Arachne top-shell eligibility across the existing flag consumers.
Deferred: new beta top-fill wall reclamation/classification, beading/outline
changes, precise seams and broader geometry migration. The approved bottom-shell
eligibility and center-wall ordering remain unchanged.

## Public witness and failure inventory

The public seam is official-beta Native CLI replay plus
`ares_core::slice_project_plate_validated`. The frozen input derives from
`tests/parity/top_layer_one_wall_requires_top_shell/classic_without_top_shell.3mf`,
changing only `wall_generator` from classic to Arachne. The cube, two requested
wall insets, top one-wall flag enabled, first-layer one-wall flag disabled, zero
shells/infill/brim/support and all other archive members remain unchanged.
No Native output is committed.

Failure modes are retaining the raw flag, gating on thickness/density rather than
shell layers, updating a consumer instead of its shared eligibility flag, or
unintentionally changing earlier approved wall behavior. This RED observes the
zero-top-shell topmost branch, not every intermediate top-surface geometry,
positive-top-shell or raft case.

`arachne_top_layer_keeps_inner_wall_without_top_shell` qualifies full Native double
replay and requires runtime Native last-layer Inner/Outer wall roles, then strictly
compares the last object-layer output with the public API, including wall-entry
travel, widths and E. Emitted layer/object delimiters select the block; no private
state or copied wall algorithm supplies expectations. Raw streams and compared
blocks are repeatable external evidence, not complete G-code/CLI parity proof.

No private test is added or executed. Registration stays in the existing
option-domain group; the test root remains byte-identical to main. Gates are the
directed public RED/GREEN filter plus fmt and workspace all-target Clippy.
Broad E2E, beta-ledger linkage and batch integration remain integrator work;
this slice does not authorize a merge.
