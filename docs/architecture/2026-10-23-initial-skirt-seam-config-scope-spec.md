# Stateful skirt/brim seam-gap configuration SPEC

## Source-owned boundary and corrected envelope

Port only the `seam_gap` field of OrcaSlicer's current `GCode::m_config` state
(upstream revision `8500fcdccaa10b5099ac20d252af3a7c560046f1`). This is not a
new skirt policy or a generic m_config framework. Original approved RED41423c412
and its two input SHAs remain frozen. Its fixture is **per-object skirt**, not
combined: the original SPEC/discovery description of the envelope was wrong.
The first observed skirt residual is still valid, but the owning call path is
`GCode.cpp:5334::generate_object_skirt_group`, not the combined-only branch.

## Literal upstream single-field state machine

`seam_gap` is a PrintRegionConfig field (`PrintConfig.hpp:1190`), exposed as
float-or-percent (`PrintConfig.cpp:5515+`, `GUI/Tab.cpp:2336`). Define its current
value from the actual configuration apply sequence, never from the currently
emitting object's identity or skirt layer index alone:

1. `GCode.cpp:2533-2534`: apply default object and default region configurations;
   initialize the current seam-gap value to default-region `seam_gap`.
2. `4593-4594::process_layer`: apply first layer object's OBJECT configuration.
   This does not contain/overwrite the region seam-gap field. Retain its current
   value across layer boundaries, including the last region applied previously.
3. `5243-5251`: combined skirts read the current value before instance entry.
4. `5334-5335`: each instance's per-object skirt then object brim read the current
   value BEFORE that instance's default-region/object apply. A later instance
   can inherit the previous instance's last applied region override; reading the
   default unconditionally is not a source-faithful substitute.
5. `5341-5342`: reset region fields to default region, then apply instance OBJECT
   configuration. Reset current seam gap to default here; object apply leaves it.
6. `6136::extrude_perimeters`: apply the actual region for nonempty perimeter
   containers BEFORE `should_print` phase filtering. Even an unprinted phase may
   change the current field. Mirror this ordering, not just emitted walls.
7. `6163::extrude_infill`: apply the actual region when the selected infill/ironing
   entities are nonempty. Retain that field after the region branch returns.
8. `5538-5551`: process_layer returns without restoring region defaults;
   `5557::apply_print_config` is a separate function, not an end-of-layer reset.
9. `4388-4451::generate_skirt` and `4486-4532::generate_object_brim` route loops
   to `5793-5804::extrude_loop`, resolving the current seam-gap percentage against
   the current nozzle and clipping the loop. Preserve geometry and nozzle rules.

For the original fixture the first skirt and first brim see default10%; its
line204 residual on attempted6e1adca05 is first-layer Z0.2, first Brim block,
first closed-loop endpoint, NOT a later skirt. Existing attempt fixes all eight
skirt loop endpoints in that fixture but leaves brim reading eager options50%.
This does not prove that all later skirts should use defaults: the previous
instance's last region in this fixture happens to make those endpoints match.
Diagnosis retained externally in
`worker2-skirt-seam-state-diagnosis-20261023T043438/DIAGNOSIS-REVIEW-REQUEST.md`.

## Rust destination and narrow scope

Destination: existing `project_slice/gcode_emit/skirt.rs`, brim emission/
`motion/skirt_brim.rs`, their direct layer call sites, and a narrowly scoped
current-seam-gap carrier at the existing default-region/region apply boundaries.
`motion/options/build/context.rs` currently eagerly selects the first object's
region, while `motion/options/build/region.rs:160-163` assigns seam gap from
selected region and skirt/brim clipping reads `state.options.seam_gap`.
`object_entry.rs` already applies region options for wall/fill emission. This
scaffold is only a temporary compatibility shell around the cited upstream
configuration lifetime and must not become an independent Ares policy.

A later GREEN may model ONLY current seam_gap, preserving the above apply
sequence and skirt/brim reads. Do not create a general mutable m_config engine,
change other motion options, wall seam geometry, scarf/seam-slope, travel, cooling,
M73/time, tool-changing, validation, geometry clipping or tests. Audit exact
existing region-application carriers before implementation and report required
wiring first. Do not add default-vs-region guards or compatibility fallbacks.
The current layer-index default-gap patch6e1adca05 is unapproved and must be
replaced, not retained as a parallel policy.

## Frozen original RED

Base main `afcb6aa7c`, seed `ordered-brim-owner/far-uncombined.3mf` SHA256
`c9f9c760caedd4fc2118e491b0a3df667346c6b4d2435ef4f752c8c87c4929a6`.
Object2 seam_gap only differs10%→50%; geometry/placement/settings unchanged:

- control SHA256 `d3f6e170ab2162df20575cd39ec9f289df2a899ce2d13ec10040d01666156b12`
- target SHA256 `fe99c85d5beb6585633f12ddd518aaa37bc88b012da9b6819dd6bcfb6a9615c6`

Do not edit original tests, fixtures, SHA assertions, helper or callback.

## Additional state-vs-default witness and RED gate

Coordinator authorized this docs revision followed by an ADDITIONAL test-only
RED on top of attempted6e1adca05. Preserve the existing chain explicitly; that
attempt is not a successful GREEN. Derive two new GUI inputs from the same
far-uncombined seed, retaining geometry, transforms and placement. Keep object2
seam_gap10% in both. Set the first printed object's (source object3) region seam
gap to10% for control and50% for target. Verify printing order and that the
following object's skirt/brim inherits this region state before its own reset.
This must distinguish state inheritance from an unconditional default read.
Record the two input SHAs with the new tests and exact input difference proof.
External geometry observations are diagnostic, not hard-coded test constants.

Use unchanged `runner::native_stream_case::check` with `(false, |_, _| {})` for
one new target and one new control; no copied harness/extra callbacks/guards/
expected-error assertions or masks. Register as a child of an existing module;
DO NOT move/include the root test registry. Keep all files<=400LOC.

Exact new RED tip: CPU13, build jobs1, test threads1, PIN_OFFSET13 and replayCPU13;
both modes serial with independent fresh empty caches. Require Native double
replay success, one genuine strict target failure and one complete control pass,
successful public API/actual CLI generation and raw API/CLI equality. Compare
complete ordered generator-line-only streams; no M73/time/travel/metadata masks.
Run fmt and workspace all-targets clippy, submit SPEC-revision/RED RR and STOP.
No production edits or FULL until RR approval. W2 integration notification has
immediate priority; prior017a9876b chain remains frozen separately.
