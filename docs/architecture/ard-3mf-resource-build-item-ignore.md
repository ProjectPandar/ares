# ARD: Resource build payload belongs to ObjectImporter's ignored dispatch

Status: **PROPOSED**, associated [SPEC](3mf-resource-build-item-ignore-spec.md).
Docs only; not runtime or production authorization. Approved omission GREEN87ac
is complete and entering stack-17 with REDa885; do not rerun that GREEN batch.

## Source / destination

Orca8500fcdcc `libslic3r/Format/bbs_3mf.cpp::ObjectImporter` active resource
workers1780-1817 use start/end dispatch5612-5687, which does not dispatch build
or item. Resource end_model5323-5327 adds no instances. Main dispatch3350-3435
and `_handle_start_item:3893-3909` separately own item-field parsing/instances.

Rust destination is private `project/model_xml.rs` resource transport and
`project/load/graph.rs` role-aware decoding under
`load_project_with_preparation`. Existing DTO/graph are temporary compatibility
shells around the named upstream import owner, not a new Ares pipeline.

## Decisions requiring approval

- Root role comes from the resolved package root relationship. Main required
  Build and typed BuildItem decoding remain unchanged; no filename/index policy
  or global serde build default.
- Resource build payload is not a second main-build document. Its fields must
  not trigger main item deserializers or create instances. Current optional
  Build admits omission but still consumes present item fields; this follow-up
  replaces only that remaining compatibility shell behavior.
- Keep the existing XML validation/security pass. Ignoring a resource payload
  does not waive malformed XML, namespace/entity/size/depth/path checks or mesh/
  relationship/identity validation. No XML insertion/reparse/error fallback.
- The proposed `<item/>` is deliberately missing a **main-item** attribute but
  is well-formed XML. Native must prove it is ignored at this resource owner;
  no general claim about schema-invalid main-model admission is permitted.
- Same qualified control, one resource-entry edit, one positive/repeat/control
  per mode. Full Native→public test only, no bypass switches or injected refs.
  Runtime/fixture freezing requires exact input/SPEC approval first.
- RED is attributable public API+realCLI rejection versus complete neutral
  Native output, with full strict-PASS control. No stream means comparison not
  evaluated. Never weaken M73/metadata/motion/statistics/footer comparisons.
- Separate RED review and production approval; freeze tests for GREEN. Any
  residual G-code difference => STOP, not scope growth. Core remains byte-only
  on WASM/Windows/macOS/Linux; changed modules remain under400LOC.

## Deferred

Main build schema/admission, arbitrary unknown elements, unrelated import policy,
units/transforms, geometry/seam/lower-slice work and configuration. M73 discovery
remains archived/deferred. No work on those owners or prior GREEN reruns.
