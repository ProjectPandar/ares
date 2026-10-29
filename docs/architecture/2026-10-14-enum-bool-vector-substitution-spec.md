# SPEC: coBool/coEnums 未知值静默替换——Config.cpp:654-672 剩余分支

状态：SPEC-REVIEW-REQUEST（只读调研 + docs-only；未改代码）。
来源：enum 标量切片（APPROVED 284c7180e）批准条件中"enum vectors/
Bool 须另有 Native 证据"的后续。**Native 证据已采齐**（§1）。

## 1. Native 证据（官方 2.4.2，pristine overhang-organic fixture，CPU7/8）

### coBool（`support_ironing`，基线 "1"）
| 输入 | rc | ironing E | sha（去时间戳） | echo |
|---|---|---|---|---|
| `enabled` | 0 | **719（ON）** | `4eb488251bff`（=基线） | `1` |
| `true` | 0 | **0（OFF）** | `84b88a13e474` | `0` |
| `xyz` | 0 | 0（OFF） | `84b88a13e474` | `0` |

字面 `"true"` 落 **false**：`ConfigOptionBool::deserialize` 只认 "1"/"0"
（trim 后取首段，`Config.hpp:1828-1848`），失败即走替换
`value = enum_looks_like_true_value(value)`（`Config.cpp:666-668`；
`Config.hpp:96-99`：trim + iequals "enabled"/"on"）——"true" 不在集合内。

### coEnums（`filament_z_hop_types`，基线 `["nil"]`）
| 输入 | rc | E | sha | echo |
|---|---|---|---|---|
| `["Bogus Lift"]` | 0 | 719 | `e2e4d82988bd` | **`filament_z_hop_types = Slope Lift`** |

整向量静默替换为选项默认值（nil→Slope Lift 连回显都变了——替换
可观测）。

### ares 现状（本分支 CLI 实测）
- bool 三变体：`InvalidInput("invalid project settings JSON: invalid
  Orca option support_ironing: expected Orca boolean 0 or 1")` 硬错。
- zhop：`invalid Orca option filament_z_hop_types: unknown variant
  \`Bogus Lift\`…` 硬错。
- 两层同为直连键路径（project_settings JSON，非 legacy assign）。

## 2. 上游边界（补充自前一 SPEC）

- `Config.hpp:1828-1848` `ConfigOptionBool::deserialize`（"1"/"0"）。
- `Config.cpp:666-668` coBool 失败 → `looks_like_true`；`Config.hpp:96-99`
  （trim + iequals "enabled"/"on"）。
- coEnums：`ConfigOptionEnumsGeneric` 等 coEnum 向量 `deserialize` 逐项
  `from_string`，任一未知 → false → `Config.cpp:668-671`
  `opt->set(optdef->default_value)`（**整向量**替换单默认项）。
- 直连字符串项统一走 `Config.cpp:927-929 set_deserialize` → 同一
  `set_deserialize_raw` 替换分支（:654-672）。

## 3. Ares 边界与 GREEN 提案

- 直连键解码层：`options/json_deserialize.rs:22-43`——逐键
  `decode(kind, value)`，失败即 `invalid Orca option {key}: …` 硬错
  （两处证据错误均出自此层）。
- GREEN：在该层镜像替换——`decode` 失败且
  - `kind == Bool` → 按 `looks_like_true`（trim + ASCII iequals
    "enabled"/"on"）以 "1"/"0" 重试；
  - `kind == Enums | EnumsNullable` → 以 `default_value`（单元素 wire）
    重试（整向量替换语义）。
  - 其余 kind 保持硬错（无证据不动）。
- legacy 路径（`typed_legacy/project/lexical.rs substitute_enum_default`）
  本切片不扩展：证据两键均为直连路径；legacy 命名键命中 bool/enums
  失败时再凭证据立项。

## 4. RED 计划（单独提交）

ironing 域三腿（helper 改写 `support_ironing` / `filament_z_hop_types`）：
- `unknown_bool_enabled` → oracle 4eb488251bff / 719E / echo 1
- `unknown_bool_true` → oracle 84b88a13e474 / 0E / echo 0
- `unknown_vector_enum_zhop` → oracle e2e4d82988bd / 719E / echo
  Slope Lift

现三腿全为 ares 硬错 → RED。既有 23 腿零回归。GREEN 验收：official+
seeded 双模式 26 腿；strict/fmt/wasm 于 exact HEAD。

## 5. 边界声明

- 不改 `OrcaBool` 访问器（"1"/"0" 语义镜像 `Config.hpp:1828-1848`）；
  替换发生在加载层。
- `looks_like_true` 集合仅 "enabled"/"on"（上游 `Config.hpp:96-99`），
  不含 "true"——RED 腿 `unknown_bool_true` 即钉此陷阱。
- coBools（向量 bool）的 `deserialize_with_substitutions`
  （`Config.hpp:1919+`，DefaultsToFalse 机制）不在范围：无证据不动。
