# Print applied filament_map：隔离晚期 ToolOrdering recommendation

## 上游 rewrite boundary

固定 OrcaSlicer v2.4.2 `8500fcdccaa10b5099ac20d252af3a7c560046f1`。本片是 `Print::validate` 的 applied config 与 `Print::process` 中 `GCode::ToolOrdering` 晚期 map 更新之间的生命周期 rewrite，不是“重复映射合法性”的自研规则。

- `OrcaSlicer.cpp:6038,6063-6094` apply→Print::validate；有错误返回205，不执行后面的 `:6146-6161 print->process`。
- `Print.cpp:2175::Print::process` 的层工具阶段 `:2429` 构建 `ToolOrdering(*this,...)`；sequential 路径 `:2460-2491` 汇集真实 layer filaments 后推荐 map 并更新。
- `GCode/ToolOrdering.cpp:1288-1303::ToolOrdering::initialize` 对 Auto 模式调用推荐，然后 `Print::update_filament_maps_to_config` (`Print.cpp:3166-3176`) 更新 `m_config` 和原 full config。这是 processing/pre-slicing 后的更新，不是 applied validation 之前的 override。
- `ToolOrdering.cpp:1107-1117::get_recommended_filament_maps` 从 **filament_colour.size()** 取 filament 数量；`:1152-1153` 初始化 master map、`:1204-1213` 非 BBL 多 nozzle 分支。不要改成 filament_diameter.len()，不要补色、不扩大数组、不通过 Manual/input repair 绕过真实原案。

Rust destination 是 `project/effective_config/{phases,types}.rs` 保存 validation 用的 applied map，与 `print/validate/extrusion_width.rs` 唯一 nozzle mapper 接线。现有 resolver 把 pre-validation 应用配置与暂提前产生的 processing/export 配置折叠成一个 `views.full`，这是上游生命周期的临时兼容 shell。本片拆开仅影响 Print validation 的 map 生命周期，不另造 pipeline。

## 已定位的错误链

真实原案：原 S1 Plus 3MF 不改，只公开 overrides：raft_layers=2、support_filament=2、support_interface_filament=2、filament_diameter=[1.75,1.75]、filament_map=[1,1]、initial_layer_print_height=0.3。原色数组仍单元素，物理 nozzle 仍一个0.2。Native 到 Print height检查，205精确 `Layer height cannot exceed nozzle diameter.`；Ares 更早 `invalid filament_map`。

**抛错者**是 `print/validate/extrusion_width.rs::nozzle_bounds` 的 `filament_map.get(filament).ok_or_else("invalid filament_map")`，不是 flat API 特有路径，也不是 cardinality 禁止重复。applied/flat 共用 mapper；失败的公开 project API 使用 applied 路径。

`effective_config/cardinality.rs::validate` 只要求 map.length == logical diameter count、元素是合法一基物理下标，**无唯一性要求**，原 [1,1] 通过。`domain/profile_input.rs::initialize_cli_loaded_project_config` 只插入 upward_compatible_machine，不改 map。

`effective_config/phases.rs::resolve_with_observer` 在两个 materialization/cardinality pass、最终 candidates/usage 之后，但返回 views、调用 Print validation 之前，执行 `options/filament_map_recommend.rs::apply_recommended_filament_map`。auto recommendation 按原色数1、单 nozzle/master1，覆写成 `[1]`；`project_config_views.rs::resolve_project_config_views` 将覆写的 config 原样放进 full。usage 仍含 support logical index1（第二耗材），lookup 对 length1 map 失败。**合法的 applied map 被晚期状态替换，失败不是 repeated physical index 的 rejection predicate。**

项目 global Manual override 的诊断实验仍失败，不是推荐 helper 忽略 Manual：原 `model_settings.config` plate metadata 指定 Auto For Flush；`project/plate_config.rs::apply_plate_config_overrides` 与 `OrcaSlicer.cpp:5979` 的 plate apply 让 plate 模式胜过 global。不要据此修枚举解析或 plate precedence。

## 不可协商的最小修复

在现有 resolver 最终 materialization/validation/candidates 完成、`apply_recommended_filament_map` **之前**保存一个 compact、owned applied `OrcaInts filament_map`，归 `BoundedResolvedProjectConfig` 所有。它是上游不同阶段状态的显式快照，不是 defensive clone 或另一份可变配置；不复制整个 ProjectSettings，不缓存另一套 used/nozzle 计算。

Print applied validation 的 nozzle_bounds 接收该 applied map 的 borrowed reference；flat/STL 仍接收自身 settings map。所有 min/max、one-based physical mapping、usage/support0/wipe/dedup、无效下标错误依然由**同一个 mapper**处理，不移除 ok_or_else、宽松截断、get_at fallback 或重复值特判来消掉症状。

当前 `views.full/runtime/runtime_gcode` 的 recommended map 和 auto helper时机、算法、colour count/normalization 保持完全不变，保护已有 processing/export 字节。这个仍提前构造的 export view 是明确的临时 compatibility shell；本片只禁止晚期 map 污染 applied Print validation，**不声称完整 ToolOrdering pipeline 时序已移植**。将来移植 `Print::process / ToolOrdering` 的原生时序后，必须删除这个 applied map 快照，把 recommendation 移回 validate 之后执行；快照不能成为永久架构。不为两个阶段创建泛化模式/旗标/第二解析器，不让纯 processing 隐藏调用 Print validation。

保留已合入 raft 的怪癖：raft首层 selector−1 **直接** nozzle get_at，不经 filament_map；0/minUsed、越界/front 不改。applied map 只用于现有普通 used-nozzle traversal，不把 raft direct lookup“修正”成它。不触碰 O3 options validation、layer-height/width predicates及文本、Flow/geometry/模板、CLI 参数/plate overlay。

## Included / deferred 与 TDD gate

包含 applied Print validation/config 与 late recommended/export map 的 compact阶段分离、两个 project公开validation/composition入口。profile validated入口共用同 owner；flat/STL语义不变。延后完整 ToolOrdering/FilamentGroup optimizer、晚期 map 重建的 runtime materialization/潜在进一步差异、全 pipeline 生命周期、colour/diameter不同长度的config export合法性；不声称原案能够成功产 G-code，Native 原案在 height 处拒绝。

批准后恢复原 shared-logical 的 native-backed public API 回归，先真实 RED：Native205 height/noGcode、Ares invalid_map/noGcode，原 hash/overrides/stdout/stderr留证。不得把 Native接受到 Print阶段偷换成 Native完整 slicing成功。之前撤回的测试/原BTeCte证据继续保留。

控制先写：合法 [1,1] 与正常非共享 map、auto recommendation不会改变独立Print诊断、plate Auto precedence、同原案 flat/项目 API 同源height错误、真正缺失/非法map仍由现有加载/应用边界拒绝、raft direct-index/front怪癖及原O4/upper/height首错不变。推荐/export parity 用当前base独立官方/seeded全集及480/469共同Ares G-code逐字节对照，不修输入/配置/模板或 comparator。新的 phase状态若使旧 private trace字段过时，仅向协调者逐条列明申请，不暗改旧断言。

测试 per-test kept tempdir 默认留证，不依赖 artifact env。门禁 plain workspace nextest、strict all-target/all-feature Clippy、fmt、WASM；每模块<=400LOC。日志/计数在外部证据或commit message，ARD仅记录源边界/判断/不可协商决策。未获本 spec 产品授权前不修改 Rust。
