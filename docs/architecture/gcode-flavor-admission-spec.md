# SPEC: loaded gcode_flavor allowed-enum admission

Status: **SPEC-REVIEW-REQUEST, docs only; reproduced public admission divergence**.
Stacked approved100639724818b10d96a1d995f1bfef8ba43cfe15 queuedstack16;
merged base98a2bfc325b2a06ba4e8b95b9719acd827e2f8f1.
No runtime/code authorization inferred from this document.

## Upstream / Rust destination

PrintConfig.cpp:3904-3920 defines gcode_flavor's allowed enum_values:
marlin,klipper,reprapfirmware,repetier,marlin2. MakerWare at3916 is commented
out despite enum_keys_map retaining its internal enum at171. The parser's known
vocabulary is NOT the public allowed-value set. Native --slice on the pure3MF
rejects makerware before slicing with gcode_flavor: invalid value makerware,
return-18. Preserve this upstream loaded-config validation, not an Ares-owned
unsupported-feature policy. OrcaSlicer.cpp parameter-error reporting supplies
the existing invalid-project-values error map/CLI -18 protocol.

Rust already owns the correct five-member allowlist in
options/validation/gcode_flavor.rs::validate_gcode_flavor_option and uses it in
options/validation/aggregate.rs. Exact destination is the loaded-project
admission boundary: options/validation/loaded.rs::
ProjectSettings::validate_loaded_config (~294-303), with the minimum immediate
raw/typed adapter needed to reuse that existing validator. Existing
project_slice/state.rs and profile_entry.rs call validate_loaded_config after
settings resolution. Do not add a second enum table or change GCodeFlavor's
internal vocabulary, gcode writer or typed parser. Existing option validation is
not absent globally; its missing loaded-entry consumption is the candidate gap.

## Authentic discovery, not M103 parity

Prior bounded M103 discovery root /tmp/ares-makerware-discovery-fsM9rD:
Marlin2/wipe0+wipe1 Native/API/CLI full outputs PASS both modes; MakerWare/wipe0+
wipe1 Native reject in both modes before slice, so no M103 RED. That STOP is
accepted and remains closed. No runtime oracle bypass/admission relaxation.

Explicitly authorized follow-up root /tmp/ares-makerware-admission-F3UXMq/
admission-report.json, CPU18 fresh root, existing exact inputs:
- makerware-wipe0 SHA26dfb65f65dce55b3aed63e6588374c48a6b37080900912649dadab796acb965
- makerware-wipe1 SHAfbb5460de7dc2ed0a1c01da7ae90764faa20d360311b26dabfa3f633a08191f8
Both public slice_project_plate_validated API requests succeed; actual outputs
225119/241130 bytes respectively. Real CLI exit0, empty stderr, raw API=CLI;
actual public descendants/threads CPU18, cleanHEAD receipts and full bytes.
Same input bytes as Native failures. This is **admission parity RED**, not a
comparison of unsupported MakerWare G-code or a rejection-as-success shortcut.
The external generic helper's borrowed Marlin comparison fields are explicitly
NON-GATE and ignored: there is no Native MakerWare G-code. Public RED must NOT
copy that expedient reference setup; assert Native rejection directly.

## Included / deferred

Included only public loaded-config validation consumes the existing source
allowed set and returns existing InvalidValuesInProject map with
'gcode_flavor: invalid value makerware'. Preserve existing error aggregation,
key order, first-error ownership, CLI -18 / shell238, API enum error contract.
No independent unsupported-feature error, generic new guard/fallback or
substitution to marlin2. Reuse existing helper/formatting rather than duplicate
policy. Check resolved settings/raw provenance before choosing minimal adapter.

Deferred internal enum deletion, MakerWare M103/M101/writer/preamble parity,
all inactive-flavor inventory, unknown-token parser policy, legacy flavor
migration, request-shape variants, unrelated range/firmware validation,
profile loading refactors, processed G-code renderer changes, WASM binding
redesign. If another earlier policy blocks exact rejection parity, request
amendment rather than grow this slice. No whole aggregate-validator activation
unless that is proven source-scoped and required; avoid pulling unrelated
unverified rules into loaded admission.

## Public test-first RED design after SPEC approval

Freeze the four discovery pureinputs: marlin2/makerware x wipe0/1, with existing
mesh/non-config bytes/settings/SHA unchanged; comments1 PAexplicitOFF so no
flavor-specific PA commands. No Native outputs in git. Fresh official+seeded
OrcaRunner::from_env/stages required-Native; explicit native result/stdout/stderr
must show MakerWare invalid-value -18, no sliced output used as oracle.
Then public validated byte API MUST return the existing InvalidValuesInProject
with exact line; realCLI MUST return -18/shell238 and existing parameter-error
message, not accepted G-code. API/CLI accepted slicing is the genuine baseline
FAIL for the two positives; no calling that a PASS or dropping rejection cases.
For Marlin2 controls Native must slice and public API/CLI must succeed, raw-equal
and pass the unmodified generator-only whole-output comparator. Admission map
and positive-stream comparisons are separate explicit contracts, no comparator
normalization/tolerance/classifier/verify_value edits.

Expected two authentic admission FAIL+two full-stream PASS per mode. Independently
review exact public RED before production. GREEN only minimal loaded-validation
wiring, same frozen inputs/assertions, exact errors/exit and passing controls;
raw same-input Marlin2 API/CLI bytes unchanged RED->GREEN. Existing validation,
loaded-config and wipe/retraction/comment controls; fmt/strict <=400LOC exact
clean fullhash. CPU18jobs2threads2, fresh serial official+seeded caches/artifacts,
canonical ALL oracle env archived/hashed at launch including current dynamics,
actual Native/thread affinity. No full/workspace/push/integration except owner
allocation; exclusive freezes respected; no Nix GC/store deletion/rebuild.
This commit is docs only. Await approval before public RED/code.
