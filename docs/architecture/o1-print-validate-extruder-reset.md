# O1 spec — Print::validate 的逐层 E 重置一致性

## 上游边界

本切片重写 OrcaSlicer v2.4.2 `src/libslic3r/Print.cpp:1695-1749` 的 `Print::validate()` G92 校验块，不是 `DynamicPrintConfig::validate()`。O2、O4、其余 Print 校验、模板求值及结构化 object/opt_key 错误 API 延期。

公开生命周期与调用边界见 [Print 校验与处理分离 ARD](o1-validate-process-separation.md)。现有 Print/STL scaffold 仅作上游生命周期的临时兼容壳，不新增 Ares-owned pipeline。

## 规则与不可协商决策

- `Print.cpp:1700-1706` 在原始模板上逐行搜索两个 Boost 正则：`^[ \t]*[gG]92[ \t]*[eE](0(\.0*)?|\.0+)[ \t]*(;.*)?$` 与对应大写 G92/E 版本。不用 contains、trim、大小写归一化或先渲染模板代替；注释中的 `;G92 E0` 不算命令。
- 相对 E：模板有任意大小写匹配却无大写匹配时，先报大小写错误，before 模板优先。仅 `marlin`/`marlin2`、非 BBL 且两个模板均无大写匹配时，报缺少 reset。一个模板同时有正确和错误大小写时，不加强拒绝条件。
- 绝对 E：所有 flavor/BBL 均禁止任一模板中的任意大小写匹配；before 优先，再 layer。不自动插入/删除 G92、修复 stock preset 或豁免默认/稀疏配置。
- BBL 身份来自 `src/OrcaSlicer.cpp:1730-1765,6043-6059`：非空 printer_model 优先，否则 selected printer 名，再用项目 system printer 名（inherits_group 最末 printer 槽；无继承用 printer 名）；前缀严格为 Bambu Lab。沿用现有配置/预设元数据，不改变 config-export 的 model 判断。
- Rust 规则由 `ares-core/src/print/validate.rs` 统一拥有，返回 `SliceError::InvalidInput` 携带精确上游文本。该文本适配对应 `src/libslic3r/PrintBase.hpp:30-39::StringObjectException`，不得复制到 CLI/WASM 或混入配置反序列化。
- CLI 保留非零失败退出和不产生新 G-code 的合同，不新增退出码体系。上游 `src/OrcaSlicer.cpp:6063-6094` 的 `CLI_VALIDATE_ERROR(-51)` 对应 native exit205。

## 精确错误表

| 分支 / upstream opt_key | 文本 |
| --- | --- |
| relative 错误大小写 / before_layer_change_gcode | `"G92 E0" was found in before_layer_change_gcode, but the G or E are not uppercase. Please change them to the exact uppercase "G92 E0".` |
| relative 错误大小写 / layer_change_gcode | `"G92 E0" was found in layer_change_gcode, but the G or E are not uppercase. Please change them to the exact uppercase "G92 E0".` |
| relative 缺 reset / before_layer_change_gcode | `Relative extruder addressing requires resetting the extruder position at each layer to prevent loss of floating point accuracy. Add "G92 E0" to layer_gcode.` |
| absolute / before_layer_change_gcode | `"G92 E0" was found in before_layer_change_gcode, which is incompatible with absolute extruder addressing.` |
| absolute / layer_change_gcode | `"G92 E0" was found in layer_change_gcode, which is incompatible with absolute extruder addressing.` |

配置 wire key 是 layer_change_gcode，不是缺 reset 诊断中的 layer_gcode 成员名。

## E2E 验收合同

- 覆盖原始 Anycubic Kobra Max/Plus/Vyper、Raise3D Pro3 Plus Dual/Left/Right、Wanhao D12-300 的 relative 缺 reset，以及 Dremel 3D40/45 的 absolute before reset。
- 仅固定两类最小未修改原始 3MF：`tests/parity/validation/layer-reset-kobra-plus-relative.3mf` 与 `layer-reset-dremel-3d40-absolute.3mf`；其余原始案例在完整 stock sweep 验证，不入库。
- 拒绝证据必须同时满足 native exit205/O1 精确文本、Ares InvalidInput/同文且无新 G-code；其他错误、崩溃或单侧拒绝不得算通过。双方拒绝不得算有效输出的 byte PASS。
- 覆盖两模板、大小写、注释、零数字形式、STL/project/profile/processed 边界；合法大写 reset 的非 BBL Marlin、无 reset 的非 BBL 非 Marlin、无 reset 的 BBL Marlin 不误拒。
- 合法配置保持既有 G-code 字节比较器，错误保持精确类型/文本断言。真实 Orca E2E 的 RED 先于产品实现；全量 workspace nextest、Clippy、fmt、wasm32 以及 official/seeded 双侧对照为验收门禁，参考故障独立报告。
