# ARD: Separate resource-part admission from main-model build semantics

Status: boundary approved at c6a9e152a; owner-amended
[SPEC](3mf-resource-part-build-spec.md) uses canonical slicing, not export/load-only.
Main3ee8f272c has advanced/freeze lifted. First RED e1de695aa was rejected for
control G-code divergence. This docs-only input revision requires approval before
any new runtime. Independent qualified test-only RED review precedes production.

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

- Old v1/v2 geometry is frozen history, not the next qualifying pair. Its control
  M73 insertion difference is independent deferred discovery, not this slice's fix.
  New control is exact FULL16 strict-PASS control-before50 SHAadb38ee8; new
  positive0eab5d4d removes ONLY one referenced resource's empty build. Both retain
  main build. Complete SHAs/entry proof/known current-main streams are in SPEC2a.
  No selective M73/settings changes, comparator weakening or implicit substitutions.
- Resource omission and empty build must preserve imported meshes, identities,
  component transforms, roles and instance counts. No manufactured instances.
- Preserve current main-model behavior without a global serde build default;
  stop/amend scope if the minimum change would broaden it. Do not add a novel
  Unsupported guard or treat an unrelated parse failure as the desired RED.
- Preserve namespace/entity/size/depth/security/path/relationship/identity checks;
  no XML-string insertion, error-catch/reparse fallback, filename role heuristic,
  archive rescanning, filesystem/terminal IO in core, new resolver or public API.
- Verification seam is Native --slice completing on the identical frozen3MF,
  then public slice_project_plate_validated and real bare-3MF CLI. No extra
  loaded JSONs, export-only/load-only success, observer or cached Native substitute.
  Use the qualified source control's --slice0 --outputdir argv (no extra arrange
  flag); EACH enabled official/seeded invocation always executes fresh Native→
  public end to end. No Native-only early return, external reference injection or
  reuse of FULL16 output as the new runtime baseline.
- Approved RED is the specific Ares resource-build admission error versus complete
  Native G-code. CLI must confirm the same admission cause. A missing Ares stream
  is not raw byte equality, G-code PASS or an invented ordered-stream difference.
  Historical failed discovery admission remains separately classified.
- Every admitted Ares leg uses API/CLI raw equality and the unchanged strict full
  generator-only comparison, retaining movement/arcs/M73/statistics/footer and
  effective-config differences. Repeat/control neutrality remains mandatory.
- After separately approved GREEN, ANY residual G-code difference means STOP and
  report the complete streams/first difference. No downstream scope growth,
  comparator weakening, fixture substitution or metadata filtering to get PASS.
- Boundary approved ->docs-only seam amendment ->Native-first public test-only
  RED after MAIN-ADVANCE/FREEZE LIFTED ->independent exact-hash RED review ->
  separate production approval. Old control failure is NOT approved RED; new
  control must strict-PASS, positive missing-build error is the real admission
  RED. No Native/nextest before input revision approval or during any new freeze.

## Deferred / trade-off

Main missing-build policy, nonempty resource build/items, arbitrary schema
relaxation, transforms/units, nested relationship policy, mesh repair, lower-slice
geometry, seam and config behavior remain separate. Retain the old frozen finding
and v2 fixture correction without substituting the latter for the positive.
A role-aware private decode is narrower than relaxing every model document;
keep modules below400LOC and core byte-only on WASM/Windows/macOS/Linux.
