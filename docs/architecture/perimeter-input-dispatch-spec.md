# Proposed SPEC: PrintObjectConfig → LayerRegion perimeter dispatch handoff

Status: **REVIEW-REQUEST, docs only; no RED/test/Native/production budget**.
Base mainc251aa616 via approved docs892a850d9. This proposal follows the
[seam v2 NO-RED closure](seam-candidate-slice-z-no-red.md), not a production bug
claim from mismatched configurations. See [ARD](ard-perimeter-input-dispatch.md).

## Owning upstream rewrite slice / exact Rust destination

Orca source pin8500fcdccaa10b5099ac20d252af3a7c560046f1, paths src/libslic3r:
- `PrintConfig.hpp`: PrintObjectConfig.wall_generator1020 and
  PrintRegionConfig.wall_sequence1217 are distinct typed input owners.
- `PrintConfig.cpp::PrintConfigDef::init_fff_params:2158-2182,7155-7167` defines
  wall sequence vocabulary and Classic/Arachne generator vocabulary/default.
- `LayerRegion.cpp::make_perimeters:86-142`: obtains print/region/object config,
  constructs PerimeterGenerator, selects process_arachne vsprocess_classic at
  138-141, with spiral mode restriction. This is the included producer→consumer
  handoff, not a newly designed Ares perimeter dispatcher.
- Downstream context only: Classic `PerimeterGenerator.cpp::traverse_loops:
  237-279` / `process_classic:1450-1469`; Arachne `process_arachne:2269-2340,2471`;
  `GCode.cpp::extrude_perimeters:6131-6146` / `extrude_entity:6087-6097` emits
  supplied loops before seam placement/travel.

Rust destination: `crates/ares-core/src/project_slice/perimeters/context.rs::
prepare_object_contexts:59-144`, especially typed dispatch107-111 and retained
PerimeterInputRecord. Existing resolved object/region options and input record
are a compatibility shell for the cited LayerRegion boundary, not an Ares-owned
pipeline. Reuse those owners; no independent settings/generator planner or API.
Existing Classic `entity_collections/traverse.rs::traverse_loops:202` and Arachne
`arachne/order.rs::order_walls:16` are downstream observation boundaries, NOT
preapproved repair destinations.

## Evidence and falsifiable admission goal

Retained single-control first motion34/layer0/line99 travels after identical skirt
wipe toward different first Inner wall loops: Native smaller clockwise ring,
API/CLI larger CCW ring. Effective NativeClassic/inner-outer differs from
API/CLI Arachne/inner-outer-inner, plus brim/counterbore/reversal settings.
This identifies config-selected perimeter producer/first-entity ownership,
not a matched-input algorithm failure or travel-router defect.

Before any RED request, prove one immutable public 3MF input is admitted through
Native and matching API/CLI project entry with the SAME effective print/object/
region config, including generator, sequence, brim, counterbore, reversal, raft,
fuzzy policy and seam mode. Preserve source/loaded-profile/override provenance;
literal vocabulary aliases require source-grounded interpretation, never a
comparator alias filter. Require byte-equal complete main control first. No
qualified fixture/control is yet selected; fixture/budget approval is a separate
gate. Do not recast the failed v2 control as a positive or drop it to claim RED.

## Included / deferred / review gates

Included proposed scope: fidelity of the existing typed object/region config
handoff to LayerRegion-compatible generator dispatch and retained first-perimeter
identity under matched effective inputs. If parity is already correct, report
NO RED FOUND; no accounting-only change. If a valid failure lands elsewhere,
stop and amend SPEC with its upstream boundary before production.

Deferred: config loader redesign, profile inheritance fixes, enum/API additions,
Classic/Arachne geometry or ordering repairs, seam planes/scoring, travel routing,
brim/support and M73/statistics/footer behavior. No low-level candidate observation
or Native patch is authorized. The first motion difference alone cannot authorize
any of these repairs.

After SPEC/fixture/budget review: public E2E test first, separate frozen RED commit
and independent approval, then only a source-cited correction within the admitted
handoff boundary. Current proposal does not launch that sequence. Full ordered
G-code/API-CLI equality, CPU/resource/exclusive-freeze rules, fresh repeatable
artifacts and all Tier1 platform boundaries remain mandatory. No fallback,
Unsupported guard, options/fixture substitution or assertion weakening.
