# SPEC request: loaded extruder-clearance strict-positive admission

## Source / exact Rust destination

PrintConfig.cpp:2215-2250 declares four coFloat settings with min0:
extruder_clearance_height_to_rod(default40),
extruder_clearance_height_to_lid(default120),
extruder_clearance_radius(default40), nozzle_height(default2.5).
PrintConfig.cpp:10436-10447 Slic3r::validate independently rejects each <=0,
adding its key and 'invalid value '+std::to_string(value) to the shared ordered
map. No by-object/print_sequence predicate surrounds these validation rules.
Zero must be representable during loading, then rejected before processing;
this is source configuration validation, not a new collision-avoidance feature.

Rust options/validation/dimensions.rs::
SliceOptions::validate_extruder_clearance_options already owns these four exact
predicates and existing invalid_float_value_message. aggregate.rs consumes it;
loaded.rs's normalized project validation chain does not directly consume it.
Exact prospective destination: loaded.rs::ProjectSettingsRaw::loaded_range_options
normalized chain, reuse existing dimensions validator/error-map merge. Existing
ProjectSettings::validate_loaded_config consumes the same map. This call-site
observation is only a likely gap, not Native RED. Do not activate full aggregate.
If typed loading rejects earlier, stop for a source-cited amendment; no silent
parser/ProjectSettings widening in this SPEC.

## Bounded Native-first discovery

Reuse admitted flow-1-1 pure3MF from bridge-flow fixtures, exact geometry and
non-config ZIP members unchanged; Marlin2, PAoff, wipe0, Classic/no scarf/tower.
Keep print_sequence by-layer (verify actual token from existing fixture).
Six configurations with all four keys explicit:
- control radius40/rod40/lid120/nozzle2.5;
- each of the four keys individually0, other three control values;
- all four0 for exact map aggregation/order witness.
No negative/NaN/extreme values or additional geometry/admission bypass.

Fresh official+seeded, independent Native roots/cache/artifacts. Native first
record actual input hashes, return/error map, exact key:value lines. Then public
validatedAPI+realCLI on SAME input even when Native rejects. Acceptance versus
Native rejection, or differing InvalidInput/CLI1 versus Native InvalidValues map/
CLI238, is a discovery divergence, not an equivalent passing rejection.
All accepted control outputs must compare complete ordered Native/API/CLI with
unmodified generator-only comparator and rawAPI=CLI; positiveXY/E and effective
four settings required. Rejected inputs have no borrowed G-code oracle.

Native expected invalid value0.000000 with own key, -18/shell238/noGcode;
all-zero must retain all four lexicographically sorted keys. Establish actual
behavior rather than assume rules from schema. If equivalent for all: NO-RED
STOP, evidence only, no passing-test submission. Different upstream stage or
unreached rule: STOP/request amendment, not new guards or broader pipeline work.

## Included / deferred / acceptance

Included only these four loaded strict-positive checks and existing exact map/
InvalidValuesInProject/CLI238/no-output protocol, if live Native divergence
proves the missing consumer. No duplicate policy, min-bound edits, fallback,
clamping or replacement dimensions. Deferred typed parser changes, collision
geometry, print_sequence placement, safe-height emission, profile inheritance,
other validation families (including skirt_height), unknown/negative/nil values,
region overrides and renderer/UI behavior. Existing helpers are compatibility
shells for the cited upstream rules, not an independently designed Ares pipeline.

After discovery: public test-first frozen input-only fixtures,
OrcaRunner::from_env/stages requiredNative exact rejection map and publicAPI Err/
CLI238/noGcode; ordinary control full comparator/API=CLI. RED precedes any
production edit. GREEN minimal loaded consumer, same assertions/
inputs, exact admission parity, raw same-input control unchanged and existing
config/admission regressions. fmt/strict <=400LOC. No committed Native artifacts.
