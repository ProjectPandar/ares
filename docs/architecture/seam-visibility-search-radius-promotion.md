# SeamPlacer 可见性样本搜索半径的双精度求值边界

本文记录只读诊断结论与验证边界。

## 上游所有权与不可协商决策

上游 pin：`8500fcdccaa10b5099ac20d252af3a7c560046f1`。

本片移植 `OrcaSlicer/src/libslic3r/GCode/SeamPlacer.cpp` 的 `SeamPlacerImpl::compute_global_occlusion`（:625–710），唯一生产替换点为 **:685 `float search_radius = sqrt(search_area / PI);`**。`:678–684` 的 probability、density、search_area 保持 `float`；`libslic3r.h:71` 的 `PI` 是 `double`，因此除法与平方根为双精度，最后赋值才舍入为 `float`。消费者为 `GlobalModelInfo::calculate_point_visibility`（:324–349）。

Rust 目的地仅为 `crates/ares-core/src/project_slice/seam_placement/visibility.rs` 的 `GlobalVisibility::from_mesh` 半径表达式。现有 `(search_area / PI).sqrt()` 使用该文件的 `std::f32::consts::PI`，是尚未匹配上游求值边界的临时兼容表达式，不是 Ares 独立 seam 策略。替换为 `(f64::from(search_area) / std::f64::consts::PI).sqrt() as f32`；保持 search_area 及前序计算为 f32，保留 hemisphere direction 使用的 f32 PI。

禁止修改全局 `Vec3::dot`、已合入的 visibility 距离 yz-then-x 归约、平面距离、KD-tree、采样、raycast、比较器、alignment、projection、travel、counterbore mask。禁止 epsilon、候选坐标特判、分数补丁、UB 仿真或 x86 专属路径；四个 Tier 1 平台使用相同显式 f64→f32 求值边界。

## 因果证据

冻结首层带孔输入 SHA256：`0a18241c2d988fb4f3500a41c6e1b2abe30aafbd8dacdb75012778cea1b55948`，原始 WEMAKE3D PhoenixProV1 0.4mm machine、0.2mm Standard process、Generic PLA @System，partiallybridge 无覆盖。公有完整输出首差在 line100：Native `G1 X141.8 Y142.029 Z.72 F15000`，Ares `G1 X139.574 Y143.212 Z.72 F15000`。

只读 Rust GDB 确认唯一 perimeter record 为 layer0、lower_layer_index=None，counterbore process_surfaces 未执行；差异不是 counterbore mask。Native 首个洞 seam 为 `(-5.25,-5.25)`，Rust 为 `(-5.25,+5.25)`，发生在 travel 请求前。

固定 Native ELF SHA256：`59fb88a2cb441713c2f572cb06e30e4ab58c22a676cacca1812372b9575e73ff`。只读捕获显示：

- 30000 个采样 positions（360000 bytes）、normals（360000 bytes）及 KD nodes（262144 bytes）均 Native/Rust **逐字节相同**，total_area 均为 `1064.9599609375`。
- Native radius `0.6549710035324097`，Rust radius `0.6549709439277649`。
- Native 全体 sample visibility 中有 292 个值不同，但本输入全部十个候选的附近样本值 **零差异**；该非局部现象不属于本片，不用它证明 raycast 全局等价。
- 全部十个候选的附近 sample ID 与遍历顺序与 Native 完全一致，使用任一半径均未改变本输入的 ID 序列。
- 在相同 Rust buffers 上仅替换半径，逐步 f32 离线回放从 **10/10 Rust visibility** 转为 **10/10 Native visibility**；每个候选的 total_weight、total_visibility 也精确吻合 Native 捕获，不只是最终除法值。切换 sample visibility buffer 不改变这十个结果。
- 首个洞候选：Rust总权重 `56.359222412109375`、总 visibility `90.17476654052734`、ratio `1.6000001430511475`；Native为 `56.35923385620117`、`90.17477416992188`、`1.600000023841858`。其余洞候选的变化保留严格比较器并解释 seam 选择变化。

半径独立回放：density_f32=`28.170072555541992`、neglogf=`0.10536054521799088`、search_area_f32=`1.3477023839950562`。f32 PI/平方根产生捕获的 Rust radius；仅将 `search_area / PI` 与平方根提升为 f64、末尾转换 f32，产生捕获的 Native radius。

Native `compute_global_occlusion` 模块相对地址 `0x100bd08–0x100bd56`：前序 `divss/mulss/divss`，然后 `cvtss2sd`（0x100bd2b）、`divsd`（0x100bd2f）、`sqrtsd`（0x100bd40）、`cvtsd2ss`（0x100bd4b）、写入 radius（0x100bd4f），与上述 C++ 类型提升边界一致。

所有 Native输出、maps、buffers、运行记录保留仓库外；无效的 optimized-out 探针不算证据，未初始化 unsupported 数据不参与论证。GDB exit0 不等于公有完整输出 PASS。

## 独立 RED / GREEN 边界

Included：上述搜索半径求值边界，及冻结首层带孔输入的公有完整输出端到端验证。

Deferred：292 个非附近 sample visibility 差异的独立归属诊断、其他 double/float 调用、KD 搜索/遍历、sampler/raycast、counterbore Bridges/Filled/Arachne、travel 及其他切片边界。

独立 RED：新增 standalone seam 完整输出测试，复用冻结首层 STL/stock 父链及 public `ares_core::slice_with_profiles`，不携带 counterbore GREEN 生产代码。**main 的 preflight 当前对任何带孔 partial 配置无条件拒绝，因此本片使用独立 `counterbore_hole_bridging=none` 控制**，Native/Ares 同时使用该覆盖并分别比较自己的完整 footer；不修改、不重命名、不计入原 counterbore 首层 partial 控制，不用它代替该控制 GREEN。上游 `PerimeterGenerator.cpp:1731–1735` 在 None 时立即返回，而 partial 在没有 lower_slices 时也不进入 surface rewrite，故 None 只绕开 main 的尚未支持 partial 守卫，不改变本片的几何/seam 所有权。独立 None 控制需要 fresh Native RED 验证其 radius、候选与原 partial 诊断一致，且完整输出确实在同一 seam/travel 症状失败；若不符则保留结果、重新确定范围，不调 fixture、不以 guard 拒绝充当目标 RED。

Native 必须通过 OrcaRunner 的 direct-STL recipe（arrange0/slice0）、运行时新生成并绑定 `ARES_ORCA_CACHE_DIR`；共享 Native availability contract，无 silent skip。仅规范化 generated-by identity，运动、M73、统计、footer 全保留。已有独立 seam None/solid、STL footer 及其他匹配控制不得改断言或调 fixture。

RED 独立提交后才编辑半径表达式。GREEN 需要首层及既有控制完整 PASS、匹配基线/候选 seeded 原始流与失败集审计、`cargo nextest run --workspace`、严格 Clippy、rustfmt、WASM 检查。Native seed0、新缓存，不重试至成功。

counterbore 切片保持 5PASS+1RED、不改实现或断言；本前置片完成后再重新验证 counterbore。离线数值回放不是 E2E GREEN。
