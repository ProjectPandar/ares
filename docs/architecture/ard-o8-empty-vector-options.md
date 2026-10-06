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
