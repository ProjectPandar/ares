# SeamPlacer 可见性距离的 Eigen 三维归约顺序

状态：待独立 spec 审批；仅结论与 RED 计划，不含生产改动或 GREEN 声明。

## Finding 与不可协商边界

上游 pin：`8500fcdccaa10b5099ac20d252af3a7c560046f1`。

责任边界为 `OrcaSlicer/src/libslic3r/GCode/SeamPlacer.cpp`：

- `GlobalModelInfo::calculate_point_visibility`（:324–349）：按附近样本顺序累计可见性权重。
- **:342 `(position - sample_point).norm()`**：本片唯一替换调用点。Native 的 Eigen 三维 squaredNorm 先加 `y²+z²`，再加 `x²`，然后取平方根。
- `SeamComparator::is_first_better`（:753–797）与 `pick_seam_point`（:917–925）：保留严格小于与原始候选顺序，不修改比较器。

Rust 目的地仅为 `crates/ares-core/src/project_slice/seam_placement/visibility.rs` 的 `GlobalVisibility::at` 距离权重表达式。现有 `.norm()` 经 `mesh::Vec3::dot` 使用 `(x²+y²)+z²`，在这里不是上游 Eigen 归约的等价实现；它是需要替换的临时兼容表达式，不是新增 Ares seam 策略。

采用局部名称 `eigen_yz_then_x_squared_norm`，以普通 `f32` 乘、加与平方根明确写出 **`(dy*dy + dz*dz) + dx*dx`**；在修改处注明上游调用点及下述 asm 证据。不得全局修改 `Vec3::dot`，也不得改变平面距离、KD-tree 搜索/顺序、mesh sampling、raycast、角度罚分、alignment、loop projection 或 travel。

这不是 epsilon、末位修补、候选坐标特判、UB 仿真或 x86 指令模拟。它是该上游调用点的实际求值顺序移植；WASM、Windows、macOS、Linux 都使用相同显式标量 `f32` 运算。

## 已确认的因果结论

冻结输入为 `tests/parity/counterbore-partiallybridge/stepped-hole.stl`，SHA256 `1274c56d97fd205740505d88f077971c7ffe79f1552df78a4e7432de557e465e`。原始 WEMAKE3D PhoenixProV1 0.4mm machine、0.2mm Standard process、Generic PLA @System 保持不变，唯一覆盖仍是 `counterbore_hole_bridging=none`。

Native 首层方孔候选 0 的 visibility 为 `1.600000023841858`，候选 2 为 `1.5999999046325684`；Rust 候选 0 为 `1.5999997854232788`，候选 2 与 Native 相同。角度同为 `-π/2`，overhang 与 embedded 输入均为零。严格罚分比较因此分别选择 `(5.25,5.25)` 与 `(-5.25,-5.25)`。选点差异发生在 travel 请求产生前，与 avoid-crossing、wipe 状态或 worker2 的 router 修改无关。

固定 Native ELF SHA256 `59fb88a2cb441713c2f572cb06e30e4ab58c22a676cacca1812372b9575e73ff` 的 `calculate_candidates_visibility` TBB `run_body` 中，模块相对地址 `0x1017941–0x1017965` 的 `mulps/mulss`、`shufps $0x55`、两次 `addss`、`sqrtss` 证明距离计算为 `(y²+z²)+x²`。平面距离的归约不在此变更范围。

同一批捕获的 Rust 样本、normal、visibility 值与 KD-tree 在离线逐步 `f32` 回放中：左结合重现 **15/15 Rust visibility**；只将距离 squaredNorm 换为 yz-then-x 后重现 **15/15 Native 候选表 visibility**。附近样本 ID 集及顺序均未改变；只发生三个候选值变化（0、4、6，包含重复闭环端点），其他十二个保持原值。首个方孔位置的总权重由 `44.07510757446289` 变为 `44.075103759765625`，总 visibility 累计保持 `70.5201644897461`，因而改变除法结果及选点。

完整的 15 行逐候选绑定和离线回放证据保存在仓库外：

- `${TMP_ROOT}/ares-none-travel/visibility-reduction-audit.json`：位置、Native/Rust/两种离线值、相同附近 ID 顺序的逐行核验。
- `${TMP_ROOT}/ares-none-travel/offline-callsite-distance.json`：SHA256 `510c94a26aab4e05753a93567ac61af79797f4d730c2ce763a3ae1326af484e7`；对应 `offline_callsite_distance.py` 保持 KD-tree 与平面距离的原求值顺序，只改变该距离调用，结果与原离线回放逐字节相同。
- `${TMP_ROOT}/ares-none-travel/native-selection-events.json`：SHA256 `9247cb5e84bbb76d15902c8898d596b87744d57b2d752a82e276b03de0634ff0`。
- `${TMP_ROOT}/ares-none-travel/ares-visibility-events.json`：SHA256 `2cea89cb6bbf46ccef4b01f49fb777cf2ddf68bb1ec9015b5abb02f71972ebda`。
- `${TMP_ROOT}/ares-none-travel/native-visibility-disasm.txt`：SHA256 `c8edc253a64cd9e45cd39496395a903ac270754d164c3c157a7f9738d98bb9d7`。

这些是仓库外诊断证据，不是提交的 oracle fixture，也不替代公有 E2E GREEN；`unsupported_dist` 不参与本片的选点论证。

## Included / deferred

Included：上述可见性距离调用的 Eigen 归约顺序，以及它在公有 profile slicing 输出上的端到端验证。

Deferred：全局向量归约改写、其他 Eigen 调用、sampler/KD-tree/raycast 改进、seam 比较器或闭环重复端点处理、travel、counterbore Bridges 实现、by-object、layer ranges、role filament。尤其不能把本片合进 counterbore GREEN；它必须独立批准、RED、GREEN、门禁、审查并合入。

## RED 计划与退出标准

1. 复用冻结 None-control 输入及原始 stock 父链，通过公有 `ares_core::slice_with_profiles`、`Placement::None`、相同 metadata/context 执行。Native 每次在运行时通过 OrcaRunner 的相同 direct-STL recipe 生成，并走 `ARES_ORCA_CACHE_DIR`；producer、输入、flattened profile 与 recipe 进入缓存身份。不改成 3MF export，不改变 arrange=0 或 geometry 顺序。缓存适配及未合入 RED 的过程记录清理须在重新提审前完成。
2. 主 RED 保留现有完整输出硬断言，预期在当前生产基线上仍于 line100 失败。比较仅规范化 generated-by identity，保留运动、M73、统计与 footer；不得只检查目标坐标或接受差异。
3. 在生产编辑前冻结不改变测试输入的控制：原有 solid-control、已支持的 seam/public slicing 控制。对已批准 counterbore RED，四例守卫拒绝必须保持拒绝、solid 必须保持 PASS；本片预期只消除 None-control 的既有差异，不宣称 counterbore 支持完成。
4. 定向 Native 采用 CPU0、seed0、runs1、独立 fresh cache、每例共享 300s deadline 及 owned-descendant 清理；预编译在 deadline 外，Rust1.99、16 threads、禁止重试至成功。Native `.gcode`、stdout/stderr/result、GDB map、运行过程记录均保存在仓库外，不提交。
5. 独立 RED 审批后才能修改上述一个距离调用点。GREEN 要求原始 None-control 完整输出 PASS，控制断言保持不变，并完成 matched base/candidate seeded/official 比较、`cargo nextest run --workspace`、严格 Clippy、rustfmt、WASM 检查及独立 REVIEW。不得用离线算式吻合替代公有 E2E GREEN。

spec 审批与 RED heavy 是独立门禁；BROWSER HOLD 期间不启动 RED heavy。此文不授权生产编辑。
