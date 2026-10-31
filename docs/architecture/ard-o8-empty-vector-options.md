# ARD: 空向量选项的数据模型与 set_at 拒绝（O8 Cubicon）

日期：2026-10-06 · 状态：已批准 · 范围：options/json_deserialize、project/domain/profile_input/overlay.rs

## 决策

1. **数据模型（不可协商）**：向量类（vector-kind）选项的裸空字符串在
   raw 视图中是**空向量**（`List([])`），不是单元素文本。上游依据：
   preset JSON 标量分支（Config.cpp:927-929）→
   `ConfigOptionStrings::deserialize`（Config.hpp:1153-1159）→
   `unescape_strings_cstyle("")` 不追加任何元素（Config.cpp:146-149）。
   3MF 的 `project_settings.config` 走同一反序列化语义——Ares 的 catalog
   文档解析（profiles/catalog `insert`）与 3MF 内嵌 raw
   （project/load）共用 `options::json_deserialize::
   normalize_empty_vector_strings`，同一输入经两条路径必须同语义。
   typed 路径（wire decode）原本就产出空序列，本决策只是让 raw 对齐。

2. **拒绝面（不可协商）**：逐 filament 合并中供源为空向量时返回
   `InvalidInput("ConfigOptionVector::set_at(): Assigning from an empty
   vector")`（OrcaSlicer.cpp:3301 + Config.hpp:432-441 的消息原文）。
   遵守上游跳过列表（:3293）：compatible_printers / compatible_prints /
   model_id / dev_model_name / filament_settings_id 的空值合法（兼容门
   把空 compatible_printers 视为无限制），不拒绝。

## 已接受偏差

- 上游 CLI 因该 `ConfigurationError` 未被捕获而 `std::terminate` →
  SIGABRT（矩阵 exit 134，Export 阶段）；Ares 以 `InvalidInput` 错误返回
  并由 CLI 打印错误文本。拒绝类一致，呈现不同——这是与 O5 post_process、
  O6 上游 SIGSEGV 偏差同族的"上游崩溃 → Ares 结构化错误"决策。

## 影响面（vendor 全量扫描）

向量类裸空串仅三处：Cubicon `default_filament_colour` ×8（矩阵 3 例，
本决策的直接目标）、Overture `filament_notes` ×1（原生同样 SIGABRT，已
实测；Ares 同样拒绝，同拒绝类）、Prusa `process_common_mk3.json`
`compatible_printers` ×1（子级全部覆盖为数组、矩阵 corpus 1471 例中
不可达，且属跳过列表）。其余 70+ 高频裸空串键均为标量 String 类，不受
影响。

## 2026-10-07 精确 seed-replay 重建与调用栈复核

复核不新增生产逻辑、类型化错误或 key 前缀。三个目标为 Cubicon
xCeler-I / xCeler-Mini / xCeler-Plus 0.4 nozzle。

历史 `5cde90c23` 单次矩阵 Export command 引用的临时 preset 文件已删除。
按协调者批准，使用隔离的 `5cde90c23` worktree、未修改的矩阵导出器、
同 Cubicon vendor/default selection 重建。外部 launcher 在执行 Native
之前保存导出器写出的 machine/process/filament JSON 与原 cube10 STL。
这些明确标为**重建输入**，不声称与已删除文件逐字节一致。有效的有界
重建只有三个目标；初次 vendor 目录配置错误的已停止运行单独保留，不
计入本次结果。三个 filament JSON 的唯一空字段均为
`default_filament_colour: ""`，对应既有 coStrings 空向量语义。

Native 使用权威 seed-replay derivation
`72d4hac8qmg5p9m22v2kifbp1hkxjk63` 的 separate-debug 输出，原 binary
SHA256 `59fb88a2cb441713c2f572cb06e30e4ab58c22a676cacca1812372b9575e73ff`，
Build-ID `bdbbf4336095c2ce6283fc5441e1e56fbf595b99` 与调试文件匹配。
三个重建输入各执行一次 seed0/CPU3 gdb，均复现 SIGABRT 和相同 stderr：

```text
terminate called after throwing an instance of 'Slic3r::ConfigurationError'
  what():  ConfigOptionVector::set_at(): Assigning from an empty vector
```

调用栈均为 `std::terminate` → `__cxa_throw` →
`ConfigOptionVector<std::string>::set_at`（优化调试行归属 Config.hpp:445）→
`CLI::run`（OrcaSlicer.cpp:3301）→ `main`（:7543）。真正的显式空源检查
及抛出在 `Config.hpp:438-446`，消息所在行440；调用边界为
`OrcaSlicer.cpp:3290-3302` 的非 variant vector filament 合并。
这是**显式配置拒绝的异常未被 CLI 捕获**，不是十三例 singleton-selector
越界读的 UB，不模仿 abort，也不从 signal 推测新的检测规则。

Ares 现有 `profile_input/overlay.rs::filament_slot` 已返回同一消息的
`SliceError::InvalidInput`。在 main `5fed632db` 重跑既有
`crates/ares-core/tests/cubicon_empty_filament_colour_loads_empty_vector.rs`，三个真实 vendor
preset-chain 用例 **3/3 PASS**；它们验证同一合并拒绝语义，不冒充重建
Native 输入的逐字节公共 API 重放。既有检测无缺口，按裁定不新增
带 key 的类型化错误。通用 CLI 错误路径保持不变；原生 abort 与 Ares
结构化拒绝不能宣称 CLI stdout/stderr/退出状态逐字节相等。

重建 preset/model SHA、原命令与新参数、每例 gdb 栈与 log SHA：
`/tmp/worker3-cubicon-gdb/report.json`；保存输入：
`/tmp/worker3-cubicon-reconstructed/Cubicon*`。有效重建日志：
`/tmp/worker3-cubicon-bounded.log`（3 个 ORCA_ERROR，保留 sweep exit100，
不是 GREEN coverage）；Ares 回归日志：
`/tmp/worker3-cubicon-ares-existing-tests.log`。
