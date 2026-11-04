# ARD: Resource build payload belongs to ObjectImporter's ignored dispatch

Status: accepted; implemented at main 41f2 (2026-10-16), associated
[SPEC](3mf-resource-build-item-ignore-spec.md). This docs-only refresh identifies
that the behavior is already completed in main; no new runtime,
fixture, RED, test, or production change is proposed.

Main `41f2c90f765d42c6620583121296bb61a2cb839c` contains resource-ignore GREEN
`86b7f3e1c` and omission prerequisite `61c22160b`. Existing role selection decodes
resource `ModelDocument<Option<serde::de::IgnoredAny>>`, converts its internal
build to empty items, and preserves required main `ModelDocument<Build>`.
The existing public witness asserts target Native/API/CLI whole equality and
raw API==CLI; a pre-GREEN rejection made it fail, rather than pass an inverted
assertion. On this base it is a regression, not a new RED.

Non-negotiable: do not undo already merged parity or invent another importer to
manufacture a RED. A new behavior must have a new source-cited boundary and
separate SPEC. Failure inventory and current frozen pair are in the SPEC.
The original decisions below remain the source contract, with the old optional
`Build` gap description historical rather than current. M73 stays deferred.

## Source / destination

Orca8500fcdcc `libslic3r/Format/bbs_3mf.cpp::ObjectImporter` active resource
workers1780-1817 use start/end dispatch5612-5687, which does not dispatch build
or item. Resource end_model5323-5327 adds no instances. Main dispatch3350-3435
and `_handle_start_item:3893-3909` separately own item-field parsing/instances.

Rust destination is private `project/model_xml.rs` resource transport and
`project/load/graph.rs` role-aware decoding under
`load_project_with_preparation`. Existing DTO/graph are temporary compatibility
shells around the named upstream import owner, not a new Ares pipeline.

## Source contract (implemented decisions)

- Root role comes from the resolved package root relationship. Main required
  Build and typed BuildItem decoding remain unchanged; no filename/index policy
  or global serde build default.
- Resource build payload is not a second main-build document. Its fields must
  not trigger main item deserializers or create instances. Main41f2 implements
  this through `Option<IgnoredAny>` and empty internal resource build; the prior
  optional typed `Build` was a temporary compatibility shell, now replaced.
- Keep the existing XML validation/security pass. Ignoring a resource payload
  does not waive malformed XML, namespace/entity/size/depth/path checks or mesh/
  relationship/identity validation. No XML insertion/reparse/error fallback.
- The proposed `<item/>` is deliberately missing a **main-item** attribute but
  is well-formed XML. Native must prove it is ignored at this resource owner;
  no general claim about schema-invalid main-model admission is permitted.
- Same qualified control, one resource-entry edit, one positive/repeat/control
  per mode. Full Native→public test only, no bypass switches or injected refs.
  Runtime/fixture freezing requires an exact input/SPEC first.
- RED is attributable public API+realCLI rejection versus complete neutral
  Native output, with full strict-PASS control. No stream means comparison not
  evaluated. Never weaken M73/metadata/motion/statistics/footer comparisons.
- Separate RED and production commits; freeze tests for GREEN. Any
  residual G-code difference => STOP, not scope growth. Core remains byte-only
  on WASM/Windows/macOS/Linux; changed modules remain under400LOC.

## Deferred

Main build schema/admission, arbitrary unknown elements, unrelated import policy,
units/transforms, geometry/seam/lower-slice work and configuration. M73 discovery
remains archived/deferred. No work on those owners or prior GREEN reruns.
