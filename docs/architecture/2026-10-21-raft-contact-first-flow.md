# ARD: Single-layer raft contact uses interface width, not flange width

Status: docs-only SPEC authorized, then Native-first formal RED. STOP
before GREEN without separate review approval. Independent base
47821c760d5ea86102a34bbc78c092bbaf381214, not stacked on any of the five
frozen W5 chains. No production change in this phase.

## Upstream owner and non-negotiable Rust destination

OrcaSlicer2.4.2 `libslic3r/Support/SupportCommon.cpp:1438–1441` sets
n_raft_layers=max(0,raft_layers-1) for the base-raft toolpath pass. With
raft_layers1 that pass does NOT run; its Base flange branch1493–1504
using first_layer_flow is therefore inapplicable.

The single contact layer instead runs extrude_interface's RaftContact
branch1700–1738, using raft_interface_flow.with_height(layer.height) and
erSupportMaterialInterface. SupportParameters.hpp60–63 distinguishes
first_layer_flow from raft_interface_flow (support interface flow).
This is an upstream support/raft flow-selection port, not an Ares policy.

Rust destination ONLY
`crates/ares-core/src/project_slice/raft/emit.rs::raft_layer_plans`,
width_mm selection123–135. It currently chooses first_layer_width_mm
for index0 even if the layer kind is Contact. The Contact fill spec
already has correct interface_width_mm; materialize_layer in
`gcode_emit/layers/raft_schedule.rs:144–153` correctly trusts plan width.
Do not repair that consumer or override emitted WIDTH/E afterward.

Future separately approved GREEN is ONLY width selection: apply the
base-flange first-layer width only to a layer the upstream base pass
actually emits, NOT Contact. Use existing layer-kind/base-flange identity,
not shape, dimension, width-value or raft-count-specific exceptions.
Preserve all existing downstream consumers and remaining selection arms.

Everything else explicitly deferred: adjacent height selection, Flow/
volume formula, spacing, density, angle, spec, role, geometry, EPSILON,
grid, coordinates, layer schedule/counts, parser/processor/time/M73 and
queued patches. No new API/abstraction/configuration/legacy fallback or
cleanup. Any strict residual after the one width fix requires STOP/report,
not scope expansion or normalization. No sufficiency claim before GREEN.

## GUI sole-key actual exported INPUT pair

Closed8-point star (16 alternating XYvertices per cap),34vertices/
64triangles,radii10/5mm,rotation0.17rad,height1.2mm,six object layers.
One object/part/filament,Classic1wall,top/bottom2,sparse15%,grid45°,
spiral/fuzzy/scarf/stagger/wipeOFF,no support/brim/skirt.
GUI Raft layers is the sole changed key **raft_layers0→1**. Common
initial_layer_line_width0.42,support_line_width0.38,heights0.2,
raft_contact_distance0.1. Actual official-exported3MFs copied unchanged:

| Input | Whole SHA256 |
|---|---|
| raft0 control |4a868581914a8b5dde68862978fe2678401eb4c784a7ca0e6ccc32632343704e|
| raft1 positive |e4f862f94000e30939c66a72fc94217826a4714f0fb7bf36f4280f431a91b706|

Identical ZIP inventory, only project_settings.config differs, only
raft_layers changes. Freeze whole/per-entry hashes and export provenance,
not expected-output snapshots or post-export edits. A same-geometry
raft2 screen passed and supports the Base distinction, but is not formal
dual-mode credit and need not add a third fixture to reproduce this gap.

## Native-first strict feedback and direct flow proof

Discovery official1/official2/seeded stable: control wholePASS all3,
positive328 strict line differences all3. First strict diff69 is NON-M73:
Native WIDTH0.38 / Ares WIDTH0.42; first same-XY extrusion diff71 is
Native E.00741 / Ares E.00829. No motion/width/time/header/M73/E/config/
trailing-line mask or tolerance, only canonical generated-by identity/date.

Read-only GDB captures Native FIRST actual SupportMaterialInterface path
width0.3799999952316284,height0.20000000298023224,
mm3_per_mm0.06741592288017273. Rust ONLY raft plan before materialization
is Contact: spec_width0.3799999952316284, but plan width0.41999998688697815;
height_mm/layer_height0.2,print_z0.2,bottom_z0. Thus interface width is
already resolved, then incorrectly overwritten by first-layer width.

All371 first-raft commanded XYZ moves agree in diagnostic correspondence;
this is NOT acceptance filtering or a claim all internal polygons match.
Both debugger full outputs equal ordinary CLI; no inferior function,
data/register write, may-call-functions off. No geometry/processor/seam
fix follows from these observations. Discovery evidence stays separate:
`/home/indexyz/ares-coord/evidence/w5-478-raft-contact-first-flow/`.

## Included / deferred / exclusions

Included after GREEN review ONLY the upstream Base-flange WIDTH predicate
inside raft_layer_plans. No change to height or generic Flow. Existing
correct geometry/spacing/density/angle/spec/interpreter/materializer remain
untouched; no fallback to preserve a known-wrong Contact override.

Avoid queued W1 painted/side-cut,W2 brim/EFC,W3 combine-solid/aligned-
template,W4 concentric sparse,W5 spiral/processor/Lightning join,W6 zero-
area/occlusion/medial/interlocking/nearest. SupportCommon also contains
brim code but this slice is only raft contact flow, with brim disabled.
Five approved W5 chains remain frozen and unconsumed. No dependencies,
platform I/O, public API, new Rust module or unrelated cleanup.

## Formal RED then review

Two DIRECT main native_stream_case::check calls: raft0 control/raft1
positive. Existing checker double-replays Native then delegates complete
validated public API/bare CLI comparison to native_api_cli. No duplicate
replay/normalization/JSON/orchestration, Native output constants/assertions,
private seam, ignored/expected-failure test decoration or G-code snapshot.
Input-only3MFs plus input manifest/README. CPU11/thread1/job1, official
then seeded controlPASS/positiveactualFAIL, stable repeats and complete
retained public attempts. No masking or reduced ordered-byte comparator.
Send SPEC+RED RR and STOP. No GREEN/FULL/rebase/integration/main update;
later GREEN must freeze all tests/inputs/helpers and respect residual STOP.
