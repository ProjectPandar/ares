# SPEC: 未知 enum 键的静默默认替换——Config 反序列化边界对齐

状态：SPEC-REVIEW-REQUEST（stacked 窗口只读调研 + docs-only；未改代码）。
来源：zag-ironing RED 期间发现的解析分歧（协调裁定单独立项，排在该
切片后）。Native 证据已按指示先行采集。

## 1. Native 证据（官方 2.4.2，pristine overhang-organic fixture，CPU7）

| 输入 `support_ironing_pattern` | rc | ironing E 行 | 全文 sha（去时间戳行） | gcode 配置回显 |
|---|---|---|---|---|
| `cross-zag`（连字符未知键） | 0 | 719 | `4eb488251bff` | `rectilinear` |
| `bogus-pattern`（任意未知键） | 0 | 719 | `4eb488251bff` | `rectilinear` |
| `zig-zag`（legacy 迁移键） | 0 | 719 | 同上 | `rectilinear` |
| `rectilinear`（默认值显式） | 0 | 719 | 同上 | `rectilinear` |
| `zigzag`（规范键） | 0 | 127 | `f1390cf69b00` | `zigzag` |

要点：未知 enum 键 → **静默落选项默认值**（连 gcode 配置回显都是替换
后的默认值），无 stderr、无警告、rc=0。`zig-zag` 的行为一致是因为它
在 handle_legacy 阶段被改写为 `rectilinear`（同一默认值路径）。

## 2. 上游精确边界（OrcaSlicer 2.4.2）

1. `Config.cpp:577-579` `set_deserialize_nothrow`：先 `handle_legacy`
   （`PrintConfig.cpp:8203-8208`：六个 pattern 键 `zig-zag`→
   `rectilinear`；死键清空）。
2. `Config.hpp:2032-2040` `ConfigOptionEnum<T>::from_string`：查
   `enum_keys_map`（`PrintConfig.cpp:229-256`，仅规范键）未命中 → false。
3. `Config.cpp:654-672` `set_deserialize_raw` 替换分支：`coEnum`/
   `coEnums`/`coBool` 反序列化失败且 rule≠Disable →
   **`opt->set(optdef->default_value)`（落默认）**，`success=true`、
   `substituted=true`。
4. `Config.cpp:676-678`：仅 `Enable`/`EnableSystemSilent` 记录日志。
5. 3MF 加载链：`Model.cpp:259`
   `ConfigSubstitutionContext(EnableSilent)` → `_load_model_from_file`
   → `bbs_3mf.cpp:2632-2651
   _extract_project_config_from_archive` →
   `Config.cpp:820 load_from_json`（:929 字符串项
   `set_deserialize`）→ 上述替换路径。**EnableSilent = 全静默**。

## 3. Ares 现状（分歧点）

- `options/typed_legacy/actions/rules.rs:113-118`：六个 pattern 键已有
  `zig-zag`→`rectilinear` 改写（镜像 ：8206）✓——hyphen `zig-zag` 平价
  腿已通过。
- **分歧**：`options/typed_legacy/project/lexical.rs:22-43 assign()` →
  `deserialize_known_value` 对未知 enum variant 直接
  `Err`（`invalid legacy Orca option … unknown variant \`cross-zag\``，
  CLI exit≠0）——Native 同输入 rc=0 + 默认值输出。
- `options/validation/helpers.rs:20` 校验层同样只认 `zig-zag` 特例，
  未知值记 error 而非落默认。

## 4. GREEN 提案（获批后执行）

1. 在 typed_legacy 反序列化边界镜像上游 ：654-672：enum 目标遇到未知
   variant 时**静默落该选项的 registry 默认值**（不报错；范围=上游
   EnableSilent 的 3MF/项目加载路径，即 `assign()` 消费的那一侧）。
   coBool 的 `enum_looks_like_true_value` 分支一并镜像
   （`Config.cpp:667-668`）。
2. RED：ironing fixture 域内两腿——`support_ironing_pattern=cross-zag`
   与 `=bogus-pattern`（现 ares 解析错误 → 失败即 RED；oracle
   rc=0/719E/`4eb488251bff`/回显 rectilinear）；既有 20 腿（含 hyphen
   `zig-zag` 迁移腿与全部规范键腿）零回归。
3. GREEN 验收：两新腿官方+seeded 双模式逐字节；全量 directed 受影响
   面（option 加载相关既有腿）不变；strict clippy/fmt/wasm 于
   exact HEAD。

## 5. 边界声明

- 不触碰 pattern 语义/分派（本切片只对齐加载边界行为）。
- 上游 `Enable`（用户可见日志）路径不在范围（ares 无对应 UI 面）。
- 死键清除（handle_legacy 清空 opt_key → 忽略）已有 ares 行为不重查。
