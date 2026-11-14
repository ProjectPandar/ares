# Preset-select GUI export configuration oracle

This is an oracle-only extension to `tools/chain-probe/preset-select.patch`, not
an Ares production configuration serializer. Each existing `bundle()` observation
adds one field, `full_fff_config`: a string containing the exact native
`project_settings.config` JSON serialization of that observed bundle's secure FFF
configuration. Initial and post-import observations include the field. All existing
fields and their outer JSON encoding remain unchanged.

## Source boundary

Pinned OrcaSlicer revision: `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`.
References below are relative to `src/` at that revision.

- `slic3r/GUI/Plater.cpp:20493` obtains `PresetBundle::full_config_secure()` in
  the live 3MF export path; `20638` assigns that configuration to `StoreParams`.
  The superficially similar `20371` call is in a commented-out AMF exporter,
  not another live 3MF entrypoint.
- `libslic3r/PresetBundle.cpp:5470–5482::full_config_secure` calls private
  `full_fff_config(false)` with the default unspecified filament-map argument,
  then removes exactly seven host/credential options: `print_host`,
  `print_host_webui`, `printhost_apikey`, `printhost_cafile`, `printhost_user`,
  `printhost_password`, and `printhost_port`.
- `libslic3r/Format/bbs_3mf.cpp:8199–8204::_add_project_config_file_to_archive`
  uses `save_to_json` with name `project_settings`, from `project`, and
  `SLIC3R_VERSION`, then stores the bytes as `Metadata/project_settings.config`
  (the archive member constant is at line230).
- `libslic3r/Config.cpp:1557–1605::ConfigBase::save_to_json(std::ostream&, ...)`
  is the shared native serializer. It writes native string/scalar/vector values,
  metadata, one-tab JSON indentation, and the final LF. Its plugin-manifest
  handling depends on the registered capability resolver; a headless observer
  must not pretend that it loaded the GUI's plugin resolver.

The oracle calls those public `full_config_secure` and stream `save_to_json`
methods directly. It does not expose the private FFF method, reimplement native
JSON formatting, write a temporary config file, or add an Ares serializer.

## Payload semantics

`full_fff_config` is JSON **text**, not a `; key = value` G-code block. Native
vectors are JSON arrays, not a re-parsed CSV/semicolon representation. It has no
G-code writer's 31-key exclusion list, `extruder_colour` aliasing, duplicate
wipe-tower coordinate records, or CONFIG_BLOCK wrappers. Only the native secure
configuration's seven-key removal applies. The outer probe response continues
using its existing two-space JSON serialization; escaping of this new string
must not alter any older field.

The composition is explicitly **unexpanded** (`false`). It is the configuration
used by the named GUI archive exporter, not `Print::apply`'s expanded state or
`GCode::append_full_config`. No caller-owned plate maps are invented or restored.
The observer reports the bundle state after its existing preset import. It does
not claim that every stateful GUI session, plugin environment, or downstream CLI
slice is equivalent to that bundle observation.

## Included and deferred scope

Included: the existing test-only preset collection/import observer, its pinned
build recipe, the additive secure FFF project-config JSON string, and native
serialization without modifying previous observations. The probe is a temporary
oracle compatibility shell around `PresetBundle`, not an Ares slicing pipeline.
The separate executable may be selected explicitly by an oracle consumer.

Deferred: Rust/runtime/browser production changes; native GUI behavior changes;
plugin-resolver initialization; scene/map restoration; applied-print/G-code
configuration synthesis; CLI equivalence and the former three-key-difference
target; SLA; and automatic replacement of any canonical oracle environment or
pointer. Canonical oracle selection is outside this serialization boundary.

## Tests

A two-filament FFF 3MF import must produce native project-config JSON strings in
both the initial and post-import bundles. Each payload retains native scalar/vector
shape, `project_settings` / `project` metadata, version, and final LF. The seven
secure keys are absent; G-code-specific filtering and formatting are not applied.
Removing only the added fields reproduces the original observer's values and exact
outer JSON bytes. Bundle values and existing fields are not rewritten or defaulted.
These assertions establish the payload contract, not slicing/G-code equivalence or
reconstruction of a whole stateful GUI session.
