# ARD: Separate resource-part admission from main-model build semantics

Status: proposed; [SPEC](3mf-resource-part-build-spec.md) review only. FULL16 freeze
active; no runtime, test, fixture or production authorization.

## Upstream owner / Rust boundary

Orca8500fcdcc `libslic3r/Format/bbs_3mf.cpp::_BBS_3MF_Importer::ObjectImporter`
loads relationship-selected resource parts before the main model. Active worker
path1780-1817, extract_object_model993 onward and object XML parser5715-5779 feed
resource handlers5612-5687. Those handlers do not dispatch build/item; end_model
5323-5327 adds no build requirement. The main parser separately dispatches item
creation3893-3909. The disabled `#if 0` sequential importer is not this owner.

Rust `project/load/graph.rs::load` currently parses both roles using required
`model_xml::ModelDocument.build`; `load/assemble.rs` consumes only the main build
for instances. Replace only that resource/main transport conflation at private
part decoding. Existing archive/graph/model DTOs are temporary compatibility
shells around ObjectImporter/imported resource identities, not a new Ares loader.

## Non-negotiable decisions

- Exact retained v1 has a valid main build; only referenced body/modifier resources
  omit it. Do not infer main-model missing-build semantics from that evidence.
- Resource omission and empty build must preserve imported meshes, identities,
  component transforms, roles and instance counts. No manufactured instances.
- Preserve current main-model behavior without a global serde build default;
  stop/amend scope if the minimum change would broaden it. Do not add a novel
  Unsupported guard or treat an unrelated parse failure as the desired RED.
- Preserve namespace/entity/size/depth/security/path/relationship/identity checks;
  no XML-string insertion, error-catch/reparse fallback, filename role heuristic,
  archive rescanning, filesystem/terminal IO in core, new resolver or public API.
- The public seam is loader admission + public Project data, not G-code parity.
  A specific public loader rejection of a Native-admitted resource can qualify
  only after separate loader RED approval; historical failed slicing admission
  remains NOT a lower-slice/seam/ordered-byte RED.
- Proposed Native export-only input contract supplies exactly the same standalone
  3MF to both loader oracles; no hidden loaded JSONs. Export neutrality must be
  preflighted under an explicit budget, not assumed. Retain raw exports and input
  entry hashes; no artifact/observer substitution on failure.
- Any future G-code check retains the entire existing strict comparator contract;
  this import-only proposal neither weakens it nor claims downstream parity.
- Separate docs approval ->fixture/export-oracle/budget approval after freeze ->
  fresh public test-only RED ->independent exact-hash review ->production approval.
  Current task is docs only. No Native/nextest before explicit freeze lift.

## Deferred / trade-off

Main missing-build policy, nonempty resource build/items, arbitrary schema
relaxation, transforms/units, nested relationship policy, mesh repair, lower-slice
geometry, seam and config behavior remain separate. Retain the old frozen finding
and v2 fixture correction without substituting the latter for the positive.
A role-aware private decode is narrower than relaxing every model document;
keep modules below400LOC and core byte-only on WASM/Windows/macOS/Linux.
