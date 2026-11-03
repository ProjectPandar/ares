# SPEC: organic tree support single-worker merge layout（核数无关化）

状态：SPEC-REVIEW-REQUEST（只读调研完成；本文档不含任何代码改动，未运行任何构建/测试——W6 full 独占期间）。
裁定来源：用户直接裁定（2026-10-14）。本 SPEC 获批后将 supersede ARD-0102 的"do not pin
production to one worker"条款（见 §3）。

## 1. 上游精确引用

- `OrcaSlicer/src/libslic3r/Support/TreeSupport3D.cpp:2386`
  `const size_t num_threads = tbb::this_task_arena::max_concurrency();`
- `:2387-2391`
  `num_buckets_min = (input_size + 2) / 4`、`num_buckets_max = input_size / 2`、
  `num_buckets_initial = min >= num_threads ? min : max`、
  `bucket_size = min >= num_threads ? 4 : 2`。
- `:2410` `tbb::parallel_for(tbb::blocked_range<size_t>(0, num_buckets_initial), …)`。

上游的 merge 配对结构（bucket 布局）由运行机器的 TBB arena 并发度决定：
`num_threads=1` 时除极小输入（n∈{0,1}，`num_buckets_min=0<1` 仍走 max
分支）外均为 4-桶布局；`num_threads=N>1` 且节点数 `n < 4N-2` 时改走
2-桶布局。配对结构影响 `merge_influence_areas_two_elements` 的成对顺序，
顺序不同可产生不同中间影响域（条件合并阈值），进而改变树形输出。
即：**上游 Native 的 organic 输出本身随机器并发度变化**。

## 2. Ares 现状（移植点）

- `crates/ares-core/src/project_slice/support_material/tree/organic/generation.rs:215-226`
  `workers()`：非 wasm 用 `std::thread::available_parallelism()`，wasm 恒 1。
- `generation.rs:122` `pathing::create_layer_pathing(…, workers(), …)` 唯一调用点，
  作为 `execution=(layer, workers)` 传入。
- `…/organic/merge_areas.rs:23-31` 按上游 :2387-2391 同式选 4/2 桶布局。
- 后果：非 wasm 的 Ares CLI 输出随本机核数/亲和性变化（与上游行为同构）；
  WASM 与被 pin 的进程输出恒为 4-桶布局。

## 3. 裁定与有意偏离（ARD）

用户裁定：`workers()` 固定返回 1（与 WASM 分支一致），使 Ares 输出对核数/亲和性
不变。这是对上游机器相关行为（:2386 TBB arena）的**有意偏离**，理由：

1. 跨平台一致（Tier 1 四平台 + 浏览器 WASM）要求同输入同输出；当前非 wasm 端
   与 WASM 端在不同机器上不可互证。
2. 可复现切片是 Ares 测试基线（parity golden/comparison.json）的前提；任何
   依赖运行环境并发度的输出都会侵蚀该基线。
3. 性能代价：无。Ares 的该 merge 阶段本来就是串行执行（无任何并行调度），
   上游的 `tbb::parallel_for` 仅并行化首轮 bucket 处理，而 Ares 从未移植该
   并行调度；固定单 worker 只消除布局选择对运行环境的依赖，不损失任何
   已有并行度。后续若要并行化 merge，属独立性能切片（见 §8 deferred）。

**Supersede**：ARD-0102（2026-10-08）规定 "Keep the native available_parallelism
derivation …; do not pin production to one worker"。本 SPEC 获批后该条款作废，
替换为"生产恒单 worker"；supersede 不止于本文声明，将在实现提交中实际修订
ARD-0102 文件本体（§8）。ARD-0102 其余内容（reference 执行域界定、matched-worker
复核协议、Grid/Snug 归一化覆盖）不受影响，其中"matched-worker"在恒 1 后退化为
"Ares=1 × Native=oracle 机器"的既有形态（见 §5）。

## 4. 影响面（现有 organic parity 用例）

organic 生成器由所有走 TreeSupport3D 路径的 style 共用。现有比对用例
（`crates/ares-cli/tests/orca_parity/`）：

- `runner/tree_support.rs` 10 例：auto default/grid/snug/hybrid/slim/strong/
  strong-lightning/organic(zero-branch-angle) overhang + manual 汇总。
- `runner/tree_support/manual/styles.rs` 4 例：manual default/grid/hybrid/organic
  painted-overhang。
- `option_coverage/tree_activation.rs` 5 例与 `option_coverage/activation/support.rs`、
  `option_coverage/normal_support_output/style.rs` 中 tree style 触发项（测量阶段
  以 `workers()` 调用是否可达为准逐例标注）。

关键事实：parity harness 中 Ares 测试二进制经 `${TMP_ROOT}/ares-pin-runner.sh`
（`exec taskset -c "$cpu"`）单核执行，`available_parallelism()` 已恒为 1——
即**现有全部比对在 Ares 侧早已运行于 workers=1 形态**，改为常量 1 对 harness
内输出是 no-op，现有 organic Native 比对不应因此翻转。改动后 Ares 输出不再
依赖 pin-runner 的单核效果：pin-runner 此后仅为 Native oracle 调度与 worker
间 CPU 隔离服务，不再承担输出确定性职责。

Native oracle 侧现状：official 腿 native 不 pin（TBB arena=64），
seeded 腿 native `taskset -c N`（arena=1）。两腿现有 organic 用例均 PASS，
说明这些 fixture 在 Native arena∈{64,1} 与 Ares workers=1 之间输出一致
（n ≥ 4N-2 走同布局，或这些输入对配对顺序不敏感）。

## 5. 测量计划（先测后改，W6 full 结束后 directed 窗口执行）

M1 复现机器依赖（证明 bug 存在）：取 `tree-support/auto/organic/zero-branch-angle`
fixture（或构造小 n 输入），**先测该 fixture 各层 nodes 数**，确认存在某层
n < 6（N=2 时 4N-2=6，即 2 核下 num_buckets_min=(n+2)/4 < 2 触发 2-桶布局
差异；若全层 n≥6 则换更小输入重测并记录全组用例的 n 与布局）。随后当前
代码 `ares --slice` 两跑：单核腿 `taskset -c 7`，多核腿 `taskset -c 7,8`
（仅 2 核，不越出分配 CPU；不用 0-63），generator-only 比对预期**不同**。
M2 harness 控制组：现有 organic 全部用例在 pin-runner（现状）下改动前后
逐字节一致（control=0）。
M3 Native 对照不变：official+seeded 各一腿现有 organic 用例结果与基线一致
（Native 侧不改动）。

## 6. 测试计划（E2E，TDD，不放宽断言）

- 新增 E2E（`crates/ares-cli/tests/orca_parity/` 体系内，镜像既有 runner 结构）：
  同一 organic fixture，经 CLI 切片两次，子进程分别以单核与 2 核亲和性执行；
  CPU 集不写死机器编号：由测试自身读取允许的 CPU 集（环境变量如
  `ARES_TEST_CPUSET` 传入，缺省时用 `sched_getaffinity` 当前集取前 1 个与
  前 2 个），`taskset`/`sched_setaffinity` 相应设为 1 核与 2 核。
  `golden::compare_ordered_bytes_generator_only` 断言两份 G-code 逐字节一致。
  RED：当前实现不同亲和性输出不同 → 失败；GREEN：单 worker 化后一致。
- 现有全部 organic 比对用例原样保留，断言零改动；RED 阶段必须全绿（M2 控制组）。
- 不引入 `#[ignore]`、不弱比较、不环境跳过。

## 7. DIVERGENT 处理预案

若 M2/M3 中任一现有 Native 比对在改动后翻转 DIVERGENT（预期不发生，因 Ares
harness 侧已是单核形态）：
1. 先核验该翻转是否由本改动引起（revert workers() 常量化单独复跑该用例）；
2. 若确因本改动：说明该用例此前依赖非 harness 路径的多核输出（不可能，因
   pin-runner 已单核）→ 必为 harness 失效（pin-runner 未生效或测试越域），
   作为 runner 缺陷独立立案，不放宽断言、不合并本切片；
3. 任何情况下不得以调整 comparator/容差收编分歧。

## 8. 实现边界与 deferred

实现（获批后，测量 M1-M3 通过后）：**删除** `generation.rs` `workers()` 的
两个 cfg 分支及函数本身；删除 `pathing.rs` 的 `workers` 参数
（`pathing.rs:20`、调用点 `generation.rs:122`）与
`merge_areas.rs` `execution` 元组中的 workers（`(layer, workers)`→仅 layer
传递，`merge_areas.rs:23-26`）；`merge_areas.rs` 布局选择处**保留上游同式比较式并代入 num_threads=1**
（`num_buckets_min >= 1`，即 num_threads=1 代入）——不可简化为"恒 4 桶"：
n∈{0,1} 时 `num_buckets_min=(n+2)/4=0 < 1`，仍走 `num_buckets_max` 分支
（审稿更正）；注释引用 `TreeSupport3D.cpp:2386-2391` 与本 ARD。
同时在本实现提交中**实际修订 ARD-0102 文件**：在其条款处标注 supersede
指向本 SPEC/ARD（§3），不只在 SPEC 内声明。clippy/fmt/wasm；测试按 §6。
Deferred：merge 阶段并行化（Ares 本就串行，无存量并行度损失，见 §3.3；若
未来需要性能，另立性能切片，不改输出语义）；上游 `tbb::parallel_for`
并行调度本身（仅 bucket 布局参与语义）；其他 geometry gap。
