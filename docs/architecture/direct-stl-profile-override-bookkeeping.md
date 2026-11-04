# Direct-STL profile override 的 CLI bookkeeping 边界

范围：独立 seam 前置片；不包含 seam 或 counterbore 生产修改。

## Finding

OrcaSlicer source pin `8500fcdccaa10b5099ac20d252af3a7c560046f1`。

公有 `ares_core::slice_with_profiles` 在 STL 输入带非空 overrides 时，借用 3MF profile overlay，误执行“将 BBL 3MF 所有 filament 映射到同一 preset”的 bookkeeping 分支。原始 stock profile＋唯一 `counterbore_hole_bridging=none` 覆盖使 Ares footer 的 process slot 变成 `enable_prime_tower`；独立 Native direct-STL 输出的该 slot 为空。这不是 seam 路由差异，也不是 5128 的 `Print::apply` 单次归一化回归：旧 counterbore RED 的原始 Ares 流在首次 line100 travel 差异之后已经保留相同 footer 缺陷。此既有问题作为独立 STL 前置片处理。

## 上游来源与为何 Native 为空

Native direct-STL recipe 为 `--load-settings machine.json;process.json --load-filaments filament.json --arrange 0 --slice 0 input.stl`，保持文件内容与命名，不先 export 3MF。

`OrcaSlicer.cpp` 责任链：

- :2073–2088 加载具名 process 配置与 system/inherits 身份，:2092–2095 可读取输入配置已有的 `different_settings_to_system` process slot。这是加载已有 metadata，不是对 override 自动求 dirty diff。
- :2815–2821 创建并调整 bookkeeping vector 到 `filament_count+2`；:2838 与 :2978 的具名 machine/process 装载路径清空对应 slot，:3155–3161 对装载 filament 重建 slot。此例一 filament 的 vector 为三个空字符串。
- **:2275–2277** 仅当 `is_bbl_3mf && load_filament_count>0 && load_filaments_set.size()==1` 才启用 `disable_wipe_tower_after_mapping`。STL 不是 BBL 3MF，即使只有一个 filament，也不进入此分支。
- :3673–3699 只有上述 mapping flag 有效且不是 smooth timelapse 才禁用 prime tower，并向 process diff slot 追加 `enable_prime_tower`。本例 STL 的 flag 保持 false，不能执行这个 metadata 追加。

`libslic3r/PresetBundle.cpp` 的 GUI/导出组成边界不同：:3926–3931、:3960–3965、:4101–4106 调用 `PresetCollection::dirty_options_without_option_list` 取得 print/filament/printer 与 parent 的差异；:4117 移除原单 preset 字段，:4146–4160 用 `add_if_some_non_empty` 重新组成 grouped vector。`Preset.cpp:3539–3564` 负责比较 edited/reference 的公共配置键并过滤 ignore list。不得把这条 GUI dirty-preset 路径当成 CLI STL 运行时任意 override 的自动差异计算器。

`libslic3r/GCode.cpp:5591–5593` 的 `GCode::append_full_config` 取 `Print::full_print_config`；:5632 起按 keys/is_nil 过滤及序列化。它输出已存在的配置 token，不在 footer 层重新推导 diff。三个空 slot 按 `ConfigOptionStrings` 的既有序列化成为 `;;`，不是要求删除此行。

## Rust 目的地与不可协商决策

唯一生产边界：`Project::from_stl_with_profiles`（原 `crates/ares-core/src/project/domain/profile_input.rs`，为满足文件≤400LOC，STL constructor 分出至 `profile_input/stl.rs`）。原非空 overrides 分支把空 embedded view 传给 `overlay::merged_profile_raw`，复用其 3MF slot 重建及 mapping 行为；`overlay.rs:159–190` 根据 single preset 禁用 tower，:328–351 `different_settings_slot` 将 `enable_prime_tower` 写入 slot0。这里是错误兼容壳，不是新的 Ares 配置合成设计。

替换为与本函数已支持的 no-override STL 路径相同的已组合配置/raw 起点。此起点不仅是稀疏 `profile.profile_raw`：`profiles/composition.rs:186–211` 向 typed `project` 注入 selected print/printer/filament IDs、filament map/IDs/self-index 与 print compatibility。两条 STL 分支使用同一份 `profile.settings.project` 来源，在分支前把这些 composition-owned 字段合入共同 raw；不在反序列化后旁路补字段。Native 在 `OrcaSlicer.cpp:2048,2088,3152–3158,3175–3180` 写入 loaded preset identities，`:3613–3615` 再 `m_print_config.apply(m_extra_config,true)`（command-line options override loaded files）。非空用户 overrides 按此最后应用顺序写入共同 raw，保留 `handle_legacy` 与 typed deserialize/resolve，然后继续原 STL import、模型身份、placement、validation 与 slicing 生命周期。coStrings 的 logical List/中央 export escape 由通用边界修复拥有：本片只调用 `insert_strings`，不改共享 helper、不保留 `insert_string_slots` alias、不手写替代转义路径。

不得调用 3MF overlay 进行 direct-STL override 合成。不得新增格式 flag、fallback、并行 raw/config 状态，或在 footer 强制清空 token。STL 输入应继承原 profile raw 明确携带的 metadata；缺省 grouped key 由原配置导出器 `options/config_export/writer.rs:212–220` 按 CLI slot 个数输出默认空 vector。3MF 的 embedded slot、mapping、颜色/timelapse 语义不变。

Included：仅 direct-STL＋非空 overrides 的合成起点纠正，保留原 override/legacy/typed 流程。Deferred：3MF mapping gate 本身的其他行为、GUI dirty-preset diff、其他 metadata、全局 normalization、seam Eigen 距离、counterbore、travel、layer range、by-object、role filament。

## RED 与验证结论要求

主 RED：冻结 `solid-plate.stl`（SHA256 `252418ebf4f2e4c9c0685ec615a5d98c6f9fdb6312cdba6a12457cf49e5b10de`），原始 WEMAKE3D PhoenixProV1 0.4mm machine、0.2mm Standard process、Generic PLA @System，唯一覆盖 `counterbore_hole_bridging=none`。公有 `slice_with_profiles`、原始 catalog/selection、`Placement::None`、固定 seed/metadata。无孔 solid 避免 seam 前置故障，当前完整输出首差是 `different_settings_to_system`。

Native 在运行时经 OrcaRunner 的相同 direct-STL recipe 和 `ARES_ORCA_CACHE_DIR` 生成，绑定 producer/seed/resources/实际 STL 与 flattened profile bytes；不提交 Native 产物或过程记录。完整输出 comparator 仅规范化 generated-by identity，保留 footer、movement、M73、统计；不接受 witness-only GREEN。

生产前冻结 no-override STL whole-output 控制及已有 3MF profile-overlay/mapping controls。

旧 first-difference 被 seam 修正消除不等于此完整输出已 GREEN。
