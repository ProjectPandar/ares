# Vendor preset include resolution and direct settings import

## Decision

Ares preserves two distinct configuration boundaries from OrcaSlicer:

1. Vendor preset composition resolves inheritance and named includes.
2. Direct `--load-settings` import reads the selected JSON records without
   consuming their vendor-preset include links.

The same resolved-layer owner must serve public catalog values and true vendor
preset composition. Direct settings import remains a separate path and must not
call that owner merely because the selected records contain `include` metadata.

The Rust destination is the profile catalog boundary:

- `crates/ares-core/src/profiles/catalog/document.rs` and `store.rs` retain
  source documents, identities, inheritance, vendor scope, and dependency
  metadata.
- `crates/ares-core/src/profiles/catalog/includes.rs` owns resolved layers,
  ordered include application, and semantic default-diff handling.
- `preset_values.rs::Catalog::resolved_values` and
  `compose.rs::Catalog::merge` consume that one resolved-layer owner.
- The direct profile-composition seam uses selected raw records when it models
  `--load-settings` input.

No machine-specific G-code exception, global include switch, hardcoded variant
list, or second resolver is permitted.

## Upstream boundary

The source baseline for this record is OrcaSlicer revision
`345efec64c7b95a05ce5cf96df0dde66a158835c`.

### Vendor preset composition

The upstream owner is `libslic3r/PresetBundle.cpp`:

- `PresetBundle::resolve_vendor_preset` at lines 7582–7667 starts with the
  resolved parent or collection default, applies named include diffs in their
  declared order, and then applies the leaf `config_src`.
- Lines 7645–7648 record a referenced non-instantiable include's resolved diff
  against the collection default before `extend_default_config_length` at line
  7650 and `Preset::normalize`. Include values must therefore be compared and
  applied before padding a template with collection defaults.
- Lines 8273–8282 parse include names. The vendor load scheduler at lines
  7844–7893 accounts for inheritance and include dependencies.
- `resolve_system_preset` at lines 724–757 obtains a resolved system preset.
  The CLI refresh caller is `OrcaSlicer.cpp:2780–2817`.

The port includes source scope, dependency ordering, include ordering, leaf
precedence, non-instantiable template handling, and semantic option comparison.
A named include is not a raw ancestor chain and is not a globally named preset.

### Direct settings import

The direct-import boundary is in `OrcaSlicer.cpp` and `libslic3r/Config.cpp`:

- `OrcaSlicer.cpp:1373–1381` reads the `load_settings` and `load_filaments`
  option lists.
- `OrcaSlicer.cpp:2200–2220` loads each selected settings file through
  `config.load_from_json(file, config_substitution_rule, key_values, reason)`.
- The overload is `DynamicPrintConfig::load_from_json` at
  `libslic3r/Config.cpp:862–869`; it invokes the four-argument loader with
  `load_inherits_to_config=true`.
- `libslic3r/Config.cpp:871–873` defines that loader boundary. The JSON keys
  `BBL_JSON_KEY_INHERITS` and `BBL_JSON_KEY_INCLUDES` are copied into metadata
  only by the `!load_inherits_to_config` branches at lines 965–980. The vendor
  bundle parser uses that metadata mode at `PresetBundle.cpp:8235`.

Consequently, direct `--load-settings` import reads selected records as direct
configuration input and does not resolve their vendor `include` link. This
semantics applies equally to selected machine, process, and filament records.

## Included behaviour

This slice ports the bounded vendor include operation into the profile catalog.
It includes:

- inheritance and include dependency ordering within a vendor and profile kind;
- ordered application of include diffs followed by leaf-owned values;
- collection-default comparison before template padding;
- typed comparison for nullable values and numeric spelling through the existing
  option registry;
- public catalog values and typed profile composition using the same resolved
  layers;
- preservation of profile identities and byte-oriented catalog input;
- direct-load composition using selected raw records without catalog include
  links when that seam represents `--load-settings`.

The catalog remains platform-neutral and receives profile records as bytes. It
must not discover files, mutate a project configuration, or add a catalog to
`SlicerContext`.

## Deferred behaviour

The following upstream owners are outside this slice:

- processing-map production in `libslic3r/GCode/ToolOrdering.cpp:1537+`,
  `build_filament_group_context`, and `get_recommended_filament_maps` at
  lines 1752–1877;
- grouping algorithms in `libslic3r/FilamentGroup.cpp`, including
  `calc_filament_group` at line 1036+, `calc_filament_group_for_flush` at
  line 1253+, and `calc_min_flush_group` at line 901+;
- map write-back in `ToolOrdering.cpp:3054–3083`;
- rematerialization in `Print.cpp:4656–4753`,
  `Print::update_filament_maps_to_config`, including saved-table selection and
  retraction overrides;
- typed variant selection and self-index remapping in
  `PrintConfig.cpp:11884–11994`,
  `DynamicPrintConfig::update_values_to_printer_extruders_for_multiple_filaments`;
- implicit raw-project refresh through `profiles/project_refresh/preset.rs`,
  including `system_config`, changed-key restoration, and project refresh
  wire-shape handling;
- complete output time, M73, footer, and estimator behaviour;
- printer apply ordering, rematerialization consumers, and unrelated geometry,
  support, ironing, retraction, or default-value work.

Deferred owners must consume the same resolved-layer view when their own slices
are implemented. They must not introduce a parallel resolver or infer include
behaviour from a catalog listing.

## Non-negotiable decisions

- `Catalog::resolved_layers` is the sole owner of true vendor preset-bundle
  composition for public catalog values and typed profile composition.
- Direct `--load-settings` composition is distinct and uses selected raw
  records; it does not consume catalog include links.
- Include diffs are captured before default padding and are applied in source
  order with leaf precedence.
- Include lookup is scoped to the vendor, profile kind, and dependency ordering;
  there is no global name lookup or machine-specific special case.
- Existing typed option decoding and semantic equality are reused.
- No filesystem access, UI behaviour, terminal behaviour, synthetic project
  state, compatibility fallback, comparator exception, or hardcoded E3D data is
  added.
- Production work remains confined to the profile/catalog boundary until each
  deferred upstream owner has its own source-cited slice.
