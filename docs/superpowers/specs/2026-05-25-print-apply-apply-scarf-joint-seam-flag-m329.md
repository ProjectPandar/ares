# Spec: M329 PrintApply apply scarf joint seam flag

## Goal

Port the `Print::apply(...)` scarf joint seam detection and guarded config-set block from `OrcaSlicer/src/libslic3r/PrintApply.cpp:1140-1154` into `ares-core` private staged state.

## Upstream source mapping

```cpp
{
    const auto& o = model.objects;
    const auto opt_has_scarf_joint_seam = [](const DynamicConfig& c) {
        return c.has("seam_slope_type") && c.opt_enum<SeamScarfType>("seam_slope_type") != SeamScarfType::None;
    };
    const bool has_scarf_joint_seam = std::any_of(o.begin(), o.end(), [&new_full_config, &opt_has_scarf_joint_seam](ModelObject* obj) {
        return obj->get_config_value<ConfigOptionEnum<SeamScarfType>>(new_full_config, "seam_slope_type")->value != SeamScarfType::None ||
               std::any_of(obj->volumes.begin(), obj->volumes.end(), [&opt_has_scarf_joint_seam](const ModelVolume* v) { return opt_has_scarf_joint_seam(v->config.get());}) ||
               std::any_of(obj->layer_config_ranges.begin(), obj->layer_config_ranges.end(), [&opt_has_scarf_joint_seam](const auto& r) { return opt_has_scarf_joint_seam(r.second.get());});
    });

    if (has_scarf_joint_seam) {
        new_full_config.set("has_scarf_joint_seam", true);
    }
}
```

Supporting enum mapping:

```cpp
enum class SeamScarfType {
    None,
    External,
    All,
};

static t_config_enum_values s_keys_map_SeamScarfType{
    { "none",           int(SeamScarfType::None) },
    { "external",       int(SeamScarfType::External) },
    { "all",            int(SeamScarfType::All) },
};
```

The Rust staging must model:

- the `seam_slope_type` query key,
- `SeamScarfType::{None, External, All}` with `None` inactive and the other values active,
- object resolved config detection,
- volume override config detection only when that config has `seam_slope_type`,
- layer-range override config detection only when that config has `seam_slope_type`,
- `std::any_of` semantics across all objects and nested sources,
- setting `has_scarf_joint_seam` only when the computed bool is true.

## Non-goals / deferred behavior

- Do not implement real `DynamicConfig`, `ModelObject`, `ModelVolume`, `ConfigOptionEnum`, or fallback config lookup.
- Do not mutate a real `new_full_config`.
- Do not implement logging from `PrintApply.cpp:1155` or a real `BOOST_LOG_TRIVIAL` equivalent.
- Do not implement extruder variant expansion from `PrintApply.cpp:1157+`.
- Do not implement public APIs, UI/runtime wiring, profile loading, slicing, extrusion, G-code, new crates, dependencies, or Ares-owned pipeline behavior.
- Do not change existing public `SliceOptions` APIs.

## Acceptance criteria

- Staged evaluation records the scarf query key as exactly `seam_slope_type`.
- Staged evaluation uses the config-set key exactly `has_scarf_joint_seam` when it emits a set record.
- Empty object input produces `has_scarf_joint_seam = false` and no set record.
- Object-level `External` or `All` produces `has_scarf_joint_seam = true` and a set record.
- Object-level `None` does not produce true unless another source is active.
- Volume override `External` or `All` with the option present produces true.
- Missing volume override option and volume override `None` do not produce true unless another source is active.
- Layer-range override `External` or `All` with the option present produces true.
- Missing layer-range override option and layer-range override `None` do not produce true unless another source is active.
- Duplicate or multiple active sources still produce one staged set record.
- All new symbols stay private to `ares-core` staged modules.
