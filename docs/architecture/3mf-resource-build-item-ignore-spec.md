# SPEC: ObjectImporter ignores resource-part build/item payload

Status: **implemented.** Main includes `86b7f3e1c` (resource build/item
ignore), following `61c22160b` (resource build omission). Do not recreate the
RED on a main which already contains its GREEN.
M73 remains deferred.

## Current-main finding

`project/load/graph.rs::load` already selects required `ModelDocument<Build>`
for the resolved root, and `ModelDocument<Option<serde::de::IgnoredAny>>` for
resource parts. `project/model_xml.rs::into_resource_document` already moves
unit/extensions/metadata/resources unchanged and supplies an empty internal
resource build; only the main build can create instances. The destination and
source boundary are therefore already present.

The existing frozen input-only witness is
`tests/parity/3mf-resource-item-ignore/{positive,control}.3mf`; existing public
test is `runner/resource_item_ignore.rs::resource_item_ignored_public_slicing`.
Both roles still pass the common XML security/mesh/path/relationship validation.
No change to test, production, fixture or comparator is needed.

A new public RED cannot honestly be produced for identical inputs on this base.
The oracle seam remains fresh Native at runtime, validated public API,
real bare CLI, raw API==CLI and complete strict generator-only equality; the
existing test asserts target success/equality and failed before GREEN. On this
base it is a nonregression witness, not a new RED. Any request for additional
behavior requires a separately named upstream boundary and input.

### Failure inventory

| Failure | Classification / action |
|---|---|
| Requested boundary already present in main | Report duplicate/completed slice; no fabricated RED or removal of GREEN. |
| Fixture archive/entry hash drift | STOP before Native; retain frozen positive/control identities. |
| Native nonzero/incomplete footer/snapshot | Invalid oracle, not product RED; STOP without retries or input substitution. |
| Native positive/repeat non-neutral | STOP/adjudicate nondeterminism; do not normalize behavioral lines away. |
| API or real CLI rejects while same-input Native completes | Admission delta only; target-success assertions must FAIL, not pass an expected-error branch. |
| Missing Ares stream | Raw/whole comparison NOT EVALUATED; never missing-output equality. |
| Control admission/raw/strict comparison fails | Disqualify RED; STOP, no M73/metadata/output filtering. |
| Public stream differs anywhere except permitted generator identity | Target equality FAIL; preserve first difference and whole streams; no scope growth. |
| Main build/item policy or instance count changes | Outside resource dispatch slice; STOP, no guards/fallbacks or global defaults. |
| XML/security/path/relationship checks regress | Reject change; ignoring payload does not waive document validation. |

### Current frozen pair and included/deferred scope

Positive SHA `41dca2941a280ed674b063bed0cbb5afd9d37336c4757f8327afc331e39d1464`;
control SHA `adb38ee8df9485a864135007753fb935aedf7883fe37f910109f20145f3c7be2`.
Only `3D/Objects/overhang.stl_1.model` changes `<build/>` to
`<build><item/></build>`; manifest in the fixture directory pins every entry.
Included behavior is non-consumption of resource build/item fields and no
resource-created instances, under existing XML validation. Main item decoding,
arbitrary unknown tags, nested relationship policy, units/transforms/geometry,
PrintConfig, and M73 remain deferred. No new Ares loader/pipeline is designed.

The source-cited design/history below records the original pre-GREEN proposal;
its statements about an optional typed resource `Build` gap and an unfrozen pair
are **historical**, not the state of main41f2.

## Original proposal (historical, already implemented)

## 1. Upstream rewrite slice

Pin: OrcaSlicer `8500fcdccaa10b5099ac20d252af3a7c560046f1`,
`src/libslic3r/Format/bbs_3mf.cpp`:

- `_load_model_from_file:1780-1817`: active `ObjectImporter` workers load resource
  parts before the main parser. The preceding `#if 0` loop is not the owner.
- `ObjectImporter::_handle_object_start_model_xml_element:5612-5649` and
  `_handle_object_end_model_xml_element:5651-5687`: dispatch resource/model/mesh/
  component/metadata handlers, **not BUILD_TAG or ITEM_TAG**. Resource item
  attributes are not fed to the main item's field readers.
- `ObjectImporter::_handle_object_end_model:5323-5327`: no build requirement or
  instance construction.
- In contrast, main dispatch `3350-3435` handles build/item;
  `_handle_start_item:3893-3909` reads object ID/path/transform/printable/auto_drop
  and calls `_create_object_instance`. This main behavior is not being changed.

Port the resource dispatch's non-consumption of build/item payload, not a new
Ares loader policy or pipeline. This is the separately deferred **present,
nonempty resource build** behavior, not another omission fix.

## 2. Exact Rust boundary and current gap

`project/load.rs::load_project_with_preparation` resolves the main model through
root relationships and calls private `project/load/graph.rs::load`. GREEN87ac
selects required `ModelDocument<Build>` for that resolved root and
`ModelDocument<Option<Build>>` for other loaded parts. Optionality admits absence,
but a present resource build still deserializes `Build.items: Vec<BuildItem>`.
`project/model_xml.rs::BuildItem.object_id` requires `@objectid`; transform and
boolean fields also use typed deserializers. Thus resource fields that upstream
never consumes can still reject Ares at the public admission seam.

This is a **static prediction**, not an observed RED. No new positive has been
constructed or sliced. Native completion/neutrality must adjudicate it first.

Destination: those two private decoding modules only. Keep the resolved-root
role selection and required main `Build`; replace resource build-payload decoding
with explicit non-consuming transport semantics under existing XML validation.
Existing `ModelDocument`/graph are temporary compatibility shells for the cited
ObjectImporter boundary, not an independent importer design. No public API,
resolver, fallback, filename heuristic, XML patch/reparse, diagnostic core I/O or
global serde build default. Do not implement before a qualified public RED.

Only the main build is consumed by
`load/assemble.rs::project_domain_with_budget:108-128` and
`load/assemble/component_graph.rs::validate_bare_metadata_identity:14`.
Resource build items must not manufacture instances or change graph identities.

## 3. Proposed finite input pair — not yet frozen

Control: unchanged `tests/parity/wipe-before-override/control-before50.3mf`, SHA
`adb38ee8df9485a864135007753fb935aedf7883fe37f910109f20145f3c7be2`.
It is also the resource-build control. Fresh runtime remains mandatory;
historical full-run/RED/GREEN streams are provenance, never the new oracle.

Proposed positive: replace **only** `<build/>` in
`3D/Objects/overhang.stl_1.model` with `<build><item/></build>`.
The item intentionally lacks the main item's required `objectid`: this is a
well-formed XML probe of **ignored resource payload**, not a claim that such an
item is schema-valid for a main build, nor a general malformed-3MF tolerance goal.
All other uncompressed ZIP entries and main build/IDs/roles/mesh/transforms/
settings/M73 remain identical. No loaded JSONs or extra option overrides.

Freeze one deterministic derivation, complete archive/entry
SHA manifest and exact argv before runtime. Positive SHA is deliberately not
claimed now because no new fixture bytes have been generated. No new fixture or
test files are part of this docs proposal.

## 4. Public RED seam

Proposed matrix: official and seeded, each positive + fresh positive repeat +
control; **three fresh Native slices per mode**, no alternate candidates/retries.
Each enabled public test always runs Native→public end to end. Retain the
existing `reference_enabled`/`from_env` gates only; no Native-only silent PASS,
external-reference injection or cached/export-only/load-only oracle path.

Native argv: `--slice 0 --outputdir <fresh> <same frozen bare 3MF>`;
no explicit arrange flag. Require exit0, authoritative complete G-code/footer,
snapshots and full generator-only positive/repeat/control neutrality before
public calls in each invocation. Native failure/non-neutrality means STOP and
report, not a qualified RED or permission to substitute another input.

Then canonical `ares_core::slice_project_plate_validated`: defaults, plateNone,
context seed0, deterministic metadata matching the real bare-3MF CLI. Expected
RED is a specific resource-item `objectid` decode rejection with matching CLI
cause against complete Native output. Different error => STOP/adjudication.
Missing Ares stream => raw equality and strict comparator NOT EVALUATED, not PASS.
Control must admit and pass raw API==CLI plus the **entire** unchanged strict
Native/API/CLI generator-only comparator. Preserve M73, config, ordered motion,
arcs, statistics and footer; do not precondition comparisons on config equality.

Freeze qualified test-only public RED before
production. Tests/comparator stay frozen for GREEN. After GREEN,
positive/repeat/control must all pass the whole comparator/raw equality in both
modes; control raw stream remains unchanged. ANY residual G-code delta => STOP,
no downstream or M73 repair. Existing 3MF/load regression family, fmt, strict
clippy, WASM/Tier1 byte-only checks are GREEN gates.

## 5. Included / deferred / exit

Included: build/item dispatch non-consumption in relationship-selected resource
parts, proven at the canonical public slicing seam with the finite pair above.
Preserve XML well-formedness, namespace/entity/size/depth/security checks, archive
ownership/relationships, resource identities, mesh validation and main instances.

Deferred: main missing-build/item policy; arbitrary unknown-tag tolerance;
unrelated resource metadata, nested relationships, units/transforms, mesh repair,
seam/lower-slice behavior, PrintConfig/catalog policy and M73. No blanket schema
relaxation. A failure of a preserved XML/security check is not this RED.

Exit: public RED with strict-PASS control, then bounded ObjectImporter GREEN
with complete parity/nonregression.
[ARD](ard-3mf-resource-build-item-ignore.md).
