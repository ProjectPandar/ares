# Classic top-layer one-wall reduction requires a top shell

## Owning upstream boundary

OrcaSlicer v2.5.0-beta `345efec64c7b95a05ce5cf96df0dde66a158835c`,
`src/libslic3r/PerimeterGenerator.cpp:619-622`, defines
`has_top_shell_layers(config)` as `top_shell_layers.value > 0`.
`PerimeterGenerator::process_classic` at `:1540` combines it with
`only_one_wall_top`; `:1557-1559` consumes that gated value for the topmost layer.
The same local flag controls the method's later top-surface split branches.
Zero shell layers leave no top shell for the option to act on. Top-shell
thickness, surface density and fill expansion are not this predicate.

## Rust destination and bounded scope

The existing classic rewrite materializes the upstream local flag in two
configuration records:

- `crates/ares-core/src/project_slice/perimeters/classic/preflight.rs::validate_record`,
  consumed by `classic/prelude.rs` for topmost wall-loop count.
- `crates/ares-core/src/project_slice/perimeters/classic/top_split/config.rs::validate_project`,
  consumed by `classic/top_split.rs` for top-surface splitting.

Both currently copy the raw option without the beta shell-layer predicate. They
are bounded compatibility shells around the same upstream flag, not independent
Ares scheduling decisions. The intended port gates both copies by the existing
region `top_shell_layers > 0`; no additional geometry operation, algorithm,
public API, backend or numeric policy is introduced. This scalar option gate does
not require a new Clipper2 contract.

Included: classic top-shell-layer eligibility, including the topmost reduction
and the corresponding split eligibility. Deferred: Arachne, the beta's new top-fill
wall-reclamation/classification behavior, precise seams, and geometry migration.
The already-approved bottom-shell gate and its public tests stay unchanged.

## Public RED and failure inventory

The public seam is the official beta CLI followed by
`ares_core::slice_project_plate_validated`. The frozen classic cube input derives
from the existing zero-bottom-shell wall fixture, changing only
`only_one_wall_first_layer` to zero and `only_one_wall_top` to one. It retains zero
top/bottom shell layers and sparse infill, two walls, no support/brim, and unchanged
mesh/archive members outside project settings.

Failure modes are retaining the old unconditional topmost reduction, gating on
thickness/density instead of layer count, updating only one of the two flag copies,
or accidentally changing the already-supported positive-shell branch. The single
RED witnesses the zero-shell topmost branch; it does not independently qualify
every intermediate top-surface geometry or positive-shell path.

`classic_top_layer_keeps_inner_wall_without_top_shell` qualifies full Native double
replay and verifies runtime Native last-layer Inner/Outer wall roles before
strictly comparing the last object-layer output with the public API output.
The block includes travel to its walls; this fixture has no top fill. Extraction
uses only emitted layer/object delimiters, not private state or a copied wall
algorithm. No Native output is committed. Frozen input and repeatable raw/block
artifacts provide evidence without claiming complete beta G-code or CLI parity.

Registration stays under the existing option-domain group; the test root remains
byte-identical to main. Existing comparators and approved tests are unchanged.
Validation is the single directed public RED/GREEN filter plus fmt and workspace
all-target Clippy; integration owns broad E2E and ledger linkage. This slice queues
on the existing approved chain and does not authorize a merge by itself.
