# O3 prerequisite — `handle_legacy` core on token + two thin adapters

架构裁定（协调者 2026-10-07，**最终**）：ratchet 单调收缩，只扫
`serde_json::Value`。给 `RawConfigValue` 补 Value 风格访问器并迁移
`options/` 是"换名藏动态值"，驳回；`SliceOptions.values` 存储类型本片不动
（typed 化是长期路线）。

本片 = O3 前置：**`handle_legacy` 核心只实现一份，纯 string 签名**：

```rust
fn handle_legacy(opt_key: &mut String, value: &mut String)   // PrintConfig.cpp:8033
fn handle_legacy_composite(/* &mut BTreeMap<_, RawConfigValue> */)  // :8290-
```

对应上游 `set_deserialize` 把每条 `key/value` 的**序列化 token** 交给
`handle_legacy`（`Config.cpp:577-579`）：key 置空 → 丢弃，value 置空 → 丢弃。

## 两个薄适配层

- **raw 适配层**（`project_settings_raw` 用）：遍历
  `BTreeMap<String,RawConfigValue>` 的 `entries()`，对每个值取
  `RawConfigValue::token()` 得 value 字符串，调核心 `handle_legacy(&mut key,
  &mut token)`；归一后用 `json_deserialize::decode(option_definition(key).kind,
  &token)` 解回 `RawConfigValue`；key/value 空则丢弃。`handle_legacy_composite`
  同法对整份 config（thumbnails/wiping_volumes/sla）。
- **Value 适配层**（`SliceOptions` 用）：**留在 `legacy.rs` 现有
  `normalize_legacy_option` / `normalize_legacy_option_into` /
  `normalize_legacy_options` / `normalize_legacy_composite_options` 函数里，
  函数名不变**。把这些函数里所有针对 `Value` 的 key 重命名 / value 改写逻辑
  **全部删掉**，只保留"取 token（`Value`→其字符串形式）→ 调核心
  `handle_legacy` → 写回 `Value`"这一步薄适配。

指纹按函数 + 出现序号计：适配层只**减少**现有函数里的 `Value` 出现次数，
不在任何新函数 / 新位置新增 `Value`。composite 同样一份核心 + 两适配层。

## 接点

- `project/load.rs`：在 `overrides.apply` + `normalize_option_scalars` 之后、
  `ProjectSettingsBuilder::deserialize` 之前，对 `project_settings_raw` 跑
  `handle_legacy` + `handle_legacy_composite` → `project_settings_raw` 成为
  **post-legacy** 视图，O3 range loop 读它（等价上游 `m_print_config`）。
- `SliceOptions::deserialize`（`legacy.rs`）：走 Value 适配层 → 同一核心。

## baseline 预期

- **严格收缩且只有删除行**：适配层删掉的 key/value 逻辑消掉 legacy.rs 里大量
  `serde_json::Value::String`/`Value` 出现次数（函数内 @N 序号下降 → 行删除），
  核心实现只用 `String`/`RawConfigValue`/`decode`，零新增 `Value`。
- `options/` 其余 `Value` 指纹本片不动。

## 验收

1. baseline diff 只含删除行（严格收缩，无新增）。
2. `legacy.rs` 里不再有 key/value 判断逻辑 —— `grep "enable_wipe_tower\|
   ironing_angle\|wall_infill_order\|..."` 在 legacy.rs 内为 0；这些字面量只在
   核心实现。
3. `ironing_angle:-1` 的 3MF E2E 走 post-legacy `project_settings_raw`
   （归一成 `0`，不误拒）。
4. `SliceOptions` 与 `project_settings_raw` 共用同一份 `handle_legacy` 核心；
   legacy.rs 只余薄适配。
5. 全量 `cargo nextest run --workspace`、clippy `-D warnings`、fmt、wasm 全绿。

若 Value 适配层需要的 `Value` 出现次数比现在**多**，即做不到零新增 —— 报告
具体行号，不硬做。

## 落地状态 (pi-tower)

已实现并经 7093 个 ares-core 单测验证：

- **核心**：`options/legacy/token.rs` 持有 `handle_legacy(&mut String, &mut String)`
  （纯字符串，`PrintConfig.cpp:8033` 语义；清键=删除）、`handle_legacy_derived`
  （`is_infill_first`，`Config.cpp:937-945`）、`handle_legacy_dest_bool`
  （`top_one_wall_type`→`only_one_wall_top` 的目的端 bool 判定）。`mod token` 而非
  `mod core` —— `mod core` 会遮蔽 `core` crate 引发 `rust-lld` hidden-symbol 链接错。
- **raw 适配层**：`ProjectSettingsRaw::handle_legacy`（`raw_settings.rs`）逐条
  `token()`→核心→存回 `RawConfigValue::Text`（不重解码，typed builder 之后再校验）；
  `is_infill_first` 经 `handle_legacy_derived` 落 `RawConfigValue::Text`。`load.rs` 在
  `normalize_option_scalars` 后、`ProjectSettingsBuilder::deserialize` 前调用 →
  `project_settings_raw` 成 post-legacy 视图。
- **Value 适配层**：`legacy.rs` 瘦成 ~90 行薄适配——`normalize_legacy_option`/
  `normalize_legacy_option_into`/`normalize_legacy_options`/`normalize_legacy_composite_options`
  只取 token→核心→写回 `Value`，全部 key/value 字面量已删（`grep` legacy.rs 无
  `enable_wipe_tower`/`ironing_angle`/`wall_infill_order` 等）。
- **O3 range loop**：`loaded.rs` 的 `collect_range_errors`/`elements_of`/`raw_element`
  改读 `RawConfigValue`（`Text`/`Number`/`Null`/`List`；`Flag`/`Object`/`List`-on-scalar
  → `InvalidInput`），`ProjectSettingsRaw::loaded_range_options` 直接读 `as_map()`
  （post-legacy），`SliceOptions::validate_loaded_range_options` 经
  `RawConfigValue::deserialize(value)` 桥接——零新增 `serde_json::Value`/`from_value` 指纹。

baseline `dynamic_value_baseline.txt` 再生成后 671→612（纯删除：删掉的 legacy.rs
`Value` 帮手 + `bridges.rs`/`gcode_wrapping_detection.rs` 早前的陈旧行）；ratchet 全绿。

**deferred**：`handle_legacy_composite`（thumbnails/wiping_volumes/sla）在 raw 视图
未接——`load_from_json` 的 composite 是 3MF 全局 config 的上一层；本片只交付 O3
需要的 per-entry `handle_legacy`。Value 侧 composite 仍走 `legacy/{sla,thumbnails,
wiping_volumes}.rs` 子模块（既有、已 baseline）。raw 侧 composite 统一核心留后续片。
