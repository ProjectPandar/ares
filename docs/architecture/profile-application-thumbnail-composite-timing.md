# Thumbnail composite timing during profile application

## Upstream boundary

The source boundary is OrcaSlicer `40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19`:

- `Config.cpp:1151–1154`: composite conversion runs after a JSON dictionary has been loaded.
- `PrintConfig.cpp:8598–8603`: the creation default is `48x48/PNG,300x300/PNG`.
- `PrintConfig.cpp:10046–10073`: authored thumbnail definitions are parsed and rendered with comma-space separators during composite conversion.
- `Format/bbs_3mf.cpp:2790–2810`: embedded project JSON uses the JSON loader.
- `PresetBundle.cpp:7827–7839`: vendor resolution copies parent/collection defaults before applying authored values.
- `OrcaSlicer.cpp:3100–3164,3243–3256`: applying selected machine settings copies parsed options; it does not reload JSON or repeat composite conversion.

## Rust destination and invariant

The destination is the post-application typed reconstruction in
`crates/ares-core/src/project/domain/profile_input.rs`. It rebuilds execution
settings from the already loaded embedded/selected configuration after profile
overlay, overrides, and any bed-origin adjustment. This is config application,
not another authored JSON load.

Use the existing `options::json_deserialize::deserialize_config_values`
application boundary there. It decodes typed config values without invoking
the JSON dictionary composite visitor again. Both the initial reconstruction
and the reconstruction after a bed-origin shift follow the same rule.

Authored preset JSON and embedded project JSON retain their actual-load
composite conversion. A selected creation default can overwrite the parsed
embedded thumbnail value and retain its original no-space serialization.
Already normalized authored comma-space values retain their spaces when
copied. No global whitespace rewrite or thumbnail-specific export filter is
permitted.

The existing project/profile adapter remains a compatibility shell around
upstream `DynamicPrintConfig` load/apply phases, not a new Ares-owned pipeline.

## Included and deferred behavior

Included: thumbnail composite timing in the 3MF profile-application rebuild,
including its bed-origin-adjusted reconstruction. Byte comparisons retain
their existing exact generator-only policy.

Deferred: STL override reconstruction, thumbnail codecs/image transport,
global JSON/legacy conversion changes, and broader profile-application
redesign. No new public API, filesystem, terminal, or platform-specific core
behavior is introduced.
